use fractal_gui as gui;

fn main() -> Result<(), eframe::Error>
{
    gui::run_app().ok();

    Ok(())
}
