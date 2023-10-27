use std::collections::VecDeque;

use dynamo_common::rational_angle::RationalAngle;
use dynamo_common::symbolic_dynamics::{AngleInfo, OrbitSchemaWithDegree};
use dynamo_core::dynamics::PlaneType;
use egui::{self, Key, RichText, WidgetText};
use egui::{vec2, Window};
use egui_file::FileDialog;
use std::fmt::Write;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::pane::id::{PaneID, PaneSelection};

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SaveFileType
{
    Image,
    Palette,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RayParams
{
    pub pane_id: PaneID,
    pub angle_info: AngleInfo,
    pub follow: bool,
    pub ray_type: PlaneType,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AllActiveRayParams
{
    pub panes: PaneID,
    pub orbit_schema: OrbitSchemaWithDegree,
    pub active_angles: VecDeque<RationalAngle>,
    pub include_suffixes: bool,
}

pub enum Dialog
{
    Save
    {
        pane_selection: PaneSelection,
        file_dialog: FileDialog,
        file_type: SaveFileType,
    },
    Load
    {
        pane_selection: PaneSelection,
        file_dialog: FileDialog,
    },
    Text(StructuredTextDialog),
    ConfirmRay(ConfirmationDialog<RayParams>),
    ConfirmActiveRays(ConfirmationDialog<AllActiveRayParams>),
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
        pane_selection: PaneID,
        follow: bool,
    },
    ActiveRays
    {
        pane_id: PaneID,
        include_suffixes: bool,
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
pub enum Response<D>
{
    Cancelled,
    InProgress,
    Complete
    {
        data: D,
    },
}

pub struct TextDialogBuilder
{
    input_type: TextInputType,
    title: String,
    prompt: WidgetText,
}

pub struct StructuredTextDialog
{
    pub input_type: TextInputType,
    pub dialog: TextDialog,
}

pub struct TextDialog
{
    pub title: String,
    pub prompt: WidgetText,
    pub user_input: String,
    pub state: State,
}

pub struct ConfirmationDialog<D>
{
    pub title: String,
    pub prompt: WidgetText,
    pub state: State,
    data: D,
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
    pub fn new(input_type: TextInputType) -> Self
    {
        Self {
            input_type,
            title: String::new(),
            prompt: WidgetText::default(),
        }
    }
    #[must_use]
    pub fn title(mut self, title: &str) -> Self
    {
        self.title = title.to_owned();
        self
    }

    #[must_use]
    pub fn prompt(mut self, prompt: impl Into<WidgetText>) -> Self
    {
        self.prompt = prompt.into();
        self
    }
    #[allow(clippy::missing_const_for_fn)]
    #[must_use]
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
    pub fn new(title: String, prompt: impl Into<WidgetText>) -> Self
    {
        Self {
            title,
            prompt: prompt.into(),
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
                        ui.label(self.prompt.clone());
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
    #[must_use]
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
    #[must_use]
    pub fn get_input(&self) -> &str
    {
        &self.user_input
    }

    #[inline]
    pub fn reset_input(&mut self)
    {
        self.user_input.clear();
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

impl<T> ConfirmationDialog<T>
{
    #[must_use]
    pub fn new(title: String, prompt: impl Into<WidgetText>, data: T) -> Self
    {
        Self {
            title,
            prompt: prompt.into(),
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
                        ui.label(self.prompt.clone());
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
    where
        T: Default,
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
            Self::Save { file_dialog, .. } | Self::Load { file_dialog, .. } =>
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
            Self::ConfirmActiveRays(conf_dialog) =>
            {
                conf_dialog.show(ctx);
            }
        }
    }

    #[must_use]
    pub fn visible(&self) -> bool
    {
        match self
        {
            Self::Save { file_dialog, .. } | Self::Load { file_dialog, .. } =>
            {
                file_dialog.visible()
            }
            Self::Text(text_dialog) => text_dialog.visible(),
            Self::ConfirmRay(conf_dialog) => conf_dialog.visible(),
            Self::ConfirmActiveRays(conf_dialog) => conf_dialog.visible(),
        }
    }

    #[must_use]
    pub fn confirm_ray(ray_params: RayParams) -> Self
    {
        let title = "Confirm external ray".to_owned();
        let prompt = format!(
            "The {} ray at angle {} has preperiod {} and period {}.\nThe associated kneading sequence is {}",
            ray_params.ray_type,
            ray_params.angle_info.angle,
            ray_params.angle_info.orbit_schema.preperiod,
            ray_params.angle_info.orbit_schema.period,
            ray_params.angle_info.kneading_sequence
        );
        let conf_dialog = ConfirmationDialog::new(title, prompt, ray_params);
        Self::ConfirmRay(conf_dialog)
    }

    #[must_use]
    pub fn confirm_active_rays(params: AllActiveRayParams) -> Self
    {
        let title = "Confirm active rays".to_owned();
        let OrbitSchemaWithDegree {
            preperiod,
            period,
            degree,
        } = params.orbit_schema;

        let header = format!(
            "The following angles are active with preperiod {preperiod} and period {period}:"
        );

        let pad = (period + preperiod + 1) as usize;
        let mut body = String::new();
        for a in &params.active_angles
        {
            let _ = writeln!(&mut body, "{:>8} = {:>pad$}", a, a.with_degree(degree));
        }
        let prompt = RichText::from(format!("{header}\n{body}")).monospace();
        let conf_dialog = ConfirmationDialog::new(title, prompt, params);
        Self::ConfirmActiveRays(conf_dialog)
    }
}
