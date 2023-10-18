use egui_file::FileDialog;
use script_loader::error::ScriptError;
use script_loader::parser::UnparsedUserInput;
use std::path::PathBuf;

mod config;
use config::SCRIPT_DIR;

#[derive(Clone, Debug, Default)]
pub enum Response
{
    #[default]
    DoNothing,
    Close,
    Load(PathBuf),
}

#[derive(Debug)]
pub enum Popup
{
    Edit(ScriptEditor),
    Load(FileDialog),
}
impl Popup
{
    pub fn show(&mut self, ctx: &egui::Context)
    {
        match self
        {
            Self::Edit(d) =>
            {
                d.show(ctx);
            }
            Self::Load(d) =>
            {
                d.show(ctx);
            }
        }
    }

    #[must_use]
    pub fn edit() -> Self
    {
        let mut editor = ScriptEditor::default();
        editor.open();
        Self::Edit(editor)
    }

    #[must_use]
    pub fn load() -> Self
    {
        let path = Some((*SCRIPT_DIR).to_path_buf());
        let _ = std::fs::create_dir("images");
        let mut file_dialog = FileDialog::open_file(path).title("Select a script to load");
        file_dialog.open();

        Self::Load(file_dialog)
    }

    pub fn pop_response(&mut self) -> Response
    {
        match self
        {
            Self::Load(dialog) if dialog.selected() => dialog
                .path()
                .map(|path| Response::Load(path.to_path_buf()))
                .unwrap_or(Response::Close),
            Self::Load(_) => Response::DoNothing,
            Self::Edit(editor) => editor.pop_response(),
        }
    }
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
        if let Self::ReadyToRun(p) = self
        {
            let path = std::mem::take(p);
            *self = Self::Closed;
            Some(path)
        }
        else
        {
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
    pub fn show(&mut self, ctx: &egui::Context)
    {
        if matches!(self.state, State::Editing)
        {
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
                    if ui.button("Save").clicked()
                    {
                        self.try_save(false);
                    }
                    if ui.button("Save and Run").clicked()
                    {
                        self.try_save(true);
                    }
                    if ui.button("Cancel").clicked()
                    {
                        self.hide();
                        self.reset_text();
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

    #[inline]
    pub fn reset_text(&mut self)
    {
        self.text = config::DEFAULT_TEXT.clone();
    }

    fn try_save(&mut self, run: bool)
    {
        match self.save_script()
        {
            Ok(script_path) =>
            {
                println!("Script saved to {}.", script_path.display());
                if run
                {
                    self.state = State::ReadyToRun(script_path);
                }
                else
                {
                    self.hide();
                }
                self.reset_text();
            }
            Err(e) =>
            {
                println!("Error saving script: {e:?}");
            }
        }
    }

    fn save_script(&self) -> Result<PathBuf, ScriptError>
    {
        let script_data: UnparsedUserInput =
            toml::from_str(&self.text).map_err(ScriptError::ErrorParsingToml)?;
        let filename = format!("{}.toml", script_data.metadata.short_name);
        let save_path = config::SCRIPT_DIR.join(filename);

        std::fs::write(&save_path, self.text.clone()).map_err(ScriptError::ErrorWritingFile)?;
        Ok(save_path)
    }

    fn pop_response(&mut self) -> Response
    {
        match self.state.pop_if_ready()
        {
            Some(path) => Response::Load(path),
            None if self.enabled() => Response::DoNothing,
            None => Response::Close,
        }
    }
}
