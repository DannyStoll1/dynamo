use dynamo_common::types::Period;
use lazy_static::lazy_static;
use num_complex::Complex64;
use pyo3::{Python, ToPyObject};
use regex::Regex;
use serde::Deserialize;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::str::FromStr;

mod defaults;

use crate::error::ScriptError;

#[derive(Debug, Deserialize)]
pub struct Metadata
{
    pub name: String,
    pub short_name: String,
}

#[derive(Debug, Deserialize)]
pub struct Names
{
    pub variable: String,
    pub selection: String,
}

#[derive(Debug, Deserialize)]
pub struct Functions
{
    pub map: JsonValue,
    pub start: JsonValue,
}

#[derive(Debug, Deserialize)]
pub struct EscapingReturnMapParams
{
    #[serde(default = "defaults::degree")]
    pub degree: i64,
    #[serde(default = "defaults::escaping_period")]
    pub escaping_period: Period,
    #[serde(default = "defaults::escaping_phase")]
    pub escaping_phase: Period,
}

impl Default for EscapingReturnMapParams
{
    fn default() -> Self
    {
        Self {
            degree: defaults::degree(),
            escaping_period: defaults::escaping_period(),
            escaping_phase: defaults::escaping_phase(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UnparsedUserInput
{
    pub metadata: Metadata,
    pub constants: HashMap<String, JsonValue>,
    pub parameters: HashMap<String, String>,
    pub dynamics: Functions,
    pub names: Names,
    pub optional: Option<EscapingReturnMapParams>,
}

#[derive(Clone, Debug)]
pub struct PyParams
{
    pub param_map: String,
    pub map: String,
    pub map_d: String,
    pub start: String,
    pub start_d: String,
}

pub struct ParsedUserInput
{
    pub metadata: Metadata,
    pub constants: HashMap<String, Complex64>,
    pub param_names: Vec<String>,
    pub names: Names,
    pub optional: EscapingReturnMapParams,
    pub py_params: PyParams,
}
impl TryFrom<UnparsedUserInput> for ParsedUserInput
{
    type Error = ScriptError;
    fn try_from(unparsed: UnparsedUserInput) -> Result<Self, Self::Error>
    {
        unparsed.parse()
    }
}

fn json_to_string(value: &JsonValue) -> String
{
    match value {
        JsonValue::String(s) => s.clone(),
        JsonValue::Number(n) => n.to_string(),
        JsonValue::Bool(b) => b.to_string(),
        _ => panic!("Unsupported JsonValue type"),
    }
}

pub(crate) fn json_to_complex(value: &JsonValue) -> Result<Complex64, ScriptError>
{
    match value {
        JsonValue::String(s) => {
            lazy_static! {
                static ref A_PLUS_BI: Regex =
                    Regex::new(r"(-?[0-9]+\.?[0-9]*)?\s*\+\s*(-?[0-9]+\.?[0-9]*)?i")
                        .expect("Invalid regex");
                static ref A_MINUS_BI: Regex =
                    Regex::new(r"(-?[0-9]+\.?[0-9]*)?\s*-\s*(-?[0-9]+\.?[0-9]*)?i")
                        .expect("Invalid regex");
                static ref BI: Regex = Regex::new(r"(-?[0-9]+\.?[0-9]*)?i").expect("Invalid regex");
            }

            // Handle real numbers
            if let Ok(real) = f64::from_str(s) {
                return Ok(Complex64::new(real, 0.0));
            }

            // Handle numbers expressed in the form "a+bi"
            if let Some(caps) = A_PLUS_BI.captures(s) {
                let a = caps
                    .get(1)
                    .map_or(0.0, |m| f64::from_str(m.as_str()).unwrap_or(0.0));
                let b = caps
                    .get(2)
                    .map_or(1.0, |m| f64::from_str(m.as_str()).unwrap_or(1.0));
                return Ok(Complex64::new(a, b));
            }

            // Handle numbers expressed in the form "a-bi"
            if let Some(caps) = A_MINUS_BI.captures(s) {
                let a = caps
                    .get(1)
                    .map_or(0.0, |m| f64::from_str(m.as_str()).unwrap_or(0.0));
                let b = -caps
                    .get(2)
                    .map_or(1.0, |m| f64::from_str(m.as_str()).unwrap_or(1.0));
                return Ok(Complex64::new(a, b));
            }

            // Handle numbers expressed in the form "bi"
            if let Some(caps) = BI.captures(s) {
                let b = caps
                    .get(1)
                    .map_or(0.0, |m| f64::from_str(m.as_str()).unwrap_or(0.0));
                return Ok(Complex64::new(0.0, b));
            }
            Err(ScriptError::MalformedConst)
        }
        JsonValue::Number(n) => {
            let real_part = n.as_f64().ok_or(ScriptError::MalformedConst)?;
            Ok(Complex64::new(real_part, 0.0))
        }
        _ => Err(ScriptError::MalformedConst),
    }
}

impl UnparsedUserInput
{
    pub fn parse(self) -> Result<ParsedUserInput, ScriptError>
    {
        let const_names: Vec<String> = self.constants.keys().cloned().collect();
        let param_names: Vec<String> = self.parameters.keys().cloned().collect();
        let constants: HashMap<String, Complex64> = self
            .constants
            .iter()
            .map(|(key, value)| json_to_complex(value).map(|complex| (key.clone(), complex)))
            .filter_map(Result::ok)
            .collect::<HashMap<String, Complex64>>();

        let py_params = Python::with_gil(|py| {
            let sys = py.import("sys")?;
            sys.getattr("path")?.call_method1("append", ("python",))?;
            sys.getattr("path")?
                .call_method1("append", ("scripting/loader/python",))?;

            // Convert to python types
            let map_str = &json_to_string(&self.dynamics.map).to_object(py);
            let start_str = &json_to_string(&self.dynamics.start).to_object(py);
            let z_str = self.names.variable.to_object(py);
            let t_str = self.names.selection.to_object(py);

            let param_names_py = param_names.to_object(py);
            let const_names_py = const_names.to_object(py);

            // Imports
            let sympy = py.import("sympy")?;
            let parse_expr = sympy.getattr("parse_expr")?;
            let symbols = sympy.getattr("symbols")?;
            let cse = sympy.getattr("cse")?;

            let oxidize = py.import("oxidize")?;
            let oxidize_expr = oxidize.getattr("oxidize_expr")?;
            let oxidize_cse = oxidize.getattr("oxidize_cse")?;
            let oxidize_pmap = oxidize.getattr("oxidize_param_map_cplx")?;

            // Symbol declarations
            symbols.call1(((&z_str, &t_str),))?;
            symbols.call1((&param_names_py,))?;
            symbols.call1((&const_names_py,))?;

            // Parsing
            let mut params_dict_py = HashMap::new();

            self.parameters.iter().try_for_each(|(name, val)| {
                let parsed_val = parse_expr.call1((val,))?;
                params_dict_py.insert(name, parsed_val);
                Ok::<_, ScriptError>(())
            })?;
            let params_dict_py = params_dict_py.to_object(py);

            let map_py = parse_expr.call1((map_str,))?;
            let map_d_py = map_py.call_method1("diff", (z_str,))?;
            let map_cse_py = cse.call1(([map_py, map_d_py],))?;
            let map = oxidize_expr.call1((map_py,))?.to_string();
            let map_d = oxidize_cse.call1((map_cse_py,))?.to_string();

            let start_py = parse_expr.call1((start_str,))?;
            let start_py = start_py.call_method1("subs", (&params_dict_py,))?;
            let start_d_py = start_py.call_method1("diff", (t_str,))?;
            let start_cse_py = cse.call1(([start_py, start_d_py],))?;
            let start = oxidize_expr.call1((start_py,))?.to_string();
            let start_d = oxidize_cse.call1((start_cse_py,))?.to_string();

            let param_map = oxidize_pmap.call1((params_dict_py,))?.to_string();

            let py_params = PyParams {
                param_map,
                map,
                map_d,
                start,
                start_d,
            };

            Ok::<_, ScriptError>(py_params)
        })?;

        Ok(ParsedUserInput {
            metadata: self.metadata,
            constants,
            param_names,
            names: self.names,
            optional: self.optional.unwrap_or_default(),
            py_params,
        })
    }
}
