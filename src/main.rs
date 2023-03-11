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

// use palette::ColorPalette;
//
// use primitive_types::*;
// use profiles::*;
// use traits::*;

// #[show_image::main]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    gui::run_gui()?;
    // let palette = ColorPalette::new_with_contrast(3.0, 8.0, 5.0, 0.45, 0.38);
    //
    // let period: Period = 4;
    // // let base = QuadRatPer4::new_default(2256, 2048);
    // let base = Mandelbrot::new_default(2256, 2048);
    // // let plane = base.marked_cycle_curve(period);
    // let plane = base;
    // // let plane = base.misiurewicz_curve(2, period);
    //
    // // let filename = format!("images/{}.png", base.name());
    // // let filename = format!("images/{}_MC_{}.png", base.name(), period);
    //
    // let image = plane.compute();
    //
    // image.show(palette)?;

    Ok(())
}
