#[cfg(target_arch = "wasm32")]
use {
    eframe::{WebLogger, WebOptions, WebRunner},
    fractal_gui::FractalApp,
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

    /// Call this once from JavaScript to start your app.
    #[wasm_bindgen]
    pub async fn start(&self, canvas_id: &str) -> Result<(), wasm_bindgen::JsValue>
    {
        self.runner
            .start(
                canvas_id,
                WebOptions::default(),
                Box::new(|_cc| Box::new(FractalApp::default())),
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
