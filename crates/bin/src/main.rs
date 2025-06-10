use dynamo_app as app;

fn main() -> Result<(), eframe::Error>
{
    pretty_env_logger::init();
    app::run_app()
}
