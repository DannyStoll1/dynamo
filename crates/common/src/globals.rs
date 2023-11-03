use crate::types::Real;

pub const RAY_DEPTH: u32 = 200;
pub const RAY_SHARPNESS: u32 = 25;

pub const NEWTON_MAX_ITERS: usize = 16;
/// Error threshold to stop Newton iteration before `NEWTON_MAX_ITERS`
pub const NEWTON_MIN_ERR: Real = 1e-12;
/// Error threshold beyond which a solution is rejected as invalid
pub const NEWTON_MAX_ERR: Real = 1e-5;

pub const DISPLAY_PREC: usize = 12;

pub const IMAGE_HEIGHT: usize = 768;

pub const WIN_HEIGHT: f32 = (IMAGE_HEIGHT + 192) as f32;
pub const WIN_WIDTH: f32 = (IMAGE_HEIGHT * 2 + 100) as f32;
