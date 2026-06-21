use dynamo_color::{IncoloringAlgorithm, Palette};
use dynamo_common::prelude::*;
use dynamo_core::dynamics::Displayable;
use dynamo_core::prelude::HasChild;
use egui::{Context, CursorIcon, InputState, Ui};
use egui_extras::{Column, TableBuilder};
use egui_file::FileDialog;
use log::debug;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::actions::Action;
use crate::dialog::{
    AllActiveRayParams, Dialog, RayParams, SaveFileType, TextDialogBuilder, TextInputType,
    ToggleKey, ToggleMap,
};
use crate::hotkeys::keyboard_shortcuts::shortcut_used;
use crate::hotkeys::{
    ANNOTATION_HOTKEYS, CYCLES_HOTKEYS, FILE_HOTKEYS, Hotkey, IMAGE_HOTKEYS, INCOLORING_HOTKEYS,
    OUTCOLORING_HOTKEYS, PALETTE_HOTKEYS, SELECTION_HOTKEYS,
};
use crate::pane::id::{PaneID, PaneSelection};
use crate::pane::tasks::{ChildTask, FollowState, SelectOrFollow};
use crate::pane::{Pane, WindowPane};

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
    fn update_panes(&mut self, ctx: &Context);

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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MainInterface<P, J>
where
    P: Displayable + Clone + 'static,
    J: Displayable + Clone + 'static,
{
    parent:       WindowPane<P>,
    child:        WindowPane<J>,
    image_height: usize,
    active_pane:  Option<PaneID>,
    live_mode:    bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    dialog:       Option<Dialog>,
    // save_task: SaveTask,
    click_used:   bool,
    /// Target image height (logical px) awaiting the resize debounce, set
    /// when the available pane area implies a height differing from the
    /// current one. Flushed by [`flush_pending_resize`] once stable.
    #[cfg_attr(feature = "serde", serde(skip))]
    pending_height: Option<usize>,
    /// egui input-clock time (seconds) at which a stable [`pending_height`]
    /// should be applied. Refreshed on every observed size change so a
    /// recompute fires only after the drag settles.
    #[cfg_attr(feature = "serde", serde(skip))]
    resize_deadline: Option<f64>,
    pub message:  UiMessage,
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
            pending_height: None,
            resize_deadline: None,
            message: UiMessage::default(),
        }
    }

    /// Smallest auto-fit image height (logical px); below this the panes
    /// are too small to be useful and recompute is skipped.
    const MIN_FIT_HEIGHT: usize = 200;
    /// Largest auto-fit image height (logical px). Caps compute cost so
    /// maximizing on a high-DPI or very large display cannot trigger a
    /// multi-megapixel fractal recompute; beyond this the pane letterboxes.
    const MAX_FIT_HEIGHT: usize = 1440;
    /// Seconds the observed size must stay stable before a recompute fires,
    /// so dragging a window/splitter does not thrash the CPU.
    const RESIZE_DEBOUNCE: f64 = 0.2;
    /// Minimum height delta (logical px) that counts as a real resize,
    /// avoiding oscillation from sub-pixel layout jitter.
    const RESIZE_HYSTERESIS: usize = 2;
    /// Header row height in `show`'s table (points).
    const HEADER_H: f32 = 20.0;
    /// State-info row height in `show`'s table (points).
    const STATE_ROW_H: f32 = 80.0;

    /// Compute the shared image height that fits both planes' images
    /// within `avail` (logical px) given their bounds aspect ratios.
    ///
    /// The two planes share one height; each plane's width is its bounds
    /// aspect times that height. Returns the largest height (clamped to
    /// `[MIN_FIT_HEIGHT, MAX_FIT_HEIGHT]`) for which neither plane's image
    /// overflows its column, so the images fill the area without clipping.
    fn fit_height(&self, avail: egui::Vec2) -> usize
    {
        // The image row is the available height less the header and the
        // state-info row baked into `show`'s table.
        let row_h = (avail.y - Self::HEADER_H - Self::STATE_ROW_H).max(0.0);
        // The two planes share the width roughly evenly (parent `exact`,
        // child `remainder`); fit each into its half.
        let col_w = (avail.x * 0.5).max(0.0);

        let height_from = |aspect: f32| -> f32 {
            // image width = aspect * height must fit col_w; height must fit
            // row_h. Take the binding constraint.
            if aspect <= 0.0 {
                row_h
            } else {
                row_h.min(col_w / aspect)
            }
        };

        let aspect_of = |g: &PointGrid| -> f32 {
            (g.bounds.range_x() / g.bounds.range_y()) as f32
        };
        let parent_aspect = aspect_of(self.parent.grid());
        let child_aspect = aspect_of(self.child.grid());
        let fit = height_from(parent_aspect).min(height_from(child_aspect));

        (fit as usize).clamp(Self::MIN_FIT_HEIGHT, Self::MAX_FIT_HEIGHT)
    }

    /// Observe the available area each frame and, when it implies a height
    /// differing from the current one, arm a debounced recompute. Records
    /// only the target here; [`flush_pending_resize`] applies it once the
    /// size has been stable for [`RESIZE_DEBOUNCE`] seconds.
    fn observe_available(&mut self, ctx: &Context, avail: egui::Vec2)
    {
        // Fit in logical points: egui draws the texture at its size in
        // points and applies `pixels_per_point` itself, so the grid
        // resolution must match the point size (multiplying by ppp here
        // would render the image twice as large as the pane on HiDPI).
        let target = self.fit_height(avail);

        if target.abs_diff(self.image_height) < Self::RESIZE_HYSTERESIS {
            // Already at the fitting size; drop any pending resize.
            self.pending_height = None;
            self.resize_deadline = None;
            return;
        }
        // Only (re)arm the debounce when the target itself changes. While
        // dragging, the target shifts every frame and keeps pushing the
        // deadline out (no compute mid-drag); once it settles, the
        // deadline stops moving and is allowed to elapse so the flush can
        // fire.
        if self.pending_height != Some(target) {
            self.pending_height = Some(target);
            self.resize_deadline = Some(ctx.input(|i| i.time) + Self::RESIZE_DEBOUNCE);
        }
        // Keep the frame loop alive so the flush fires without further
        // input once the deadline passes.
        ctx.request_repaint_after(std::time::Duration::from_secs_f64(Self::RESIZE_DEBOUNCE));
    }

    /// Apply a debounced resize once its deadline has passed, recomputing
    /// both planes at the new height.
    fn flush_pending_resize(&mut self, ctx: &Context)
    {
        let Some(deadline) = self.resize_deadline else {
            return;
        };
        if ctx.input(|i| i.time) < deadline {
            return;
        }
        self.resize_deadline = None;
        if let Some(target) = self.pending_height.take()
            && target.abs_diff(self.image_height) >= Self::RESIZE_HYSTERESIS
        {
            self.change_height(target);
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

    fn handle_save_dialog(
        &mut self,
        selection: PaneSelection,
        file_dialog: &FileDialog,
        file_type: SaveFileType,
    )
    {
        use SaveFileType::{Image, Palette};

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
                for pane_id in pane_ids {
                    self.get_pane_mut(pane_id).save_image(image_width, path);
                }
            }
            Palette => {
                for pane_id in pane_ids {
                    self.get_pane_mut(pane_id).save_palette(path);
                }
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
        use crate::dialog::TextInputType::{ActiveRays, Coordinates, ExternalRay, FindPeriodic};
        match input_type {
            ExternalRay { .. } => self.process_external_ray_input(text, toggle_map),
            ActiveRays { pane_id } => self.process_active_rays_input(text, toggle_map, pane_id),
            Coordinates { pane_id } => self.process_coordinates_input(text, pane_id),
            FindPeriodic { pane_id } => self.process_find_periodic_input(text, toggle_map, pane_id),
        }
    }

    fn process_external_ray_input(&mut self, text: &str, toggle_map: &ToggleMap)
    {
        use crate::dialog::ToggleKey::{DoChild, DoParent, DrawOrbit};

        let Ok(angle) = text.parse::<RationalAngle>() else {
            return;
        };
        let ray_params = RayParams {
            do_parent:     toggle_map.get(DoParent),
            do_child:      toggle_map.get(DoChild),
            angle_info:    angle.with_degree(self.child.degree()).to_angle_info(),
            follow_task:   Self::follow_task(toggle_map),
            include_orbit: toggle_map.get(DrawOrbit),
        };
        self.dialog = Some(Dialog::confirm_ray(ray_params));
    }

    fn process_active_rays_input(&mut self, text: &str, toggle_map: &ToggleMap, pane_id: PaneID)
    {
        use crate::dialog::ToggleKey::{DoChild, DoParent, PrefixAngles};

        let Ok(orbit_schema) = text.parse::<OrbitSchema>() else {
            return;
        };
        let include_suffixes = toggle_map.get(PrefixAngles);
        let orbit_schema = orbit_schema.with_degree(self.get_pane(pane_id).degree());
        let params = AllActiveRayParams {
            do_parent: toggle_map.get(DoParent),
            do_child: toggle_map.get(DoChild),
            active_angles: orbit_schema.active_angles(include_suffixes),
            orbit_schema,
            include_suffixes,
        };
        self.dialog = Some(Dialog::confirm_active_rays(params));
    }

    fn process_coordinates_input(&mut self, text: &str, pane_id: PaneID)
    {
        let Ok(point) = text.parse::<Cplx>() else {
            return;
        };
        let pane = self.get_pane_mut(pane_id);
        pane.select_point(point);
        pane.stop_following();
        self.process_child_task();
    }

    fn process_find_periodic_input(&mut self, text: &str, toggle_map: &ToggleMap, pane_id: PaneID)
    {
        use crate::dialog::ToggleKey::FollowPoint;

        let Ok(orbit_schema) = text.parse::<OrbitSchema>() else {
            return;
        };
        let follow = toggle_map.get(FollowPoint);
        match pane_id {
            PaneID::Child => self
                .child_mut()
                .set_follow_state(FollowState::SelectPeriodic {
                    orbit_schema,
                    follow,
                }),
            PaneID::Parent => {
                if self.parent_mut().select_nearby_point(orbit_schema).is_ok() {
                    self.process_child_task();
                }
            }
        }
    }

    fn follow_task(toggle_map: &ToggleMap) -> SelectOrFollow
    {
        use crate::dialog::ToggleKey::{FollowPoint, SelectPoint};

        if toggle_map.get(FollowPoint) {
            return SelectOrFollow::Follow;
        }
        if toggle_map.get(SelectPoint) {
            return SelectOrFollow::Select;
        }
        SelectOrFollow::DoNothing
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
    #[expect(
        clippy::float_cmp,
        reason = "egui uses an exact neutral zoom factor of 1.0 for no zoom input"
    )]
    fn handle_mouse(&mut self, ctx: &Context)
    {
        let clicked = ctx.input(|i| i.pointer.any_click()) && !self.click_used;
        let zoom_factor = ctx.input(InputState::zoom_delta);

        if zoom_factor != 1. {
            debug!("Mouse zoom: factor={zoom_factor}");
        }
        if clicked {
            debug!("Mouse clicked");
        }

        self.reset_click();

        let Some(pointer_pos) = ctx.pointer_latest_pos() else {
            return;
        };

        if ctx.input(|i| i.pointer.is_decidedly_dragging())
            && let Some(origin) = ctx.input(|i| i.pointer.press_origin())
        {
            let delta = ctx.input(|i| i.pointer.delta());
            if self.parent().frame_contains_pixel(origin) {
                let offset = self.parent.grid().map_vec2((delta).into());
                self.parent.pan(-offset);
            } else if self.child().frame_contains_pixel(origin) {
                let offset = self.child.grid().map_vec2((delta).into());
                self.child.pan(-offset);
            }
        }

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
    const fn schedule_close(&mut self)
    {
        self.message = UiMessage::CloseWindow;
    }

    /// Schedules a message to quit the application.
    const fn schedule_quit(&mut self)
    {
        self.message = UiMessage::Quit;
    }

    /// Schedules a message to open a new tab.
    const fn schedule_new_tab(&mut self)
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

    fn build_text_dialog(&self, input_type: TextInputType) -> crate::dialog::StructuredTextDialog
    {
        use TextInputType::{ActiveRays, Coordinates, ExternalRay, FindPeriodic};

        match input_type {
            ExternalRay {
                pane_id,
                include_orbit,
                select_landing_point,
            } => Self::build_external_ray_dialog(
                input_type,
                pane_id,
                include_orbit,
                select_landing_point,
            ),
            ActiveRays { pane_id } => Self::build_active_rays_dialog(input_type, pane_id),
            FindPeriodic { pane_id } => self.build_find_periodic_dialog(input_type, pane_id),
            Coordinates { pane_id } => self.build_coordinates_dialog(input_type, pane_id),
        }
    }

    fn build_external_ray_dialog(
        input_type: TextInputType,
        pane_id: PaneID,
        include_orbit: bool,
        select_landing_point: bool,
    ) -> crate::dialog::StructuredTextDialog
    {
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
            return builder
                .add_cond_toggle(ToggleKey::FollowPoint, "Follow landing point".to_owned())
                .build();
        }
        builder.build()
    }

    fn build_active_rays_dialog(
        input_type: TextInputType,
        pane_id: PaneID,
    ) -> crate::dialog::StructuredTextDialog
    {
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

    fn build_find_periodic_dialog(
        &self,
        input_type: TextInputType,
        pane_id: PaneID,
    ) -> crate::dialog::StructuredTextDialog
    {
        let prompt = format!(
            concat!(
                "Input the period to find a nearby point on {pane_name}.\n",
                "Format: <period> or <preperiod, period>"
            ),
            pane_name = self.get_pane(pane_id).name()
        );
        let builder = TextDialogBuilder::new(input_type)
            .title("Find nearby point")
            .prompt(prompt);
        if matches!(pane_id, PaneID::Child) {
            return builder
                .add_toggle_with_default(ToggleKey::FollowPoint, "Follow point".to_owned(), true)
                .build();
        }
        builder.build()
    }

    fn build_coordinates_dialog(
        &self,
        input_type: TextInputType,
        pane_id: PaneID,
    ) -> crate::dialog::StructuredTextDialog
    {
        let prompt = format!(
            "Enter the coordinates of the point to select on {}",
            self.get_pane(pane_id).name()
        );
        TextDialogBuilder::new(input_type)
            .title("Input coordinates")
            .prompt(prompt)
            .build()
    }

    fn process_window_action(&mut self, action: &Action) -> bool
    {
        match action {
            Action::Quit => self.schedule_quit(),
            Action::Close => self.schedule_close(),
            Action::NewTab => self.schedule_new_tab(),
            Action::SaveImage(panes) => self.prompt_save_image(*panes),
            Action::SavePalette(panes) => self.prompt_save_palette(*panes),
            Action::LoadPalette(panes) => self.prompt_load_palette(*panes),
            _ => return false,
        }
        true
    }

    fn process_annotation_action(&mut self, action: &Action) -> bool
    {
        match action {
            Action::ToggleSelectionMarker => {
                if let Some(pane) = self.get_active_pane_mut() {
                    pane.marking_mut().toggle_selection();
                    pane.schedule_redraw();
                }
            }
            Action::ToggleCritical => {
                let pane = self.child_mut();
                pane.marking_mut().toggle_critical();
                pane.schedule_redraw();
            }
            Action::ToggleMarked(selection) => {
                self.get_selected_pane_ids(*selection)
                    .into_iter()
                    .for_each(|pane_id| {
                        let pane = self.get_pane_mut(pane_id);
                        pane.marking_mut().toggle_misc_marked();
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
            _ => return false,
        }
        true
    }

    fn process_dynamics_action(&mut self, action: &Action) -> bool
    {
        match action {
            Action::FindPeriodicPoint => {
                self.prompt_for_active_pane(|pane_id| TextInputType::FindPeriodic { pane_id });
            }
            Action::EnterCoordinates => {
                self.prompt_for_active_pane(|pane_id| TextInputType::Coordinates { pane_id });
            }
            Action::MapSelection => {
                let plane = self.child_mut();
                plane.map_selection();
                plane.marking_mut().enable_selection();
            }
            Action::DrawOrbit => self.child_mut().tasks_mut().orbit.enable(),
            Action::StopFollowing => self.child_mut().stop_following(),
            Action::ClearOrbit => self.child_mut().clear_marked_orbit(),
            Action::DrawExternalRay {
                include_orbit,
                select_landing_point,
            } => {
                self.prompt_for_active_pane(|pane_id| TextInputType::ExternalRay {
                    pane_id,
                    include_orbit: *include_orbit,
                    select_landing_point: *select_landing_point,
                });
            }
            Action::DrawRaysOfPeriod => {
                self.prompt_for_active_pane(|pane_id| TextInputType::ActiveRays { pane_id });
            }
            Action::DrawContour(contour_type) => {
                if let Some(pane) = self.get_active_pane_mut() {
                    pane.draw_contour(*contour_type);
                }
            }
            Action::DrawAuxContours => {
                if let Some(pane) = self.get_active_pane_mut() {
                    pane.draw_aux_contours();
                }
            }
            Action::ClearRays => {
                if let Some(pane) = self.get_active_pane_mut() {
                    pane.clear_marked_rays();
                }
            }
            Action::ClearEquipotentials => {
                if let Some(pane) = self.get_active_pane_mut() {
                    pane.clear_equipotentials();
                }
            }
            Action::ClearCurves => {
                if let Some(pane) = self.get_active_pane_mut() {
                    pane.clear_curves();
                }
            }
            Action::ResetSelection => self.reset_active_selection(),
            Action::ResetView => {
                if let Some(pane) = self.get_active_pane_mut() {
                    pane.reset();
                }
            }
            _ => return false,
        }
        true
    }

    fn process_view_action(&mut self, action: &Action) -> bool
    {
        match action {
            Action::ToggleLiveMode => self.toggle_live_mode(),
            Action::CycleActivePlane => {
                self.parent_mut().cycle_active_plane();
                self.child_mut().cycle_active_plane();
            }
            Action::PromptImageHeight => {}
            Action::Pan(x, y) => {
                if let Some(pane) = self.get_active_pane_mut() {
                    pane.pan_relative(*x, *y);
                }
            }
            Action::Zoom(scale) => {
                if let Some(pane) = self.get_active_pane_mut() {
                    pane.zoom(*scale, pane.get_selection());
                }
            }
            Action::CenterOnSelection => {
                if let Some(pane) = self.get_active_pane_mut() {
                    let selection = pane.get_selection();
                    pane.grid_mut().recenter(selection);
                    pane.schedule_recompute();
                }
            }
            Action::ScaleMaxIter(factor) => {
                if let Some(pane) = self.get_active_pane_mut() {
                    pane.scale_max_iter(*factor);
                }
            }
            _ => return false,
        }
        true
    }

    fn process_coloring_action(&mut self, action: &Action) -> bool
    {
        match action {
            Action::RandomizePalette => self.randomize_palette(),
            Action::SetPalette(palette) => self.set_palette(*palette),
            Action::SetPaletteWhite => self.set_palette(Palette::white(16.)),
            Action::SetPaletteBlack => self.set_palette(Palette::black(16.)),
            Action::SetColoring(algorithm) => {
                if let Some(pane) = self.get_active_pane_mut() {
                    pane.set_coloring_algorithm(algorithm.clone());
                }
            }
            Action::SetColoringInternalPotential => {
                if let Some(pane) = self.get_active_pane_mut() {
                    pane.select_preperiod_smooth_coloring();
                }
            }
            Action::SetColoringPreperiodPeriod => {
                if let Some(pane) = self.get_active_pane_mut() {
                    pane.select_preperiod_coloring();
                }
            }
            Action::SetColoringPotentialPeriod => {
                if let Some(pane) = self.get_active_pane_mut() {
                    pane.select_preperiod_period_smooth_coloring();
                }
            }
            Action::ScalePalettePeriod(factor) => {
                if let Some(pane) = self.get_active_pane_mut() {
                    pane.scale_palette(*factor);
                }
            }
            Action::ShiftPalettePhase(phase) => {
                if let Some(pane) = self.get_active_pane_mut() {
                    pane.shift_palette(*phase);
                }
            }
            Action::ToggleEscapePhaseColoring => {
                if let Some(pane) = self.get_active_pane_mut() {
                    pane.get_coloring_mut().toggle_escape_phase_coloring();
                    pane.schedule_redraw();
                }
            }
            Action::CycleComputeMode(selection, change) => {
                self.get_selected_pane_ids(*selection)
                    .into_iter()
                    .for_each(|pane_id| self.get_pane_mut(pane_id).change_compute_mode(*change));
            }
            _ => return false,
        }
        true
    }

    fn prompt_for_active_pane<F>(&mut self, make_input_type: F)
    where
        F: FnOnce(PaneID) -> TextInputType,
    {
        if let Some(pane_id) = self.active_pane {
            self.prompt_text(make_input_type(pane_id));
        }
    }

    fn reset_active_selection(&mut self)
    {
        match self.active_pane {
            Some(PaneID::Parent) => self.parent.reset_selection(),
            Some(PaneID::Child) => self.child.reset_selection(),
            None => {}
        }
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
        self.dialog = Some(Dialog::Text(self.build_text_dialog(input_type)));
    }

    /// Open a dialog prompt to save an image.
    fn prompt_save_image(&mut self, pane_selection: PaneSelection)
    {
        let mut file_dialog = FileDialog::save_file()
            .initial_path(images_dir().unwrap_or_default())
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
        let mut file_dialog = FileDialog::save_file()
            .initial_path(palettes_dir().unwrap_or_default())
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
        let mut file_dialog = FileDialog::open_file()
            .initial_path(palettes_dir().unwrap_or_default())
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
        use PaneSelection::{ActivePane, BothPanes, Id};
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

    fn update_panes(&mut self, ctx: &Context)
    {
        self.flush_pending_resize(ctx);
        let parent_progress = self.parent.process_tasks();
        let child_progress = self.child.process_tasks();
        if parent_progress.busy || child_progress.busy {
            // Keep the frame loop alive so streamed tiles keep draining.
            ctx.request_repaint();
        }
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
            shortcut,
            action,
            bonus_action,
            ..
        } in FILE_HOTKEYS
            .iter()
            .chain(IMAGE_HOTKEYS.iter())
            .chain(ANNOTATION_HOTKEYS.iter())
            .chain(CYCLES_HOTKEYS.iter())
            .chain(SELECTION_HOTKEYS.iter())
            .chain(INCOLORING_HOTKEYS.iter())
            .chain(OUTCOLORING_HOTKEYS.iter())
            .chain(PALETTE_HOTKEYS.iter())
        {
            if let Some(s) = shortcut.as_ref()
                && shortcut_used!(ctx, s)
            {
                debug!("Keyboard shortcut triggered: {s:?}");
                self.process_action(action);
                if let Some(bonus_action) = bonus_action {
                    self.process_action(bonus_action);
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
        // Observe the area available to the panes and arm a debounced
        // recompute so the fractal fills the pane (and reflows when the
        // window/iframe or profile aspect changes).
        let avail = ui.available_size();
        self.observe_available(ui.ctx(), avail);

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

    /// Processes an action and updates the state of the interface accordingly.
    fn process_action(&mut self, action: &Action)
    {
        debug!("Processing action: {action:?}");
        if self.process_window_action(action)
            || self.process_annotation_action(action)
            || self.process_dynamics_action(action)
            || self.process_view_action(action)
            || self.process_coloring_action(action)
        {
            return;
        }

        unreachable!("unhandled action: {action:?}");
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
        self.update_panes(ctx);
    }
}
