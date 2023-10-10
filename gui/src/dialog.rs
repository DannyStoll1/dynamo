use egui::{self, Key};
use egui::{vec2, Window};
use egui_file::FileDialog;
use fractal_common::symbolic_dynamics::AngleInfo;

use crate::interface::PaneID;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct RayParams
{
    pub pane_id: PaneID,
    pub angle_info: AngleInfo,
    pub follow: bool,
}

pub enum Dialog
{
    Save
    {
        pane_id: PaneID,
        file_dialog: FileDialog,
    },
    Text(StructuredTextDialog),
    ConfirmRay(ConfirmationDialog<RayParams>),
}

pub enum State
{
    JustOpened,
    InProgress,
    Completed,
    Closed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TextInputType
{
    ExternalRay
    {
        pane_id: PaneID, follow: bool
    },
    FindPeriodic
    {
        pane_id: PaneID
    },
    Coordinates
    {
        pane_id: PaneID
    },
}

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Response<T>
{
    Cancelled,
    InProgress,
    Complete
    {
        data: T,
    },
}

pub struct TextDialogBuilder
{
    input_type: TextInputType,
    title: String,
    prompt: String,
}

pub struct StructuredTextDialog
{
    pub input_type: TextInputType,
    pub dialog: TextDialog,
}

pub struct StructuredFileDialog
{
    pub input_type: TextInputType,
    pub dialog: TextDialog,
}

pub struct TextDialog
{
    pub title: String,
    pub prompt: String,
    pub user_input: String,
    pub state: State,
}

pub struct ConfirmationDialog<T>
{
    pub title: String,
    pub prompt: String,
    pub state: State,
    data: T,
}

impl State
{
    const fn is_open(&self) -> bool
    {
        matches!(self, Self::JustOpened | Self::InProgress)
    }
}

impl TextDialogBuilder
{
    #[must_use]
    pub const fn new(input_type: TextInputType) -> Self
    {
        Self {
            input_type,
            title: String::new(),
            prompt: String::new(),
        }
    }
    pub fn title(mut self, title: &str) -> Self
    {
        self.title = title.to_owned();
        self
    }
    pub fn prompt(mut self, prompt: &str) -> Self
    {
        self.prompt = prompt.to_owned();
        self
    }
    #[allow(clippy::missing_const_for_fn)]
    pub fn build(self) -> StructuredTextDialog
    {
        let dialog = TextDialog::new(self.title, self.prompt);
        StructuredTextDialog {
            input_type: self.input_type,
            dialog,
        }
    }
}

impl std::ops::Deref for StructuredTextDialog
{
    type Target = TextDialog;

    fn deref(&self) -> &Self::Target
    {
        &self.dialog
    }
}

impl std::ops::DerefMut for StructuredTextDialog
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.dialog
    }
}

impl TextDialog
{
    #[must_use]
    pub const fn new(title: String, prompt: String) -> Self
    {
        Self {
            title,
            prompt,
            user_input: String::new(),
            state: State::JustOpened,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context)
    {
        if self.visible()
        {
            Window::new(self.title.clone())
                .title_bar(false)
                .collapsible(false)
                .fixed_size(vec2(440.0, 250.0))
                .pivot(egui::Align2::CENTER_CENTER)
                .default_pos(ctx.screen_rect().center())
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(&self.prompt);
                        let response = ui.text_edit_singleline(&mut self.user_input);
                        ui.memory_mut(|mem| mem.request_focus(response.id));
                    });

                    if ui.button("OK").clicked() || ctx.input(|i| i.key_pressed(Key::Enter))
                    {
                        self.state = State::Completed;
                    }
                    else if ui.button("Cancel").clicked()
                        || ctx.input(|i| i.key_pressed(Key::Escape))
                    {
                        self.disable();
                    }
                });
        }
    }

    #[inline]
    pub const fn visible(&self) -> bool
    {
        self.state.is_open()
    }

    #[inline]
    pub fn enable(&mut self)
    {
        self.state = State::InProgress;
    }

    #[inline]
    pub fn disable(&mut self)
    {
        self.state = State::Closed;
        self.reset_input();
    }

    #[inline]
    pub fn set_input(&mut self, user_input: String)
    {
        self.user_input = user_input;
    }

    #[inline]
    pub fn get_input(&self) -> &str
    {
        &self.user_input
    }

    #[inline]
    pub fn reset_input(&mut self)
    {
        self.user_input = "".to_owned();
    }

    pub fn get_response(&mut self) -> Response<String>
    {
        match self.state
        {
            State::InProgress | State::JustOpened => Response::InProgress,
            State::Closed => Response::Cancelled,
            State::Completed =>
            {
                let text = std::mem::take(&mut self.user_input);
                Response::Complete { data: text }
            }
        }
    }
}

impl<T: Default> ConfirmationDialog<T>
{
    #[must_use]
    pub const fn new(title: String, prompt: String, data: T) -> Self
    {
        Self {
            title,
            prompt,
            state: State::JustOpened,
            data,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context)
    {
        if self.visible()
        {
            Window::new(self.title.clone())
                .title_bar(false)
                .collapsible(false)
                .fixed_size(vec2(320.0, 150.0))
                .pivot(egui::Align2::CENTER_CENTER)
                .default_pos(ctx.screen_rect().center())
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(&self.prompt);
                    });

                    if ui.button("OK").clicked() || ctx.input(|i| i.key_pressed(Key::Enter))
                    {
                        self.state = State::Completed;
                    }
                    else if ui.button("Cancel").clicked()
                        || ctx.input(|i| i.key_pressed(Key::Escape))
                    {
                        self.disable();
                    }
                });
        }
    }

    #[inline]
    pub const fn visible(&self) -> bool
    {
        self.state.is_open()
    }

    #[inline]
    pub fn enable(&mut self)
    {
        self.state = State::InProgress;
    }

    #[inline]
    pub fn disable(&mut self)
    {
        self.state = State::Closed;
    }

    pub fn get_response(&mut self) -> Response<T>
    {
        match self.state
        {
            State::InProgress | State::JustOpened => Response::InProgress,
            State::Closed => Response::Cancelled,
            State::Completed =>
            {
                let data = std::mem::take(&mut self.data);
                Response::Complete { data }
            }
        }
    }
}
impl Dialog
{
    pub fn show(&mut self, ctx: &egui::Context)
    {
        match self
        {
            Self::Save { file_dialog, .. } =>
            {
                file_dialog.show(ctx);
            }
            Self::Text(text_dialog) =>
            {
                text_dialog.show(ctx);
            }
            Self::ConfirmRay(conf_dialog) =>
            {
                conf_dialog.show(ctx);
            }
        }
    }

    pub fn visible(&self) -> bool
    {
        match self
        {
            Self::Save { file_dialog, .. } => file_dialog.visible(),
            Self::Text(text_dialog) => text_dialog.visible(),
            Self::ConfirmRay(conf_dialog) => conf_dialog.visible(),
        }
    }
}
