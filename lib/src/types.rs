use derive_more::{Add, Display, From};
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

use self::param_stack::Summarize;

const DISPLAY_PREC: usize = 16;

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
        use PointInfo::{Bounded, Escaping, Periodic, Wandering};
        let result_summary = match &self.result
        {
            Escaping { potential } => format!("Escaped, potential: {potential:.DISPLAY_PREC$}"),
            Periodic {
                period,
                preperiod,
                multiplier,
                final_error: _,
            } => format!(
                "Cycle detected after {preperiod} iterations.\n    Period: {period}\n    Multiplier: {multiplier:.DISPLAY_PREC$}"
            ),
            Bounded => "Bounded (no cycle detected or period too high)".to_owned(),
            Wandering => "Wandering (appears to escape very slowly)".to_owned(),
        };
        format!(
            "Parameter: {:.*}\nStarting point: {:.*}\n{}",
            DISPLAY_PREC, self.param, DISPLAY_PREC, self.start, result_summary
        )
    }
}

#[derive(Default, Clone, Copy, Debug, Add, From, PartialEq, Display)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[display(fmt = "[ a: {a}, b: {b} ] ")]
pub struct CplxPair
{
    pub a: Cplx,
    pub b: Cplx,
}

impl Summarize for CplxPair {}

impl From<Cplx> for CplxPair
{
    fn from(_z: Cplx) -> Self
    {
        unimplemented!()
    }
}

impl From<CplxPair> for Cplx
{
    fn from(_value: CplxPair) -> Self
    {
        unimplemented!()
    }
}

#[derive(Default, Clone, Copy, Debug, Add, From, PartialEq, Display)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[display(fmt = "[ a: {a}, b: {b}, c: {c}, d: {d} ] ")]
pub struct ComplexQuad
{
    pub a: Cplx,
    pub b: Cplx,
    pub c: Cplx,
    pub d: Cplx,
}

impl Summarize for ComplexQuad {}

impl From<Cplx> for ComplexQuad
{
    fn from(_z: Cplx) -> Self
    {
        unimplemented!()
    }
}

impl From<ComplexQuad> for Cplx
{
    fn from(_value: ComplexQuad) -> Self
    {
        unimplemented!()
    }
}
