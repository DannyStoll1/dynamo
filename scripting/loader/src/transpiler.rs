use crate::{error::ScriptError, parser::*};
use std::path::Path;

pub struct Transpiler
{
    pub parsed_input: ParsedUserInput,
}

impl Transpiler
{
    #[must_use]
    pub fn new(unparsed_input: UnparsedUserInput) -> Self
    {
        let parsed_input = unparsed_input.parse().unwrap();
        Self { parsed_input }
    }

    #[must_use]
    pub fn from_toml_path(path: &Path) -> Result<Self, ScriptError>
    {
        let content = std::fs::read_to_string(path).map_err(ScriptError::ErrorReadingToml)?;
        let user_input: UnparsedUserInput =
            toml::from_str(&content).map_err(ScriptError::ErrorParsingToml)?;
        Ok(Self::new(user_input))
    }

    fn parameter_decl(&self) -> String
    {
        if self.parsed_input.param_names.is_empty()
        {
            return "type Parameters = NoParam;".to_owned();
        }

        let member_decls: Vec<String> = self
            .parsed_input
            .param_names
            .iter()
            .map(|name| format!("{}: Cplx,", name)) // adjust the format as needed
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
        if self.parsed_input.param_names.is_empty()
        {
            return "".to_owned();
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
            fn describe(&self) -> Option<String> {{
                Some(self.to_string())
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
        {param_map}
    }}

    fn escape_radius(&self) -> Real
    {{
        1e26
    }}

    fn start_point(&self, {t}: Cplx, {c}: Self::Param) -> Self::Var
    {{
        {start}
    }}

    fn start_point_d(&self, {t}: Cplx, {c}: Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {{
        let (z0, dz0_dt) = {{
            {start_d}
        }};
        (z0, dz0_dt, ZERO)
    }}

    fn map(&self, {z}: Self::Var, {c}: Self::Param) -> Self::Var
    {{
        {map}
    }}

    fn map_and_multiplier(&self, {z}: Self::Var, {c}: Self::Param) -> (Self::Var, Self::Deriv)
    {{
        {map_d}
    }}

    fn dynamical_derivative(&self, {z}: Self::Var, param: Self::Param) -> Self::Deriv
    {{
        self.map_and_multiplier(z, param).1
    }}

    fn parameter_derivative(&self, {z}: Self::Var, param: Self::Param) -> Self::Deriv
    {{
        ONE
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
            param_map = self.parsed_input.context.get::<String>("param_map_rs"),
            map = self.parsed_input.context.get::<String>("f_rs"),
            map_d = self.parsed_input.context.get::<String>("f_df_rs"),
            start = self.parsed_input.context.get::<String>("z0_rs"),
            start_d = self.parsed_input.context.get::<String>("z0_dz0_rs"),
            name = self.parsed_input.metadata.name,
        )
    }
    fn imports(&self) -> String
    {
        "use fractal_common::prelude::*;\n\
        use fractal_core::prelude::*;\n\
        use fractal_gui::interface::{MainInterface, Interface};\n\
        "
        .to_owned()
    }

    fn constructor(&self) -> String
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

    pub fn gen_rust_profile(&self) -> String
    {
        format!(
            "{imports}\n\
            {param_decl}\n\
            {param_impls}\n\
            {struct_decl}\n\
            {plane_impl}\n\
            {constructor}",
            imports = self.imports(),
            param_decl = self.parameter_decl(),
            param_impls = self.parameter_impls(),
            struct_decl = self.user_struct_decl(),
            plane_impl = self.parameter_plane_impl(),
            constructor = self.constructor()
        )
    }

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

        std::fs::write(mod_rs_path, mod_rs).map_err(ScriptError::ErrorWritingFile)?;
        std::fs::write(profile_rs_path, profile_rs).map_err(ScriptError::ErrorWritingFile)
    }
}