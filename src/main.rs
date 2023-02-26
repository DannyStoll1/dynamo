pub mod palette;
pub mod point_grid;
pub mod primitive_types;
pub mod traits;
pub mod profiles;
pub mod covering_maps;

use profiles::*;
use palette::ColorPalette;
use traits::{FractalImage, ParameterPlane};

fn main() {
    let palette = ColorPalette::new(8.0, 3.0, 5.0);

    // let base = QuadRatPer2::new_default(1400, 1024);

    // let plane = base.marked_cycle(5);
    let plane = BurningShip::new_default(1400, 1024);
    let filename = "images/burning_ship.png";

    let image = plane.compute();
    image.draw(palette, filename);
}
