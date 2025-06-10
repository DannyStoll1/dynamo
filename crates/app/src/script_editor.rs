use dynamo_common::directories::script_dir;
use script_loader::error::ScriptError;
use script_loader::parser::UnparsedUserInput;
use std::path::{Path, PathBuf};

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

#[derive(Clone, Default, Debug)]
pub enum State
{
    #[default]
    Closed,
    Editing,
    ReadyToRun(PathBuf),
}
impl State
{
    pub fn pop_if_ready(&mut self) -> Option<PathBuf>
    {
        if let Self::ReadyToRun(p) = self {
            let path = std::mem::take(p);
            *self = Self::Closed;
            Some(path)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub struct ScriptEditor
{
    pub text: String,
    pub state: State,
}
impl Default for ScriptEditor
{
    fn default() -> Self
    {
        Self {
            text: config::DEFAULT_TEXT.clone(),
            state: State::default(),
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
        let text = std::fs::read_to_string(script_file)?;
        Ok(Self {
            text,
            ..Default::default()
        })
    }

    pub fn show(&mut self, ctx: &egui::Context)
    {
        if matches!(self.state, State::Editing) {
            egui::Window::new("Script Editor")
                .vscroll(true)
                .default_height(720.)
                .default_width(600.)
                .show(ctx, |ui| {
                    egui::TextEdit::multiline(&mut self.text)
                        .code_editor()
                        .desired_rows(30)
                        .desired_width(std::f32::INFINITY)
                        .show(ui);
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
        self.state = State::Editing;
    }

    #[inline]
    pub fn enabled(&self) -> bool
    {
        matches!(self.state, State::Editing)
    }

    #[inline]
    pub fn hide(&mut self)
    {
        self.state = State::Closed;
    }

    fn try_save(&mut self, run: bool)
    {
        match self.save_script() {
            Ok(script_path) => {
                info!("Script saved to {}.", script_path.display());
                if run {
                    self.state = State::ReadyToRun(script_path);
                } else {
                    self.hide();
                }
            }
            Err(e) => {
                error!("Error saving script: {e:?}");
            }
        }
    }

    fn save_script(&self) -> Result<PathBuf, ScriptError>
    {
        let script_data: UnparsedUserInput =
            toml::from_str(&self.text).map_err(ScriptError::ErrorParsingToml)?;
        let filename = format!("{}.toml", script_data.metadata.short_name);
        let save_path = script_dir()
            .unwrap_or(SCRIPT_PROJ_DIR.to_path_buf())
            .join(filename);

        std::fs::write(&save_path, self.text.clone()).map_err(ScriptError::ErrorWritingFile)?;
        Ok(save_path)
    }

    fn pop_response(&mut self) -> Response
    {
        match self.state.pop_if_ready() {
            Some(path) => Response::Load(path),
            None if self.enabled() => Response::DoNothing,
            None => Response::Close,
        }
    }
}
