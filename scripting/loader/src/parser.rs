use dynamo_common::types::Period;
use inline_python::{python, Context};
use lazy_static::lazy_static;
use num_complex::Complex64;
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

pub struct ParsedUserInput
{
    pub metadata: Metadata,
    pub constants: HashMap<String, Complex64>,
    pub param_names: Vec<String>,
    pub names: Names,
    pub optional: EscapingReturnMapParams,
    pub context: Context,
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
    match value
    {
        JsonValue::String(s) => s.clone(),
        JsonValue::Number(n) => n.to_string(),
        JsonValue::Bool(b) => b.to_string(),
        _ => panic!("Unsupported JsonValue type"),
    }
}

pub(crate) fn json_to_complex(value: &JsonValue) -> Result<Complex64, ScriptError>
{
    match value
    {
        JsonValue::String(s) =>
        {
            lazy_static! {
                static ref A_PLUS_BI: Regex =
                    Regex::new(r"(-?[0-9]+\.?[0-9]*)?\s*\+\s*(-?[0-9]+\.?[0-9]*)?i").unwrap();
                static ref A_MINUS_BI: Regex =
                    Regex::new(r"(-?[0-9]+\.?[0-9]*)?\s*-\s*(-?[0-9]+\.?[0-9]*)?i").unwrap();
                static ref BI: Regex = Regex::new(r"(-?[0-9]+\.?[0-9]*)?i").unwrap();
            }

            // Handle real numbers
            if let Ok(real) = f64::from_str(s)
            {
                return Ok(Complex64::new(real, 0.0));
            }

            // Handle numbers expressed in the form "a+bi"
            if let Some(caps) = A_PLUS_BI.captures(s)
            {
                let a = caps
                    .get(1)
                    .map_or(0.0, |m| f64::from_str(m.as_str()).unwrap_or(0.0));
                let b = caps
                    .get(2)
                    .map_or(1.0, |m| f64::from_str(m.as_str()).unwrap_or(1.0));
                return Ok(Complex64::new(a, b));
            }

            // Handle numbers expressed in the form "a-bi"
            if let Some(caps) = A_MINUS_BI.captures(s)
            {
                let a = caps
                    .get(1)
                    .map_or(0.0, |m| f64::from_str(m.as_str()).unwrap_or(0.0));
                let b = -caps
                    .get(2)
                    .map_or(1.0, |m| f64::from_str(m.as_str()).unwrap_or(1.0));
                return Ok(Complex64::new(a, b));
            }

            // Handle numbers expressed in the form "bi"
            if let Some(caps) = BI.captures(s)
            {
                let b = caps
                    .get(1)
                    .map_or(0.0, |m| f64::from_str(m.as_str()).unwrap_or(0.0));
                return Ok(Complex64::new(0.0, b));
            }
            Err(ScriptError::MalformedConst)
        }
        JsonValue::Number(n) =>
        {
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
        let z = &self.names.variable;
        let t = &self.names.selection;

        let const_names: Vec<String> = self.constants.keys().cloned().collect();
        let constants: HashMap<String, Complex64> = self
            .constants
            .iter()
            .map(|(key, value)| json_to_complex(value).map(|complex| (key.clone(), complex)))
            .filter_map(Result::ok)
            .collect::<HashMap<String, Complex64>>();

        let param_names: Vec<String> = self.parameters.keys().cloned().collect();
        let params = &self.parameters;

        let map_str = &json_to_string(&self.dynamics.map);
        let start_str = &json_to_string(&self.dynamics.start);

        let py_constants = &constants;
        let py_param_names = &param_names;

        let context: Context = python! {
            from sympy import symbols, lambdify, parse_expr, cse

            import os, sys
            sys.path.append("python")
            sys.path.append(os.path.join("scripting", "loader", "python"))
            from oxidize import *

            i = 1j
            [z, t] = symbols(['z, 't])
            const_names = symbols('const_names)
            param_names = symbols('py_param_names)
            param_dict = {
                name: parse_expr(val) for (name, val) in 'params.items()
            }

            consts = 'py_constants
            z0 = parse_expr('start_str)
            z0_t = z0.subs(param_dict)
            z0_dz0 = cse([z0, z0.subs(param_dict).diff(t)], optimizations="basic")

            f = parse_expr('map_str)
            f_df = cse([f, f.diff(z)], optimizations="basic")

            z0_rs = oxidize_expr_cplx(z0)
            f_rs = oxidize_expr_cplx(f)
            z0_dz0_rs = oxidize_cse_cplx(z0_dz0)
            f_df_rs = oxidize_cse_cplx(f_df)
            param_map_rs = oxidize_param_map_cplx(param_dict)
        };

        Ok(ParsedUserInput {
            metadata: self.metadata,
            constants,
            param_names,
            names: self.names,
            context,
            optional: self.optional.unwrap_or_default(),
        })
    }
}

// let context = python! {
//     from sympy import symbols, lambdify, parse_expr
//
//     parameters = 'parameters
//     variables = 'variables
//     all_symbols = symbols(parameters + variables)
//     dynamics_expr = parse_expr('dynamics_str)
//     print(dynamics_expr)
//
//     f = lambdify(all_symbols, dynamics_expr, "numpy")
// };
