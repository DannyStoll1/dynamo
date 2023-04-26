use num_complex::Complex;
use std::f64::consts::PI;

pub type IterCount = f64;
pub type RealNum = f64;
pub type ComplexNum = Complex<RealNum>;
pub type Period = u32;
pub type ComplexVec = Vec<ComplexNum>;

pub const TAU: RealNum = 2. * PI;
pub const TWO: RealNum = 2.;
pub const ONE_COMPLEX: ComplexNum = ComplexNum::new(1., 0.);
pub const TAUI: ComplexNum = ComplexNum::new(0., 2. * PI);

#[derive(Clone, Copy, Debug)]
pub enum EscapeState
{
    Escaped
    {
        iters: Period,
        final_value: ComplexNum,
    },
    Periodic
    {
        preperiod: Period,
        period: Period,
        multiplier: ComplexNum,
        final_error: ComplexNum,
    },
    NotYetEscaped,
    Bounded,
}

#[derive(Clone, Copy, Debug)]
pub enum PointInfo
{
    Escaping
    {
        potential: IterCount,
    },
    Periodic
    {
        preperiod: Period,
        period: Period,
        multiplier: ComplexNum,
        final_error: ComplexNum,
    },
    Bounded,
}
