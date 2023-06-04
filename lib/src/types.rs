use num_complex::Complex;
use std::f64::consts::PI;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub type IterCount = f64;
pub type RealNum = f64;
pub type ComplexNum = Complex<RealNum>;
pub type Period = u32;
pub type ComplexVec = Vec<ComplexNum>;

pub mod variables;
pub use variables::{Dist, Norm};
pub mod param_stack;
pub use param_stack::{NoParam, ParamList, ParamStack};

pub const TAU: RealNum = 2. * PI;
pub const ZERO: ComplexNum = ComplexNum::new(0., 0.);
pub const ONE: ComplexNum = ComplexNum::new(1., 0.);
pub const TWO: ComplexNum = ComplexNum::new(2., 0.);
pub const TAUI: ComplexNum = ComplexNum::new(0., 2. * PI);
pub const OMEGA: ComplexNum = ComplexNum::new(-0.5, 0.866025403784439);
pub const OMEGA_BAR: ComplexNum = ComplexNum::new(-0.5, -0.866025403784439);
pub const SQRT_I: ComplexNum = ComplexNum::new(0.707106781186548, 0.707106781186548);
pub const ONE_THIRD: f64 = 1. / 3.;
pub const TWO_THIRDS: f64 = 2. / 3.;
pub const SQRT_3: f64 = 1.73205080756888;
pub const NAN: ComplexNum = ComplexNum::new(f64::NAN, 0.);

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
        final_error: RealNum,
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
        final_error: RealNum,
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
