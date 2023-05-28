use eframe::{WebRunner, WebOptions, WebLogger};
use log;
use fractal_lib::gui::FractalApp;
use wasm_bindgen::prelude::*;

#[derive(Clone)]
#[wasm_bindgen]
pub struct WebHandle {
    runner: WebRunner,
}

#[wasm_bindgen]
impl WebHandle {
    /// Installs a panic hook, then returns.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        // Redirect [`log`] message to `console.log` and friends:
        WebLogger::init(log::LevelFilter::Debug).ok();

        Self {
            runner: WebRunner::new(),
        }
    }

    /// Call this once from JavaScript to start your app.
    #[wasm_bindgen]
    pub async fn start(&self, canvas_id: &str) -> Result<(), wasm_bindgen::JsValue> {
        self.runner
            .start(
                canvas_id,
                WebOptions::default(),
                Box::new(|cc| Box::new(FractalApp::default())),
            )
            .await
    }
}

impl Default for WebHandle {
    fn default() -> Self {
        Self::new()
    }
}
