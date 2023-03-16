use num_complex::Complex;

pub type IterCount = f32;
pub type RealNum = f64;
pub type ComplexNum = Complex<RealNum>;
pub type Period = u32;

#[derive(Clone, Debug)]
pub enum EscapeState {
    Escaped{ iters: Period, final_value: ComplexNum},
    Periodic{ preperiod: Period, period: Period},
    NotYetEscaped,
    Bounded,
}
