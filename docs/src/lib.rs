use eframe::{WebOptions, start_web};
use fractal_lib::gui::FractalApp;
use wasm_bindgen::{prelude::*, JsValue};

#[wasm_bindgen]
pub async fn run_app(canvas_id: &str) -> Result<(), JsValue>
{
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    let web_options = WebOptions::default();

    start_web(
        canvas_id,
        web_options,
        Box::new(|_cc| Box::new(FractalApp::default())),
    )
    .await?;
    Ok(())
}
