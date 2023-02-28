pub mod covering_maps;
pub mod math_utils;
pub mod palette;
pub mod point_grid;
pub mod primitive_types;
pub mod profiles;
pub mod traits;
pub mod gui;

use palette::ColorPalette;
use point_grid::PointGrid;
use primitive_types::*;
use profiles::*;
use traits::{FractalImage, HasDynamicalCovers, ParameterPlane};

fn main() {
    let palette = ColorPalette::new_with_contrast(3.0, 8.0, 5.0, 0.45, 0.38);

    let period: Period = 3;
    // let base = QuadRatPer4::new_default(2256, 2048);
    let base = QuadRatPer4::new_default(1024, 16384);
    let plane = base.marked_cycle_curve(period);
    // let plane = base.misiurewicz_curve(2, period);

    // let filename = format!("images/{}_Misiurewicz_2_{}.png", base.name(), period);
    // let filename = format!("images/{}.png", base.name());
    let filename = format!("images/{}_MC_{}.png", base.name(), period);

    let image = plane.compute();

    image.draw(palette, filename);
}
