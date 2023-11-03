use crate::{error::ScriptError, parser::*};
use std::path::Path;

pub struct Transpiler
{
    pub parsed_input: ParsedUserInput,
}

impl Transpiler
{
    pub fn new(unparsed_input: UnparsedUserInput) -> Result<Self, ScriptError>
    {
        let parsed_input = unparsed_input.parse()?;
        Ok(Self { parsed_input })
    }

    pub fn from_toml_path(path: &Path) -> Result<Self, ScriptError>
    {
        let content = std::fs::read_to_string(path).map_err(ScriptError::ErrorReadingToml)?;
        let user_input: UnparsedUserInput =
            toml::from_str(&content).map_err(ScriptError::ErrorParsingToml)?;
        Self::new(user_input)
    }

    fn parameter_decl(&self) -> String
    {
        if self.parsed_input.param_names.is_empty() {
            return "type Parameters = NoParam;".to_owned();
        }

        let member_decls: Vec<String> = self
            .parsed_input
            .param_names
            .iter()
            .map(|name| format!("{name}: Cplx,")) // adjust the format as needed
            .collect();

        let derives = "#[derive(Clone, Copy, Default, PartialEq, Debug)]";

        format!(
            "{}\npub struct Parameters {{\n{}\n}}",
            derives,
            member_decls.join("\n")
        )
    }
    fn parameter_impls(&self) -> String
    {
        // Necessary impls aready provided for NoParam
        if self.parsed_input.param_names.is_empty() {
            return String::new();
        }
        let names = self
            .parsed_input
            .param_names
            .iter()
            .map(|c| format!("self.{c}"))
            .collect::<Vec<_>>()
            .join(", ");
        let names_and_values = self
            .parsed_input
            .param_names
            .iter()
            .map(|c| format!("{c}: {{}}"))
            .collect::<Vec<_>>()
            .join(", ");
        let display_impl = format!(
            "impl std::fmt::Display for Parameters {{\n\
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result\
                {{\n\
                    write!(f, \"[{names_and_values}]\", {names})
                }}\n\
            }}"
        );

        format!(
            "impl Named for Parameters {{}}\n\
        impl From<Cplx> for Parameters {{
            fn from(_value: Cplx) -> Self {{
                unimplemented!()
            }}
        }}\n\
        {display_impl}\n\
        impl Describe for Parameters {{
            fn describe(&self, conf: &DescriptionConf) -> Option<String> {{
                conf.is_enabled.then(|| self.to_string())
            }}
        }}"
        )
    }

    fn user_struct_decl(&self) -> String
    {
        let const_decls: Vec<String> = self
            .parsed_input
            .constants
            .iter()
            .map(|(name, value)| {
                format!(
                    "const {}: Cplx = Cplx::new({:?}, {:?});",
                    name, value.re, value.im
                )
            }) // adjust the format as needed
            .collect();

        let derives = "#[derive(Clone, PartialEq, Debug)]";

        format!(
            "{derives}\n\
            pub struct UserPlane {{\n\
                point_grid: PointGrid,\n\
                max_iter: Period,\n\
            }}\n\
            impl UserPlane {{\n\
                const DEFAULT_BOUNDS: Bounds = Bounds::centered_square(2.5);\n\
            }}\n\
            impl Default for UserPlane {{\n\
                fractal_impl!();\n\
            }}\n\
            const i: Cplx = Cplx::new(0., 1.);\n\
            {}",
            const_decls.join("\n")
        )
    }

    fn destructure_param(&self) -> String
    {
        format!(
            "Parameters {{ {} }}",
            self.parsed_input.param_names.join(", ")
        )
    }

    fn parameter_plane_impl(&self) -> String
    {
        format!(
            "impl ParameterPlane for UserPlane {{
    type Param = Parameters;
    type Var = Cplx;
    type MetaParam = NoParam;
    type Deriv = Cplx;
    type Child = JuliaSet<Self>;
    basic_plane_impl!();

    fn param_map(&self, {t}: Cplx) -> Self::Param {{
        {param_map}.into()
    }}

    fn escape_radius(&self) -> Real
    {{
        1e26
    }}

    fn start_point(&self, {t}: Cplx, {c}: Self::Param) -> Self::Var
    {{
        {start}.into()
    }}

    fn start_point_d(&self, {t}: Cplx, {c}: Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {{
        let (z0, dz0_dt) = {{
            {start_d}
        }};
        (z0.into(), dz0_dt.into(), ZERO)
    }}

    fn map(&self, {z}: Self::Var, {c}: Self::Param) -> Self::Var
    {{
        {map}
    }}

    fn map_and_multiplier(&self, {z}: Self::Var, {c}: Self::Param) -> (Self::Var, Self::Deriv)
    {{
        {map_d}
    }}

    fn name(&self) -> String
    {{
        \"{name}\".to_owned()
    }}

    fn default_bounds(&self) -> Bounds
    {{
         Bounds::centered_square(2.5)
    }}
}}",
            t = self.parsed_input.names.selection,
            z = self.parsed_input.names.variable,
            c = self.destructure_param(),
            param_map = self.parsed_input.py_params.param_map,
            map = self.parsed_input.py_params.map,
            map_d = self.parsed_input.py_params.map_d,
            start = self.parsed_input.py_params.start,
            start_d = self.parsed_input.py_params.start_d,
            name = self.parsed_input.metadata.name,
        )
    }

    fn other_impls(&self) -> String
    {
        format!(
            "impl InfinityFirstReturnMap for UserPlane {{
    #[inline]
    fn degree(&self) -> AngleNum
    {{
        {degree}
    }}
    #[inline]
    fn degree_real(&self) -> Real
    {{
        {degree} as Real
    }}

    fn escaping_phase(&self) -> Period {{
        {escaping_phase}
    }}

    fn escaping_period(&self) -> Period {{
        {escaping_period}
    }}
}}

