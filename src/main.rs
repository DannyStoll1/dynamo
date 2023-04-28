#![allow(dead_code)]
#![feature(const_fn_floating_point_arithmetic)]

pub mod math_utils;
pub mod point_grid;
pub mod types;
pub mod profiles;
pub mod dynamics;
pub mod gui;
pub mod macros;
pub mod iter_plane;
pub mod coloring;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    gui::run_app()?;

    Ok(())
}
