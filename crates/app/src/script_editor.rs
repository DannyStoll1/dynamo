use std::path::{Path, PathBuf};

use dynamo_common::directories::script_dir;
use script_loader::error::ScriptError;
use script_loader::parser::UnparsedUserInput;

pub(super) mod config;
use config::SCRIPT_PROJ_DIR;

pub mod popup;
pub use popup::*;

#[derive(Clone, Debug, Default)]
pub enum Response
{
    #[default]
    DoNothing,
    Close,
    Load(PathBuf),
}

#[derive(Clone, Debug, Default)]
pub struct ScriptDocument
{
    pub path: Option<PathBuf>,
    pub text: String,
}

#[derive(Clone, Debug, Default)]
pub enum ValidationState
{
    #[default]
    Unknown,
    Valid,
    Invalid(String),
}

#[derive(Clone, Debug, Default)]
pub enum RunState
{
    #[default]
    Idle,
    Pending(PathBuf),
}

#[derive(Clone, Debug)]
pub struct ScriptEditor
{
    pub document:   ScriptDocument,
    pub validation: ValidationState,
    pub run_state:  RunState,
    pub visible:    bool,
}
impl Default for ScriptEditor
{
    fn default() -> Self
    {
        Self {
            document: ScriptDocument {
                path: None,
                text: config::DEFAULT_TEXT.clone(),
            },
            validation: ValidationState::Unknown,
            run_state: RunState::Idle,
            visible: true,
        }
    }
}
impl ScriptEditor
{
    #[must_use]
    pub fn load<P>(script_file: P) -> std::io::Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = script_file.as_ref().to_path_buf();
        let text = std::fs::read_to_string(&path)?;
        Ok(Self {
            document: ScriptDocument {
                path: Some(path),
                text,
            },
            ..Default::default()
        })
    }

    pub fn show(&mut self, ctx: &egui::Context)
    {
        if self.visible {
            egui::Window::new("Script Editor")
                .vscroll(true)
                .default_height(720.)
                .default_width(600.)
                .show(ctx, |ui| {
                    egui::TextEdit::multiline(&mut self.document.text)
                        .code_editor()
                        .desired_rows(30)
                        .desired_width(std::f32::INFINITY)
                        .show(ui);
                    self.show_validation(ui);
                    if ui.button("Save").clicked() {
                        self.try_save(false);
                    }
                    if ui.button("Save and Run").clicked() {
                        self.try_save(true);
                    }
                    if ui.button("Cancel").clicked() {
                        self.hide();
                    }
                });
        }
    }

    #[inline]
    pub fn open(&mut self)
    {
        self.visible = true;
    }

    #[inline]
    pub fn enabled(&self) -> bool
    {
        self.visible
    }

    #[inline]
    pub fn hide(&mut self)
    {
        self.visible = false;
    }

    fn show_validation(&self, ui: &mut egui::Ui)
    {
        match &self.validation {
            ValidationState::Unknown => {}
            ValidationState::Valid => {
                ui.colored_label(egui::Color32::LIGHT_GREEN, "Script validation succeeded.");
            }
            ValidationState::Invalid(message) => {
                ui.colored_label(egui::Color32::LIGHT_RED, message);
            }
        }
    }

    fn try_save(&mut self, run: bool)
    {
        match self.save_script() {
            Ok(script_path) => {
                self.validation = ValidationState::Valid;
                if run {
                    self.run_state = RunState::Pending(script_path);
                } else {
                    self.hide();
                }
            }
            Err(e) => {
                self.validation = ValidationState::Invalid(e.to_string());
            }
        }
    }

    fn save_script(&mut self) -> Result<PathBuf, ScriptError>
    {
        let script_data: UnparsedUserInput =
            toml::from_str(&self.document.text).map_err(ScriptError::ErrorParsingToml)?;
        let filename = format!("{}.toml", script_data.metadata.short_name);
        let save_path = script_dir()
            .unwrap_or(SCRIPT_PROJ_DIR.to_path_buf())
            .join(filename);

        std::fs::write(&save_path, &self.document.text).map_err(ScriptError::ErrorWritingFile)?;
        self.document.path = Some(save_path.clone());
        Ok(save_path)
    }

    fn pop_response(&mut self) -> Response
    {
        match std::mem::take(&mut self.run_state) {
            RunState::Pending(path) => {
                self.hide();
                Response::Load(path)
            }
            RunState::Idle if self.enabled() => Response::DoNothing,
            RunState::Idle => Response::Close,
        }
    }
}
