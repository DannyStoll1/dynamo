use crate::types::{ComplexNum, RealNum};
pub use std::f64::consts::{PI, TAU};

pub const ZERO: ComplexNum = ComplexNum::new(0., 0.);
pub const ONE: ComplexNum = ComplexNum::new(1., 0.);
pub const TWO: ComplexNum = ComplexNum::new(2., 0.);
pub const TAUI: ComplexNum = ComplexNum::new(0., 2. * PI);
pub const OMEGA: ComplexNum = ComplexNum::new(-0.5, 0.866025403784439);
pub const OMEGA_BAR: ComplexNum = ComplexNum::new(-0.5, -0.866025403784439);
pub const ONE_THIRD: f64 = 1. / 3.;
pub const TWO_THIRDS: f64 = 2. / 3.;
pub const SQRT_3: f64 = 1.73205080756888;
pub const ZETA_5_1: ComplexNum = ComplexNum::new(0.309016994374947, 0.951056516295154);
pub const ZETA_5_2: ComplexNum = ComplexNum::new(-0.809016994374947, 0.587785252292473);
pub const NAN: ComplexNum = ComplexNum::new(f64::NAN, 0.);
