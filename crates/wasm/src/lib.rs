#[cfg(target_arch = "wasm32")]
use web_sys::HtmlCanvasElement;
#[cfg(target_arch = "wasm32")]
use {
    dynamo_app::FractalApp,
    eframe::{WebLogger, WebOptions, WebRunner},
    log,
    wasm_bindgen::prelude::*,
};

#[cfg(target_arch = "wasm32")]
#[derive(Clone)]
#[wasm_bindgen]
pub struct WebHandle
{
    runner: WebRunner,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WebHandle
{
    /// Installs a panic hook, then returns.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self
    {
        // Redirect [`log`] message to `console.log` and friends:
        WebLogger::init(log::LevelFilter::Debug).ok();

        Self {
            runner: WebRunner::new(),
        }
    }

    /// Call this once from JavaScript to start the app.
    #[wasm_bindgen]
    pub async fn start(&self, canvas_id: HtmlCanvasElement) -> Result<(), wasm_bindgen::JsValue>
    {
        let opts = WebOptions::default();
        self.runner
            .start(
                canvas_id,
                opts,
                Box::new(|_cc| Ok(Box::new(FractalApp::default()))),
            )
            .await
    }
}

#[cfg(target_arch = "wasm32")]
impl Default for WebHandle
{
    fn default() -> Self
    {
        Self::new()
    }
}
