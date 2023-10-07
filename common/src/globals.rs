use crate::types::Real;

pub static DISPLAY_PREC: usize = 16;

pub static RAY_DEPTH: u32 = 150;
pub static RAY_SHARPNESS: u32 = 25;

pub static NEWTON_MAX_ITERS: usize = 16;
/// Error threshold to stop Newton iteration before NEWTON_MAX_ITERS
pub static NEWTON_MIN_ERR: Real = 1e-12;
/// Error threshold beyond which a solution is rejected as invalid
pub static NEWTON_MAX_ERR: Real = 1e-5;
