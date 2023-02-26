use num_complex::Complex;

pub type ComplexNum = Complex<f64>;
pub type Period = u32;

pub enum EscapeState {
    Escaped(ComplexNum),
    NotYetEscaped,
    Bounded,
}
