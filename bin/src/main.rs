extern crate fractal_lib;
use fractal_lib::gui;

fn main() -> Result<(), eframe::Error>
{
    gui::run_app().ok();

    Ok(())
}
