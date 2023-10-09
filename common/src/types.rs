use num_complex::Complex;
use num_rational::Ratio;

pub mod variables;
pub use variables::*;
pub mod param_stack;
pub use param_stack::{NoParam, ParamList, ParamStack};

pub type IterCount = f64;
pub type Real = f64;
pub type Cplx = Complex<Real>;
pub type ComplexVec = Vec<Cplx>;
pub type Period = u32;
pub type SignedPeriod = i32;
pub type AngleNum = i64;
pub type Rational = Ratio<AngleNum>;
