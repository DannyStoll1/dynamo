use std::path::PathBuf;

use egui::{Context, CursorIcon, InputState, Ui};
use egui_extras::{Column, TableBuilder};
use egui_file::FileDialog;

use dynamo_color::{IncoloringAlgorithm, Palette};
use dynamo_common::prelude::*;
use dynamo_core::{dynamics::Displayable, prelude::HasChild};

use crate::{
    actions::Action,
    dialog::*,
    hotkeys::{
        keyboard_shortcuts::*, Hotkey, ANNOTATION_HOTKEYS, FILE_HOTKEYS, IMAGE_HOTKEYS,
        INCOLORING_HOTKEYS, PALETTE_HOTKEYS, SELECTION_HOTKEYS,
    },
    pane::{
        id::*,
        tasks::{ChildTask, FollowState, SelectOrFollow},
        Pane, WindowPane,
    },
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
/// Represents different types of messages that can be sent within the UI.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum UiMessage
{
    #[default]
    DoNothing,
    CloseWindow,
    Quit,
    NewTab,
}
impl UiMessage
{
    /// Takes the current value out of the `UiMessage`, leaving `DoNothing` in its place.
    fn pop(&mut self) -> Self
    {
        std::mem::take(self)
    }
}

/// A trait defining the relationship between a parent and child pane.
pub trait PanePair
{
    fn parent(&self) -> &dyn Pane;
    fn parent_mut(&mut self) -> &mut dyn Pane;
    fn child(&self) -> &dyn Pane;
    fn child_mut(&mut self) -> &mut dyn Pane;
    fn randomize_palette(&mut self);
    fn set_palette(&mut self, palette: Palette);
    fn set_coloring_algorithm(&mut self, coloring_algorithm: IncoloringAlgorithm);

    fn get_pane(&self, pane_id: PaneID) -> &dyn Pane;
    fn get_pane_mut(&mut self, pane_id: PaneID) -> &mut dyn Pane;
    fn set_active_pane(&mut self, pane_id: Option<PaneID>);
    fn get_active_pane(&self) -> Option<&dyn Pane>;
    fn get_active_pane_mut(&mut self) -> Option<&mut dyn Pane>;
    fn get_selected_pane_ids(&self, selection: PaneSelection) -> Vec<PaneID>;
    fn prompt_save_image(&mut self, panes: PaneSelection);
    fn prompt_save_palette(&mut self, panes: PaneSelection);
    fn prompt_load_palette(&mut self, panes: PaneSelection);
    fn prompt_text(&mut self, input_type: TextInputType);

    /// Updates the state of both the parent and child panes.
    fn update_panes(&mut self);

    // fn descend(self) -> Box<dyn PanePair>;
}

/// A trait for interactive elements within the UI that can handle input and display dialogs.
pub trait Interactive
{
    /// Displays a dialog to the user, if any is active.
    fn show_dialog(&mut self, ctx: &Context);
    fn consume_click(&mut self);
    fn reset_click(&mut self);
    fn handle_input(&mut self, ctx: &Context);
    fn get_message(&self) -> UiMessage;
    fn pop_message(&mut self) -> UiMessage;
    fn name(&self) -> String;
    fn get_image_height(&self) -> usize;
    fn change_height(&mut self, new_height: usize);
    fn show(&mut self, ui: &mut Ui);
    fn process_action(&mut self, action: &Action);
}

/// The main interface structure that holds the parent and child panes along with UI state.
pub struct MainInterface<P, J>
where
    P: Displayable + Clone + 'static,
    J: Displayable + Clone + 'static,
{
    parent: WindowPane<P>,
    child: WindowPane<J>,
    image_height: usize,
    active_pane: Option<PaneID>,
    live_mode: bool,
    dialog: Option<Dialog>,
    // save_task: SaveTask,
    click_used: bool,
    pub message: UiMessage,
}

impl<P, J> MainInterface<P, J>
where
    P: Displayable + HasChild<J> + Clone,
    J: Displayable + Clone,
{
    /// Constructs a new `MainInterface` with the given parent and child panes and image height.
    pub fn new(parent: P, child: J, image_height: usize) -> Self
    {
        Self {
            parent: parent.into(),
            child: child.into(),
            image_height,
            active_pane: Some(PaneID::Parent),
            live_mode: false,
            dialog: None,
            click_used: false,
            message: UiMessage::default(),
        }
    }

    /// Sets a new parameter for the child pane based on the parent pane's parameter.
    fn set_child_param(&mut self, new_param: P::Param)
    {
        let old_center = self.child.grid().center();
        let old_default_center = self.child.plane.default_bounds().center();

        if self.child.set_param(P::to_child_param(new_param)) {
            let mut new_bounds = self.child.plane.default_bounds();

            // Set the new center to equal the old center plus whatever deviation the user has created
            let offset = new_bounds.center() - old_default_center;
            let new_center = old_center + offset;

            if offset.is_finite() {
                new_bounds.zoom(self.child.zoom_factor, new_center);
                new_bounds.recenter(new_center);
                self.child.grid_mut().change_bounds(new_bounds);
                self.child.schedule_recompute();
            } else {
                // Reset child bounds to default
                self.child.grid_mut().change_bounds(new_bounds);
                self.child.grid_mut().resize_y(self.image_height);
                self.child.schedule_compute();
            }
        }
    }

    /// Closes the currently active dialog, if any.
    #[inline]
    fn close_dialog(&mut self)
    {
        self.dialog = None;
    }

    fn handle_save_dialog(
        &mut self,
        selection: PaneSelection,
        file_dialog: &FileDialog,
        file_type: SaveFileType,
    )
    {
        use SaveFileType::*;

        // Ensure file selection was confirmed
        if !file_dialog.selected() {
            return;
        }

        let Some(path) = file_dialog.path() else {
            return;
        };

        let pane_ids = self.get_selected_pane_ids(selection);

        match file_type {
            Image => {
                let image_width: usize = 4096; // You can make this dynamic as per your requirement
                pane_ids
                    .into_iter()
                    .for_each(|pane_id| self.get_pane_mut(pane_id).save_image(image_width, path));
            }
            Palette => {
                pane_ids
                    .into_iter()
                    .for_each(|pane_id| self.get_pane_mut(pane_id).save_palette(path));
            }
        }
        self.set_active_pane(None);
    }

    fn handle_load_dialog(&mut self, file_dialog: &FileDialog, pane_selection: PaneSelection)
    {
        // Ensure file selection was confirmed
        if !file_dialog.selected() {
            return;
        }

        let Some(path) = file_dialog.path() else {
            return;
        };

        self.get_selected_pane_ids(pane_selection)
            .into_iter()
            .for_each(|pane_id| {
                self.get_pane_mut(pane_id).load_palette(path);
            });

        self.set_active_pane(None);
    }

    fn process_text_dialog_input(
        &mut self,
        input_type: TextInputType,
        text: &str,
        toggle_map: &ToggleMap,
    )
    {
        use crate::dialog::TextInputType::*;
        use crate::dialog::ToggleKey::*;
        match input_type {
            ExternalRay { .. } => {
                if let Ok(angle) = text.parse::<RationalAngle>() {
                    let angle_info = angle.with_degree(self.child.degree()).to_angle_info();

                    let include_orbit = toggle_map.get(DrawOrbit);

                    let follow_task = if toggle_map.get(FollowPoint) {
                        SelectOrFollow::Follow
                    } else if toggle_map.get(SelectPoint) {
                        SelectOrFollow::Select
                    } else {
                        SelectOrFollow::DoNothing
                    };

                    let ray_params = RayParams {
                        do_parent: toggle_map.get(DoParent),
                        do_child: toggle_map.get(DoChild),
                        angle_info,
                        follow_task,
                        include_orbit,
                    };
                    let dialog = Dialog::confirm_ray(ray_params);
                    self.dialog = Some(dialog);
                }
            }
            ActiveRays { pane_id } => {
                if let Ok(o) = text.parse::<OrbitSchema>() {
                    let include_suffixes = toggle_map.get(PrefixAngles);

                    let degree = self.get_pane(pane_id).degree();
                    let od = o.with_degree(degree);
                    let active_angles = od.active_angles(include_suffixes);

                    let params = AllActiveRayParams {
                        do_parent: toggle_map.get(DoParent),
                        do_child: toggle_map.get(DoChild),
                        orbit_schema: o.with_degree(degree),
                        active_angles,
                        include_suffixes,
                    };

                    let dialog = Dialog::confirm_active_rays(params);
                    self.dialog = Some(dialog);
                }
            }
            Coordinates { pane_id } => {
                if let Ok(point) = text.parse::<Cplx>() {
                    let pane = self.get_pane_mut(pane_id);
                    pane.select_point(point);
                    pane.stop_following();
                }
            }
            FindPeriodic { pane_id } => {
                if let Ok(orbit_schema) = text.parse::<OrbitSchema>() {
                    let follow = toggle_map.get(FollowPoint);
                    match pane_id {
                        PaneID::Child => {
                            self.child_mut()
                                .set_follow_state(FollowState::SelectPeriodic {
                                    orbit_schema,
                                    follow,
                                });
                        }
                        PaneID::Parent => {
                            if self.parent_mut().select_nearby_point(orbit_schema).is_ok() {
                                self.process_child_task();
                            }
                        }
                    }
                }
            }
        }
    }

    /// Draw a ray, and possibly select or follow it, according to the ray_params provided from a
    /// confirmation dialog response.
    fn process_conf_ray_response(&mut self, ray_params: &RayParams)
    {
        if ray_params.do_parent {
            let pane = self.get_pane_mut(PaneID::Parent);
            ray_params.follow_task.run_on(pane, ray_params);
        }
        if ray_params.do_child {
            let pane = self.get_pane_mut(PaneID::Child);
            ray_params.follow_task.run_on(pane, ray_params);
        }
    }

    fn process_child_task(&mut self)
    {
        if self.parent.pop_child_task() == ChildTask::UpdateParam {
            let parent_selection = self.parent.get_selection();
            let new_child_param = self.parent.plane.param_map(parent_selection);
            self.set_child_param(new_child_param);
        }
    }

    /// Handles mouse input, updating the state of the panes accordingly.
    fn handle_mouse(&mut self, ctx: &Context)
    {
        let clicked = ctx.input(|i| i.pointer.any_click()) && !self.click_used;
        let zoom_factor = ctx.input(InputState::zoom_delta);

        self.reset_click();

        let Some(pointer_pos) = ctx.pointer_latest_pos() else {
            return;
        };

        if self.parent().frame_contains_pixel(pointer_pos) {
            ctx.set_cursor_icon(CursorIcon::Crosshair);
            self.set_active_pane(Some(PaneID::Parent));
            let reselect_point = self.live_mode || clicked;
            let pointer_value = self.parent().map_pixel(pointer_pos);
            self.parent_mut()
                .process_mouse_input(pointer_value, zoom_factor, reselect_point);
            self.process_child_task();

            if clicked {
                self.consume_click();
                self.parent_mut().marking_mut().enable_selection();
            }
        } else if self.child().frame_contains_pixel(pointer_pos) {
            ctx.set_cursor_icon(CursorIcon::Crosshair);
            self.set_active_pane(Some(PaneID::Child));
            let pointer_value = self.child().map_pixel(pointer_pos);
            self.child_mut()
                .process_mouse_input(pointer_value, zoom_factor, clicked);

            if clicked {
                self.consume_click();
                self.child_mut().marking_mut().enable_selection();
            }
        } else {
            ctx.set_cursor_icon(CursorIcon::Default);
        }
    }

    /// Schedules a message to close the current window.
    fn schedule_close(&mut self)
    {
        self.message = UiMessage::CloseWindow;
    }

    /// Schedules a message to quit the application.
    fn schedule_quit(&mut self)
    {
        self.message = UiMessage::Quit;
    }

    /// Schedules a message to open a new tab.
    fn schedule_new_tab(&mut self)
    {
        self.message = UiMessage::NewTab;
    }

    /// Toggles the live mode state of the interface.
    fn toggle_live_mode(&mut self)
    {
        self.live_mode ^= true;
        if self.live_mode {
            self.parent.stop_following();
            self.child.frame_mut().set_live();
        } else {
            self.child.frame_mut().unset_live();
        }
    }

    /// Checks if there is a visible dialog currently active.
    fn has_visible_dialog(&self) -> bool
    {
        self.dialog.as_ref().is_some_and(Dialog::visible)
    }
}

/// Implementation of `PanePair` for `MainInterface`, providing access to parent and child panes.
impl<P, J> PanePair for MainInterface<P, J>
where
    P: Displayable + HasChild<J> + Clone,
    J: Displayable + Clone,
{
    /// Returns a reference to the parent pane.
    fn parent(&self) -> &dyn Pane
    {
        &self.parent
    }
    /// Returns a mutable reference to the parent pane.
    fn parent_mut(&mut self) -> &mut dyn Pane
    {
        &mut self.parent
    }
    /// Returns a reference to the child pane.
    fn child(&self) -> &dyn Pane
    {
        &self.child
    }
    /// Returns a mutable reference to the child pane.
    fn child_mut(&mut self) -> &mut dyn Pane
    {
        &mut self.child
    }
    /// Randomizes the color palette for both the parent and child panes.
    fn randomize_palette(&mut self)
    {
        let palette = Palette::new_random(0.45, 0.38);
        self.parent.change_palette(palette);
        self.child.change_palette(palette);
    }

    /// Prompt for text input for a specified purpose.
    fn prompt_text(&mut self, input_type: TextInputType)
    {
        use TextInputType::*;
        let text_dialog = match input_type {
            ExternalRay {
                pane_id,
                include_orbit,
                select_landing_point,
            } => {
                let prompt = concat!(
                    "Input an angle to draw a ray\n",
                    "Example formats: <15/56>, <110>, <p011>, <001p010>",
                );
                let builder = TextDialogBuilder::new(input_type)
                    .title("External ray angle input")
                    .prompt(prompt)
                    .pane_toggles("Draw on", pane_id)
                    .add_toggle_with_default(
                        ToggleKey::DrawOrbit,
                        "Include orbit".to_owned(),
                        include_orbit,
                    )
                    .add_toggle_with_default(
                        ToggleKey::SelectPoint,
                        "Select landing point".to_owned(),
                        select_landing_point,
                    );
                if matches!(pane_id, PaneID::Child) {
                    builder
                        .add_cond_toggle(ToggleKey::FollowPoint, "Follow landing point".to_owned())
                        .build()
                } else {
                    builder.build()
                }
            }
            ActiveRays { pane_id } => {
                let prompt = concat!(
                    "Input the period to draw all active rays.\n",
                    "Format: <period> or <preperiod, period>"
                );
                TextDialogBuilder::new(input_type)
                    .title("Draw active rays")
                    .prompt(prompt)
                    .pane_toggles("Draw on", pane_id)
                    .add_toggle(
                        ToggleKey::PrefixAngles,
                        "Include rays of shorter preperiod".to_owned(),
                    )
                    .build()
            }
            FindPeriodic { pane_id, .. } => {
                let pane = self.get_pane(pane_id);
                let prompt = format!(
                    concat!(
                        "Input the period to find a nearby point on {pane_name}.\n",
                        "Format: <period> or <preperiod, period>"
                    ),
                    pane_name = pane.name()
                );
                let builder = TextDialogBuilder::new(input_type)
                    .title("Find nearby point")
                    .prompt(prompt);
                if matches!(pane_id, PaneID::Child) {
                    builder
                        .add_toggle(ToggleKey::FollowPoint, "Follow point".to_owned())
                        .build()
                } else {
                    builder.build()
                }
            }
            Coordinates { pane_id } => {
                let pane = self.get_pane(pane_id);
                let prompt = format!(
                    "Enter the coordinates of the point to select on {pane_name}",
                    pane_name = pane.name()
                );
                TextDialogBuilder::new(input_type)
                    .title("Input coordinates")
                    .prompt(prompt)
                    .build()
            }
        };
        let dialog = Dialog::Text(text_dialog);
        self.dialog = Some(dialog);
    }

    /// Open a dialog prompt to save an image.
    fn prompt_save_image(&mut self, pane_selection: PaneSelection)
    {
        let path = PathBuf::from("images");
        let _ = std::fs::create_dir(&path);
        let mut file_dialog = FileDialog::save_file(Some(path))
            .title("Save Image")
            .show_rename(false)
            .show_new_folder(true);
        file_dialog.open();
        let file_dialog = file_dialog.default_filename(format!("{}.png", self.parent.long_name()));
        self.dialog = Some(Dialog::Save {
            pane_selection,
            file_dialog,
            file_type: SaveFileType::Image,
        });
    }

    fn prompt_save_palette(&mut self, panes: PaneSelection)
    {
        let path = PathBuf::from("palettes");
        let _ = std::fs::create_dir(&path);
        let mut file_dialog = FileDialog::save_file(Some(path))
            .title("Save Palette")
            .show_rename(false)
            .show_new_folder(true);
        file_dialog.open();
        let file_dialog = file_dialog.default_filename("palette.toml");
        self.dialog = Some(Dialog::Save {
            pane_selection: panes,
            file_dialog,
            file_type: SaveFileType::Palette,
        });
    }

    fn prompt_load_palette(&mut self, pane_selection: PaneSelection)
    {
        let path = PathBuf::from("palettes");
        let _ = std::fs::create_dir(&path);
        let mut file_dialog = FileDialog::open_file(Some(path))
            .title("Load Palette")
            .show_rename(false)
            .show_new_folder(false);
        file_dialog.open();
        self.dialog = Some(Dialog::Load {
            pane_selection,
            file_dialog,
        });
    }

    fn set_active_pane(&mut self, pane_id: Option<PaneID>)
    {
        self.active_pane = pane_id;
        match pane_id {
            None => {
                self.child_mut().frame_mut().deselect();
                self.parent_mut().frame_mut().deselect();
            }
            Some(PaneID::Child) => {
                self.child_mut().frame_mut().select();
                self.parent_mut().frame_mut().deselect();
            }
            Some(PaneID::Parent) => {
                self.parent_mut().frame_mut().select();
                self.child_mut().frame_mut().deselect();
            }
        }
    }

    fn get_pane(&self, pane_id: PaneID) -> &dyn Pane
    {
        match pane_id {
            PaneID::Parent => self.parent(),
            PaneID::Child => self.child(),
        }
    }

    fn get_pane_mut(&mut self, pane_id: PaneID) -> &mut dyn Pane
    {
        match pane_id {
            PaneID::Parent => self.parent_mut(),
            PaneID::Child => self.child_mut(),
        }
    }

    fn get_active_pane(&self) -> Option<&dyn Pane>
    {
        Some(self.get_pane(self.active_pane?))
    }

    fn get_active_pane_mut(&mut self) -> Option<&mut dyn Pane>
    {
        Some(self.get_pane_mut(self.active_pane?))
    }

    fn get_selected_pane_ids(&self, selection: PaneSelection) -> Vec<PaneID>
    {
        use PaneSelection::*;
        match selection {
            ActivePane => self
                .active_pane
                .map(|pane_id| vec![pane_id])
                .unwrap_or_default(),
            BothPanes => vec![PaneID::Parent, PaneID::Child],
            Id(pane_id) => vec![pane_id],
        }
    }

    /// Sets a new color palette for both the parent and child panes.
    fn set_palette(&mut self, palette: Palette)
    {
        self.parent.change_palette(palette);
        self.child.change_palette(palette);
    }

    /// Sets a new incoloring algorithm for both the parent and child panes.
    fn set_coloring_algorithm(&mut self, coloring_algorithm: IncoloringAlgorithm)
    {
        match coloring_algorithm {
            IncoloringAlgorithm::InternalPotential { .. } => {
                self.parent_mut().select_preperiod_smooth_coloring();
                self.child_mut().select_preperiod_smooth_coloring();
            }
            IncoloringAlgorithm::PotentialAndPeriod { .. } => {
                self.parent_mut().select_preperiod_period_smooth_coloring();
                self.child_mut().select_preperiod_period_smooth_coloring();
            }
            _ => {
                self.parent_mut()
                    .set_coloring_algorithm(coloring_algorithm.clone());
                self.child_mut().set_coloring_algorithm(coloring_algorithm);
            }
        }
    }

    fn update_panes(&mut self)
    {
        self.parent.process_tasks();
        self.child.process_tasks();
    }

    // fn descend(self) -> Box<dyn PanePair>
    // {
    //     let new_parent = self.child.plane;
    //     let new_child = C::from(new_parent.clone());
    //     Box::new(make_interface(new_parent, new_child))
    //     // Box::from(MainInterface::new(new_parent, new_child))
    // }
}

/// Implementation of `Interactive` for `MainInterface`, handling input and dialogs.
impl<P, J> Interactive for MainInterface<P, J>
where
    P: Displayable + HasChild<J> + Clone,
    J: Displayable + Clone,
{
    /// Handles user input and updates the state of the interface accordingly.
    fn handle_input(&mut self, ctx: &Context)
    {
        // Don't process input if the user is in a dialog
        if self.has_visible_dialog() {
            ctx.set_cursor_icon(CursorIcon::Default);
            return;
        }
        for Hotkey {
            shortcut, action, ..
        } in FILE_HOTKEYS
            .iter()
            .chain(IMAGE_HOTKEYS.iter())
            .chain(ANNOTATION_HOTKEYS.iter())
            .chain(SELECTION_HOTKEYS.iter())
            .chain(INCOLORING_HOTKEYS.iter())
            .chain(PALETTE_HOTKEYS.iter())
        {
            if let Some(s) = shortcut.as_ref() {
                if shortcut_used!(ctx, s) {
                    self.process_action(action);
                }
            }
        }
        self.handle_mouse(ctx);
    }

    fn show_dialog(&mut self, ctx: &Context)
    {
        let dialog = self.dialog.take();

        if let Some(mut dialog) = dialog {
            dialog.show(ctx);

            match &mut dialog {
                Dialog::Save {
                    pane_selection: panes,
                    file_dialog,
                    file_type,
                } => self.handle_save_dialog(*panes, file_dialog, *file_type),
                Dialog::Load {
                    file_dialog,
                    pane_selection,
                } => self.handle_load_dialog(file_dialog, *pane_selection),
                Dialog::Text(text_dialog) => {
                    if let crate::dialog::Response::Complete { data } = text_dialog.get_response() {
                        let (text, toggle_map) = data;
                        self.process_text_dialog_input(text_dialog.input_type, &text, &toggle_map);
                    }
                }
                Dialog::ConfirmRay(conf_dialog) => {
                    if let crate::dialog::Response::Complete { data } = conf_dialog.get_response() {
                        self.process_conf_ray_response(&data);
                    }
                }
                Dialog::ConfirmActiveRays(conf_dialog) => {
                    if let crate::dialog::Response::Complete { data } = conf_dialog.get_response() {
                        let mut draw_rays = |pane_id| {
                            let pane = self.get_pane_mut(pane_id);
                            for angle in &data.active_angles {
                                pane.marking_mut().enable_ray(*angle);
                            }
                            pane.schedule_redraw();
                        };

                        if data.do_child {
                            draw_rays(PaneID::Child);
                        }
                        if data.do_parent {
                            draw_rays(PaneID::Parent);
                        }
                    }
                }
            }

            if dialog.visible() {
                self.dialog = Some(dialog);
            }
        }
    }

    #[inline]
    fn get_message(&self) -> UiMessage
    {
        self.message
    }

    #[inline]
    fn pop_message(&mut self) -> UiMessage
    {
        self.message.pop()
    }

    fn consume_click(&mut self)
    {
        self.click_used = true;
    }

    fn reset_click(&mut self)
    {
        self.click_used = false;
    }

    fn name(&self) -> String
    {
        self.parent.name()
    }

    fn get_image_height(&self) -> usize
    {
        self.image_height
    }

    fn change_height(&mut self, new_height: usize)
    {
        self.image_height = new_height;
        self.parent.change_height(new_height);
        self.child.change_height(new_height);
    }

    /// Renders the UI elements of the main interface, which consist of the parent plane, child
    /// plane, plane names, and orbit descriptions. The menus are handled by the parent struct `app::FracalTab`.
    fn show(&mut self, ui: &mut Ui)
    {
        TableBuilder::new(ui)
            .column(Column::exact(self.parent.get_image_frame().width() as f32))
            .column(Column::remainder())
            .vscroll(false)
            .stick_to_bottom(true)
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.heading(self.parent().long_name());
                });
                header.col(|ui| {
                    ui.heading(self.child().long_name());
                });
            })
            .body(|mut body| {
                body.row(self.parent.get_image_frame().height() as f32, |mut row| {
                    row.col(|ui| {
                        self.parent.get_image_frame_mut().put(ui);
                        self.parent.put_marked_curves(ui);
                        self.parent.put_marked_points(ui);
                    });
                    row.col(|ui| {
                        self.child.get_image_frame_mut().put(ui);
                        self.child.put_marked_curves(ui);
                        self.child.put_marked_points(ui);
                    });
                });
                body.row(80., |mut row| {
                    row.col(|ui| {
                        ui.label(self.parent.state_info());
                    });
                    row.col(|ui| {
                        ui.label(self.child.state_info());
                    });
                });
            });
    }

    #[allow(clippy::option_map_unit_fn)]
    /// Processes an action and updates the state of the interface accordingly.
    fn process_action(&mut self, action: &Action)
    {
        match action {
            Action::Quit => self.schedule_quit(),
            Action::Close => self.schedule_close(),
            Action::NewTab => self.schedule_new_tab(),
            Action::SaveImage(panes) => self.prompt_save_image(*panes),
            Action::SavePalette(panes) => self.prompt_save_palette(*panes),
            Action::LoadPalette(panes) => self.prompt_load_palette(*panes),
            Action::ToggleSelectionMarker => {
                if let Some(pane) = self.get_active_pane_mut() {
                    pane.marking_mut().toggle_selection();
                    pane.schedule_redraw();
                }
            }
            Action::ToggleCritical(selection) => {
                self.get_selected_pane_ids(*selection)
                    .into_iter()
                    .for_each(|pane_id| {
                        let pane = self.get_pane_mut(pane_id);
                        pane.marking_mut().toggle_critical();
                        pane.schedule_redraw();
                    });
            }
            Action::ToggleCycles(selection, period) => {
                self.get_selected_pane_ids(*selection)
                    .into_iter()
                    .for_each(|pane_id| {
                        let pane = self.get_pane_mut(pane_id);
                        pane.marking_mut().toggle_cycles_of_period(*period);
                        pane.schedule_redraw();
                    });
            }
            Action::FindPeriodicPoint => {
                if let Some(pane_id) = self.active_pane {
                    let input_type = TextInputType::FindPeriodic { pane_id };
                    self.prompt_text(input_type);
                }
            }
            Action::EnterCoordinates => {
                if let Some(pane_id) = self.active_pane {
                    let input_type = TextInputType::Coordinates { pane_id };
                    self.prompt_text(input_type);
                }
            }
            Action::MapSelection => {
                let plane = self.child_mut();
                plane.map_selection();
                plane.marking_mut().enable_selection();
            }
            Action::DrawOrbit => {
                let plane = self.child_mut();
                plane.tasks_mut().orbit.enable();
            }
            Action::StopFollowing => {
                self.child_mut().stop_following();
            }
            Action::ClearOrbit => {
                self.child_mut().clear_marked_orbit();
            }
            Action::DrawExternalRay {
                include_orbit,
                select_landing_point,
            } => {
                if let Some(pane_id) = self.active_pane {
                    let input_type = TextInputType::ExternalRay {
                        pane_id,
                        include_orbit: *include_orbit,
                        select_landing_point: *select_landing_point,
                    };
                    self.prompt_text(input_type);
                }
            }
            Action::DrawRaysOfPeriod => {
                if let Some(pane_id) = self.active_pane {
                    let input_type = TextInputType::ActiveRays { pane_id };
                    self.prompt_text(input_type);
                }
            }
            Action::DrawEquipotential => {
                self.get_active_pane_mut().map(Pane::draw_equipotential);
            }
            Action::ClearRays => {
                self.get_active_pane_mut().map(Pane::clear_marked_rays);
            }
            Action::ClearEquipotentials => {
                self.get_active_pane_mut().map(Pane::clear_equipotentials);
            }
            Action::ClearCurves => {
                self.get_active_pane_mut().map(Pane::clear_curves);
            }
            Action::ResetSelection => match self.active_pane {
                Some(PaneID::Parent) => self.parent.reset_selection(),
                Some(PaneID::Child) => {
                    self.child.reset_selection();
                }
                None => {}
            },
            Action::ResetView => {
                self.get_active_pane_mut().map(Pane::reset);
            }
            Action::ToggleLiveMode => self.toggle_live_mode(),
            Action::CycleActivePlane => {
                self.parent_mut().cycle_active_plane();
                self.child_mut().cycle_active_plane();
            }
            Action::PromptImageHeight => {
                // TODO: Fill in with actual handling
            }
            Action::Pan(x, y) => {
                self.get_active_pane_mut().map(|p| p.pan_relative(*x, *y));
            }
            Action::Zoom(scale) => {
                self.get_active_pane_mut()
                    .map(|p| p.zoom(*scale, p.get_selection()));
            }
            Action::CenterOnSelection => {
                if let Some(pane) = self.get_active_pane_mut() {
                    let selection = pane.get_selection();
                    pane.grid_mut().recenter(selection);
                    pane.schedule_recompute();
                }
            }
            Action::ScaleMaxIter(factor) => {
                self.get_active_pane_mut()
                    .map(|p| p.scale_max_iter(*factor));
            }
            Action::RandomizePalette => self.randomize_palette(),
            Action::SetPalette(palette) => {
                self.set_palette(*palette);
            }
            Action::SetPaletteWhite => {
                let white_palette = Palette::white(32.);
                self.set_palette(white_palette);
            }
            Action::SetPaletteBlack => {
                let black_palette = Palette::black(32.);
                self.set_palette(black_palette);
            }
            Action::SetColoring(algorithm) => {
                self.get_active_pane_mut()
                    .map(|p| p.set_coloring_algorithm(algorithm.clone()));
            }
            Action::SetColoringInternalPotential => {
                self.get_active_pane_mut()
                    .map(Pane::select_preperiod_smooth_coloring);
            }
            Action::SetColoringPreperiodPeriod => {
                self.get_active_pane_mut()
                    .map(Pane::select_preperiod_coloring);
            }
            Action::SetColoringPotentialPeriod => {
                self.get_active_pane_mut()
                    .map(Pane::select_preperiod_period_smooth_coloring);
            }
            Action::ScalePalettePeriod(factor) => {
                self.get_active_pane_mut().map(|p| p.scale_palette(*factor));
            }
            Action::ShiftPalettePhase(phase) => {
                self.get_active_pane_mut().map(|p| p.shift_palette(*phase));
            }
        }
    }
}

/// A trait that extends `Interactive` with an update method for the UI.
pub trait Interface: Interactive
{
    /// Updates the state of the interface, handling input and rendering dialogs.
    fn update(&mut self, ui: &Context);
}

impl<T> Interface for T
where
    T: PanePair + Interactive,
{
    fn update(&mut self, ctx: &Context)
    {
        self.handle_input(ctx);
        self.show_dialog(ctx);
        self.update_panes();
    }
}
