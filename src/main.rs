#![allow(dead_code)]
#![feature(const_fn_floating_point_arithmetic)]

pub mod covering_maps;
pub mod math_utils;
pub mod palette;
pub mod point_grid;
pub mod primitive_types;
pub mod profiles;
pub mod traits;
pub mod gui;
pub mod macros;
pub mod julia;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    gui::run_gui()?;

    Ok(())
}
