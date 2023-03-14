use num_complex::Complex;

pub type IterCount = f32;
pub type Float = f64;
pub type ComplexNum = Complex<Float>;
pub type Period = u32;

pub enum EscapeState {
    Escaped{ iters: Period, final_value: ComplexNum},
    Periodic{ preperiod: Period, period: Period},
    NotYetEscaped,
    Bounded,
}
