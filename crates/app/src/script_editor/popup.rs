use std::path::Path;

use dynamo_common::directories::script_dir;
use egui_file::FileDialog;

use super::config::SCRIPT_PROJ_DIR;
use super::{Response, ScriptEditor};

#[derive(Debug)]
pub enum Popup
{
    Edit(ScriptEditor),
    Load(Box<LoadPopup>),
}

#[derive(Debug)]
pub struct LoadPopup
{
    dialog: FileDialog,
    mode:   LoadMode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadMode
{
    Edit,
    Run,
}

impl Popup
{
    pub fn show(&mut self, ctx: &egui::Context)
    {
        match self {
            Self::Edit(d) => {
                d.show(ctx);
            }
            Self::Load(load_popup) => {
                load_popup.dialog.show(ctx);
            }
        }
    }

    #[must_use]
    pub fn new_script() -> Self
    {
        let mut editor = ScriptEditor::default();
        editor.open();
        Self::Edit(editor)
    }

    #[must_use]
    pub fn edit<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        let mut editor = ScriptEditor::load(path).unwrap_or_else(|_| ScriptEditor::default());
        editor.open();
        Self::Edit(editor)
    }

    #[must_use]
    pub fn load_edit() -> Self
    {
        Self::load_mode(LoadMode::Edit)
    }

    #[must_use]
    pub fn load() -> Self
    {
        Self::load_mode(LoadMode::Run)
    }

    fn load_mode(mode: LoadMode) -> Self
    {
        let path = script_dir().unwrap_or_else(|| SCRIPT_PROJ_DIR.to_path_buf());
        let _ = std::fs::create_dir_all(&path);
        let title = match mode {
            LoadMode::Edit => "Select a script to edit",
            LoadMode::Run => "Select a script to load",
        };
        let mut dialog = FileDialog::open_file().initial_path(path).title(title);
        dialog.open();

        Self::Load(Box::new(LoadPopup { dialog, mode }))
    }

    pub fn pop_response(&mut self) -> Response
    {
        match self {
            Self::Load(load_popup)
                if load_popup.mode == LoadMode::Edit && load_popup.dialog.selected() =>
            {
                if let Some(path) = load_popup.dialog.path().map(Path::to_path_buf) {
                    *self = Self::edit(path);
                }
                Response::DoNothing
            }
            Self::Load(load_popup)
                if load_popup.mode == LoadMode::Run && load_popup.dialog.selected() =>
            {
                load_popup
                    .dialog
                    .path()
                    .map_or(Response::Close, |path| Response::Load(path.to_path_buf()))
            }
            Self::Edit(editor) => editor.pop_response(),
            Self::Load(..) => Response::DoNothing,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ErrorReport
{
    title:       String,
    text:        String,
    pub visible: bool,
}

impl ErrorReport
{
    #[must_use]
    pub const fn new(title: String, text: String) -> Self
    {
        Self {
            title,
            text,
            visible: true,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context)
    {
        if self.visible {
            egui::Window::new(self.title.clone())
                .title_bar(false)
                .collapsible(false)
                .auto_sized()
                .pivot(egui::Align2::CENTER_CENTER)
                .default_pos(ctx.content_rect().center())
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(self.text.clone());
                    });

                    if ui.button("OK").clicked() || ctx.input(|i| i.key_pressed(egui::Key::Escape))
                    {
                        self.disable();
                    }
                });
        }
    }

    #[inline]
    pub const fn enable(&mut self)
    {
        self.visible = true;
    }

    #[inline]
    pub const fn disable(&mut self)
    {
        self.visible = false;
    }
}
