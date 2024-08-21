pub mod exponential;
pub use exponential::Exponential;

pub mod cosine;
pub use cosine::{CoshNewton, Cosine, CosineAdd, SineWander};

pub mod zeta;
pub use zeta::{RiemannXi, RiemannXiNewton};

pub mod gudermannian;
pub use gudermannian::Gudermannian;
