use dynamo_app as app;

fn main() -> Result<(), eframe::Error>
{
    app::run_app().ok();

    Ok(())
}
