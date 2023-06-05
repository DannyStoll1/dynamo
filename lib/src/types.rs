use num_complex::Complex;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub(crate) type IterCount = f64;
pub(crate) type Real = f64;
pub(crate) type Cplx = Complex<Real>;
pub(crate) type Period = u32;
pub(crate) type ComplexVec = Vec<Cplx>;

pub mod variables;
pub use variables::{Dist, Norm};
pub mod param_stack;
pub use param_stack::{NoParam, ParamList, ParamStack};

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum EscapeState<V, D>
{
    Escaped
    {
        iters: Period,
        final_value: V,
    },
    Periodic
    {
        preperiod: Period,
        period: Period,
        multiplier: D,
        final_error: Real,
    },
    NotYetEscaped,
    Bounded,
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PointInfo<D>
{
    Escaping
    {
        potential: IterCount,
    },
    Periodic
    {
        preperiod: Period,
        period: Period,
        multiplier: D,
        final_error: Real,
    },
    Bounded,
    Wandering,
}

use std::fmt::Display;

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct OrbitInfo<V, P, D>
{
    pub start: V,
    pub param: P,
    pub result: PointInfo<D>,
}
impl<V, P, D> ToString for OrbitInfo<V, P, D>
where
    V: Display,
    P: Display,
    D: Display,
{
    #[must_use]
    fn to_string(&self) -> String
    {
        use PointInfo::*;
        let result_summary = match &self.result
        {
            Escaping { potential } => format!("Escaped, potential: {potential}"),
            Periodic {
                period,
                preperiod,
                multiplier,
                final_error: _,
            } => format!(
                "Cycle detected after {preperiod} iterations.\n    Period: {period}\n    Multiplier: {multiplier}"
            ),
            Bounded => "Bounded (no cycle detected or period too high)".to_owned(),
            Wandering => "Wandering (appears to escape very slowly)".to_owned(),
        };
        format!(
            "Parameter: {}\nStarting point: {}\n{}",
            self.param, self.start, result_summary
        )
    }
}
