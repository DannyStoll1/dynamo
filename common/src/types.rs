use self::param_stack::Summarize;
use derive_more::{Add, Display, From};
use num_complex::Complex;
use num_rational::Ratio;
use std::fmt::Display;

pub mod variables;
pub use variables::{Dist, Norm, Polar, MaybeNan};
pub mod param_stack;
pub use param_stack::{NoParam, ParamList, ParamStack};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub type IterCount = f64;
pub type Real = f64;
pub type Cplx = Complex<Real>;
pub type ComplexVec = Vec<Cplx>;
pub type Period = u32;
pub type AngleNum = i64;
pub type Rational = Ratio<AngleNum>;

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PointInfoPeriodic<V, D>
{
    pub value: V,
    pub preperiod: Period,
    pub period: Period,
    pub multiplier: D,
    pub final_error: Real,
}
impl<V, D> ToString for PointInfoPeriodic<V, D>
where
    V: Display,
    D: Display,
{
    fn to_string(&self) -> String
    {
        format!(
            "Cycle detected after {preperiod} iterations.\n\
                Period: {period}\n\
                Multiplier: {multiplier:.DISPLAY_PREC$}",
            preperiod = self.preperiod,
            period = self.period,
            multiplier = self.multiplier
        )
    }
}

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
        data: PointInfoPeriodic<V, D>,
    },
    NotYetEscaped,
    Bounded,
}
impl<V, D> Default for EscapeState<V, D>
{
    fn default() -> Self
    {
        Self::NotYetEscaped
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PointClassId(pub u8);

impl From<usize> for PointClassId
{
    fn from(n: usize) -> Self {
        Self(n as u8)
    }
}

impl From<PointClassId> for f32
{
    fn from(id: PointClassId) -> Self
    {
        id.0 as f32
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PointInfo<V, D>
{
    Escaping
    {
        potential: IterCount,
    },
    Periodic
    {
        data: PointInfoPeriodic<V, D>,
    },
    Bounded,
    Wandering,
    MarkedPoint
    {
        data: PointInfoPeriodic<V, D>,
        class_id: PointClassId,
        num_point_classes: usize,
    },
}
impl<V, D> Default for PointInfo<V, D>
{
    fn default() -> Self
    {
        Self::Bounded
    }
}

const DISPLAY_PREC: usize = 16;

pub fn format_complex(value: Cplx) -> String
{
    format!("{:.*}", DISPLAY_PREC, value)
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct OrbitInfo<V, P, D>
{
    pub start: V,
    pub param: P,
    pub result: PointInfo<V, D>,
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
            Escaping { potential } => format!("Escaped, potential: {potential:.DISPLAY_PREC$}"),
            Periodic { data } => data.to_string(),
            Bounded => "Bounded (no cycle detected or period too high)".to_owned(),
            Wandering => "Wandering (appears to escape very slowly)".to_owned(),
            MarkedPoint { data, .. } => data.to_string(),
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
pub struct Pair<T>
where
    T: std::fmt::Display,
{
    pub a: T,
    pub b: T,
}

impl<T> Summarize for Pair<T> where T: std::fmt::Display {}

impl<T> From<Cplx> for Pair<T>
where
    T: std::fmt::Display,
{
    fn from(_z: Cplx) -> Self
    {
        unimplemented!()
    }
}

impl<T> From<Pair<T>> for Cplx
where
    T: std::fmt::Display,
{
    fn from(_value: Pair<T>) -> Self
    {
        unimplemented!()
    }
}

pub type RealPair = Pair<Real>;
pub type CplxPair = Pair<Cplx>;

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
