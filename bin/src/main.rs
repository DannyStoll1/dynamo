extern crate fractal_lib;
use fractal_lib::gui;
use eframe;

fn main() -> Result<(), eframe::Error>
{
    gui::run_app().ok();

    Ok(())
}