impl EscapeEncoding for UserPlane {{}}
impl ExternalRays for UserPlane {{}}
",
            degree = self.parsed_input.optional.degree,
            escaping_period = self.parsed_input.optional.escaping_period,
            escaping_phase = self.parsed_input.optional.escaping_phase,
        )
    }
    fn imports() -> String
    {
        "use dynamo_common::prelude::*;\n\
        use dynamo_core::prelude::*;\n\
        use dynamo_gui::interface::{MainInterface, Interface};\n\
        "
        .to_owned()
    }

    fn constructor() -> String
    {
        "#[no_mangle]\n\
        pub unsafe fn create_interface() -> *mut dyn Interface {\n\
            let parent = UserPlane::default();\n\
            let child = <UserPlane as ParameterPlane>::Child::from(parent.clone());\n\
\
            let int = MainInterface::new(parent, child, 768);\n\
            Box::into_raw(Box::new(int))
        }"
        .to_owned()
    }

    #[must_use]
    pub fn gen_rust_profile(&self) -> String
    {
        format!(
            "{imports}\n\
            {param_decl}\n\
            {param_impls}\n\
            {struct_decl}\n\
            {plane_impl}\n\
            {other_impls}\n\
            {constructor}",
            imports = Self::imports(),
            param_decl = self.parameter_decl(),
            param_impls = self.parameter_impls(),
            struct_decl = self.user_struct_decl(),
            plane_impl = self.parameter_plane_impl(),
            other_impls = self.other_impls(),
            constructor = Self::constructor()
        )
    }

    #[must_use]
    pub fn gen_mod_rs(&self) -> String
    {
        format!(
            "pub mod {short_name};\n\
            pub use {short_name}::create_interface;",
            short_name = self.parsed_input.metadata.short_name
        )
    }

    pub fn write(&self, out_path: &Path) -> Result<(), ScriptError>
    {
        let profile_rs = self.gen_rust_profile();
        let mod_rs = self.gen_mod_rs();

        let profile_rs_path =
            out_path.join(format!("{}.rs", self.parsed_input.metadata.short_name));
        let mod_rs_path = out_path.join("mod.rs");

        println!("    Writing imports to\n        {}", mod_rs_path.display());
        std::fs::write(mod_rs_path, mod_rs).map_err(ScriptError::ErrorWritingFile)?;

        println!(
            "    Writing transpiled script to\n        {}",
            profile_rs_path.display()
        );
        std::fs::write(profile_rs_path, profile_rs).map_err(ScriptError::ErrorWritingFile)
    }
}
