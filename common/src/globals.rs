use crate::types::Real;

pub const DISPLAY_PREC: usize = 12;

pub const RAY_DEPTH: u32 = 200;
pub const RAY_SHARPNESS: u32 = 25;

pub const NEWTON_MAX_ITERS: usize = 16;
/// Error threshold to stop Newton iteration before `NEWTON_MAX_ITERS`
pub const NEWTON_MIN_ERR: Real = 1e-12;
/// Error threshold beyond which a solution is rejected as invalid
pub const NEWTON_MAX_ERR: Real = 1e-5;
