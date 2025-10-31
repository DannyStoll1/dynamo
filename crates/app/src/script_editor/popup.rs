use std::path::Path;

use dynamo_common::directories::script_dir;
use egui_file::FileDialog;

use super::config::SCRIPT_PROJ_DIR;
use super::{Response, ScriptEditor};

#[derive(Debug)]
pub enum Popup
{
    Edit(ScriptEditor),
    Load
    {
        dialog:     FileDialog,
        edit_after: bool,
    },
}
impl Popup
{
    pub fn show(&mut self, ctx: &egui::Context)
    {
        match self {
            Self::Edit(d) => {
                d.show(ctx);
            }
            Self::Load { dialog, .. } => {
                dialog.show(ctx);
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
        let mut editor = ScriptEditor::load(path).unwrap_or_default();
        editor.open();
        Self::Edit(editor)
    }

    #[must_use]
    pub fn load_edit() -> Self
    {
        let path = Some(script_dir().unwrap_or(SCRIPT_PROJ_DIR.to_path_buf()));
        let _ = std::fs::create_dir("user_scripts");
        let mut dialog = FileDialog::open_file(path).title("Select a script to edit");
        dialog.open();

        Self::Load {
            dialog,
            edit_after: true,
        }
    }

    #[must_use]
    pub fn load() -> Self
    {
        let path = Some(script_dir().unwrap_or(SCRIPT_PROJ_DIR.to_path_buf()));
        let _ = std::fs::create_dir("user_scripts");
        let mut dialog = FileDialog::open_file(path).title("Select a script to load");
        dialog.open();

        Self::Load {
            dialog,
            edit_after: false,
        }
    }

    pub fn pop_response(&mut self) -> Response
    {
        match self {
            Self::Load {
                dialog,
                edit_after: true,
            } if dialog.selected() => {
                if let Some(path) = dialog.path().map(|path| path.to_path_buf()) {
                    *self = Self::edit(path);
                }
                Response::DoNothing
            }
            Self::Load {
                dialog,
                edit_after: false,
            } if dialog.selected() => dialog
                .path()
                .map(|path| Response::Load(path.to_path_buf()))
                .unwrap_or(Response::Close),
            Self::Edit(editor) => editor.pop_response(),
            _ => Response::DoNothing,
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
    pub fn new(title: String, text: String) -> Self
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
    pub fn enable(&mut self)
    {
        self.visible = true;
    }

    #[inline]
    pub fn disable(&mut self)
    {
        self.visible = false;
    }
}
