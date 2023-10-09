use egui::{Context, CursorIcon, InputState, Ui};
use egui_extras::{Column, TableBuilder};
use egui_file::FileDialog;

use fractal_common::{
    coloring::{algorithms::InteriorColoringAlgorithm, palette::ColorPalette},
    types::{Cplx, ParamList},
};
use fractal_core::dynamics::{
    symbolic::{OrbitSchema, RationalAngle},
    ParameterPlane,
};

use crate::hotkeys::keyboard_shortcuts::*;
use crate::{
    actions::Action,
    dialog::{Dialog, StructuredTextDialog, TextDialogBuilder, TextInputType},
    hotkeys::{
        Hotkey, ANNOTATION_HOTKEYS, FILE_HOTKEYS, IMAGE_HOTKEYS, INCOLORING_HOTKEYS,
        PALETTE_HOTKEYS, SELECTION_HOTKEYS,
    },
    pane::{ChildTask, ComputeTask, Pane, WindowPane},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum UIMessage
{
    DoNothing,
    CloseWindow,
    Quit,
}
impl Default for UIMessage
{
    fn default() -> Self
    {
        Self::DoNothing
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PaneID
{
    Parent,
    Child,
}
impl std::fmt::Display for PaneID
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self
        {
            Self::Parent =>
            {
                if f.alternate()
                {
                    write!(f, "Parent")
                }
                else
                {
                    write!(f, "parent")
                }
            }
            Self::Child =>
            {
                if f.alternate()
                {
                    write!(f, "Child")
                }
                else
                {
                    write!(f, "child")
                }
            }
        }
    }
}

pub trait PanePair
{
    fn parent(&self) -> &dyn Pane;
    fn parent_mut(&mut self) -> &mut dyn Pane;
    fn child(&self) -> &dyn Pane;
    fn child_mut(&mut self) -> &mut dyn Pane;
    fn randomize_palette(&mut self);
    fn set_palette(&mut self, palette: ColorPalette);
    fn set_coloring_algorithm(&mut self, coloring_algorithm: InteriorColoringAlgorithm);

    fn get_pane(&self, pane_id: PaneID) -> &dyn Pane;
    fn get_pane_mut(&mut self, pane_id: PaneID) -> &mut dyn Pane;
    fn set_active_pane(&mut self, pane_id: Option<PaneID>);
    fn get_active_pane(&self) -> Option<&dyn Pane>;
    fn get_active_pane_mut(&mut self) -> Option<&mut dyn Pane>;
    fn prompt_save_active_pane(&mut self);
    fn prompt_save(&mut self, pane_id: PaneID);
    fn prompt_text(&mut self, input_type: TextInputType);
    fn get_image_height(&self) -> usize;
    fn change_height(&mut self, new_height: usize);

    // fn descend(self) -> Box<dyn PanePair>;
}

pub trait Interactive
{
    fn handle_mouse(&mut self, ctx: &Context);
    fn handle_input(&mut self, ctx: &Context);
    fn process_action(&mut self, action: &Action);

    fn toggle_live_mode(&mut self);
    fn show_dialog(&mut self, ctx: &Context);
    fn has_visible_dialog(&self) -> bool;
    fn update_panes(&mut self);
    fn show(&mut self, ui: &mut Ui);
    fn consume_click(&mut self);
    fn reset_click(&mut self);
    fn schedule_close(&mut self);
    fn schedule_quit(&mut self);
    fn get_message(&self) -> UIMessage;
    fn pop_message(&mut self) -> UIMessage;
    fn name(&self) -> String;
}

pub struct MainInterface<P, J>
where
    P: ParameterPlane + Clone + 'static,
    J: ParameterPlane + Clone + 'static,
{
    parent: WindowPane<P>,
    child: WindowPane<J>,
    image_height: usize,
    active_pane: Option<PaneID>,
    live_mode: bool,
    dialog: Option<Dialog>,
    // save_task: SaveTask,
    click_used: bool,
    pub message: UIMessage,
}

impl<P, J, C, M, T> MainInterface<P, J>
where
    P: ParameterPlane + Clone + 'static,
    J: ParameterPlane<MetaParam = M, Child = C> + Clone + 'static,
    C: ParameterPlane + From<J>,
    M: ParamList<Param = T>,
    T: From<P::Param> + std::fmt::Display,
{
    pub fn new(parent: P, child: J, image_height: usize) -> Self
    {
        Self {
            parent: parent.into(),
            child: child.into(),
            image_height,
            active_pane: Some(PaneID::Parent),
            live_mode: false,
            dialog: None,
            // save_task: SaveTask::Idle,
            click_used: false,
            message: UIMessage::default(),
        }
    }

    fn set_child_param(&mut self, point: Cplx, new_param: P::Param)
    {
        let mut new_bounds = self.parent.plane.default_julia_bounds(point, new_param);

        // Set the new center to equal the old center plus whatever deviation the user has created
        let old_center = self.child.grid().center();
        let old_default_center = self.child.plane.default_bounds().center();
        let offset = new_bounds.center() - old_default_center;
        let new_center = old_center + offset;

        if offset.is_finite()
        {
            new_bounds.zoom(self.child.zoom_factor, new_center);
            new_bounds.recenter(new_center);
            self.child.grid_mut().change_bounds(new_bounds);
            self.child.set_param(T::from(new_param));
        }
        else
        {
            // Reset child bounds to default
            self.child.grid_mut().change_bounds(new_bounds);
            self.child.set_param(T::from(new_param));
            self.child.grid_mut().resize_y(self.image_height);
            self.child.set_task(ComputeTask::Compute);
        }
    }

    #[inline]
    fn close_dialog(&mut self)
    {
        self.dialog = None;
    }

    fn handle_save_dialog(&mut self, pane_id: PaneID, file_dialog: &FileDialog)
    {
        // If file selection was confirmed
        if file_dialog.selected()
        {
            if let Some(path) = file_dialog.path()
            {
                let filename = path.to_string_lossy().into_owned();
                let image_width: usize = 4096; // You can make this dynamic as per your requirement

                let pane = self.get_pane_mut(pane_id);
                pane.save_image(image_width, &filename);
            }
            self.set_active_pane(None);
        }
    }

    fn process_text_dialog_input(&mut self, text_dialog: &StructuredTextDialog, text: String)
    {
        use crate::dialog::TextInputType::*;
        match text_dialog.input_type
        {
            ExternalRay { pane_id, follow } =>
            {
                if let Ok(angle) = text.parse::<RationalAngle>()
                {
                    let pane = self.get_pane_mut(pane_id);
                    pane.marking_mut().toggle_ray(angle);
                    pane.schedule_redraw();

                    if follow
                    {
                        pane.follow_ray_landing_point(angle);
                    }
                    else
                    {
                        pane.reset_ray_state();
                    }
                }
            }
            Coordinates { pane_id } =>
            {
                if let Ok(point) = text.parse::<Cplx>()
                {
                    let pane = self.get_pane_mut(pane_id);
                    pane.select_point(point);
                }
            }
            FindPeriodic { pane_id } =>
            {
                if let Ok(o) = text.parse::<OrbitSchema>()
                {
                    let pane = self.get_pane_mut(pane_id);
                    pane.select_nearby_point(o);
                }
            }
        }
    }
}

impl<P, J, C, M, T> PanePair for MainInterface<P, J>
where
    P: ParameterPlane + Clone,
    J: ParameterPlane<MetaParam = M, Child = C> + Clone + 'static,
    C: ParameterPlane + From<J>,
    M: ParamList<Param = T>,
    T: From<P::Param> + std::fmt::Display,
{
    fn parent(&self) -> &dyn Pane
    {
        &self.parent
    }
    fn parent_mut(&mut self) -> &mut dyn Pane
    {
        &mut self.parent
    }
    fn child(&self) -> &dyn Pane
    {
        &self.child
    }
    fn child_mut(&mut self) -> &mut dyn Pane
    {
        &mut self.child
    }
    fn randomize_palette(&mut self)
    {
        let palette = ColorPalette::new_random(0.45, 0.38);
        self.parent.change_palette(palette);
        self.child.change_palette(palette);
    }

    fn prompt_text(&mut self, input_type: TextInputType)
    {
        use TextInputType::*;
        let text_dialog = match input_type
        {
            ExternalRay { pane_id, .. } =>
            {
                let pane = self.get_pane(pane_id);
                let ray_type = if pane.is_dynamical()
                {
                    "dynamical"
                }
                else
                {
                    "parameter"
                };
                let prompt = format!(
                    concat!(
                        "Input an angle for {} ray on {}\n",
                        "Example formats: <15/56>, <110>, <p011>, <001p010>",
                    ),
                    ray_type,
                    pane.plane_name()
                );
                TextDialogBuilder::new(input_type)
                    .title("External ray angle input")
                    .prompt(&prompt)
                    .build()
            }
            FindPeriodic { pane_id } =>
            {
                let pane = self.get_pane(pane_id);
                let prompt = format!(
                    concat!(
                        "Input the period to find a nearby point on {}\n",
                        "Format: <period> or <preperiod, period>"
                    ),
                    pane.plane_name()
                );
                TextDialogBuilder::new(input_type)
                    .title("Find nearby point")
                    .prompt(&prompt)
                    .build()
            }
            Coordinates { pane_id } =>
            {
                let pane = self.get_pane(pane_id);
                let prompt = format!(
                    "Enter the coordinates of the point to select on {}",
                    pane.plane_name()
                );
                TextDialogBuilder::new(input_type)
                    .title("Input coordinates")
                    .prompt(&prompt)
                    .build()
            }
        };
        let dialog = Dialog::Text(text_dialog);
        self.dialog = Some(dialog);
    }

    fn prompt_save(&mut self, pane_id: PaneID)
    {
        let mut file_dialog = FileDialog::save_file(None)
            .default_filename(format!("{}.png", self.parent.name()))
            .show_rename(false)
            .show_new_folder(true);
        file_dialog.open();
        self.dialog = Some(Dialog::Save {
            pane_id,
            file_dialog,
        });
    }

    fn prompt_save_active_pane(&mut self)
    {
        if let Some(pane_id) = self.active_pane
        {
            self.prompt_save(pane_id);
        }
    }

    fn set_active_pane(&mut self, pane_id: Option<PaneID>)
    {
        self.active_pane = pane_id;
    }

    fn get_pane(&self, pane_id: PaneID) -> &dyn Pane
    {
        match pane_id
        {
            PaneID::Parent => self.parent(),
            PaneID::Child => self.child(),
        }
    }
    fn get_pane_mut(&mut self, pane_id: PaneID) -> &mut dyn Pane
    {
        match pane_id
        {
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

    fn set_palette(&mut self, palette: ColorPalette)
    {
        self.parent.change_palette(palette);
        self.child.change_palette(palette);
    }

    fn set_coloring_algorithm(&mut self, coloring_algorithm: InteriorColoringAlgorithm)
    {
        match coloring_algorithm
        {
            InteriorColoringAlgorithm::InternalPotential { .. } =>
            {
                self.parent_mut().select_preperiod_smooth_coloring();
                self.child_mut().select_preperiod_smooth_coloring();
            }
            InteriorColoringAlgorithm::PreperiodPeriodSmooth { .. } =>
            {
                self.parent_mut().select_preperiod_period_smooth_coloring();
                self.child_mut().select_preperiod_period_smooth_coloring();
            }
            _ =>
            {
                self.parent_mut()
                    .set_coloring_algorithm(coloring_algorithm.clone());
                self.child_mut().set_coloring_algorithm(coloring_algorithm);
            }
        }
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

    // fn descend(self) -> Box<dyn PanePair>
    // {
    //     let new_parent = self.child.plane;
    //     let new_child = C::from(new_parent.clone());
    //     Box::new(make_interface(new_parent, new_child))
    //     // Box::from(MainInterface::new(new_parent, new_child))
    // }
}

impl<P, J, C, M, T> Interactive for MainInterface<P, J>
where
    P: ParameterPlane + Clone,
    J: ParameterPlane<MetaParam = M, Child = C> + Clone + 'static,
    C: ParameterPlane + From<J>,
    M: ParamList<Param = T>,
    T: From<P::Param> + std::fmt::Display,
{
    fn handle_mouse(&mut self, ctx: &Context)
    {
        let clicked = ctx.input(|i| i.pointer.any_click()) && !self.click_used;
        let zoom_factor = ctx.input(InputState::zoom_delta);

        self.reset_click();

        let Some(pointer_pos) = ctx.pointer_latest_pos() else {return};

        if self.parent().frame_contains_pixel(pointer_pos)
        {
            ctx.set_cursor_icon(CursorIcon::Crosshair);
            self.set_active_pane(Some(PaneID::Parent));
            let reselect_point = self.live_mode || clicked;
            let pointer_value = self.parent().map_pixel(pointer_pos);
            self.parent_mut()
                .process_mouse_input(pointer_value, zoom_factor, reselect_point);
            match self.parent_mut().pop_child_task()
            {
                ChildTask::UpdateParam =>
                {
                    let parent_selection = self.parent.get_selection();
                    let new_child_param = self.parent.plane.param_map(parent_selection);
                    self.set_child_param(parent_selection, new_child_param);
                }
                _ =>
                {}
            }

            if clicked
            {
                self.consume_click();
                let param = self.parent.plane.param_map(pointer_value);
                let start = self.parent.plane.start_point(pointer_value, param);
                self.child_mut().mark_orbit_and_info(start.into());
            }
        }
        else if self.child().frame_contains_pixel(pointer_pos)
        {
            ctx.set_cursor_icon(CursorIcon::Crosshair);
            self.set_active_pane(Some(PaneID::Child));
            let pointer_value = self.child().map_pixel(pointer_pos);
            self.child_mut()
                .process_mouse_input(pointer_value, zoom_factor, clicked);

            if clicked
            {
                self.consume_click();
                self.child_mut().mark_orbit_and_info(pointer_value);
            }
        }
        else
        {
            ctx.set_cursor_icon(CursorIcon::Default);
        }
    }

    fn toggle_live_mode(&mut self)
    {
        self.live_mode ^= true;
        if self.live_mode
        {
            self.parent.reset_ray_state();
        }
    }

    fn update_panes(&mut self)
    {
        self.parent.process_task();
        self.child.process_task();
    }

    fn show_dialog(&mut self, ctx: &Context)
    {
        let dialog = self.dialog.take();

        if let Some(mut dialog) = dialog
        {
            dialog.show(ctx);

            match &mut dialog
            {
                Dialog::Save {
                    pane_id,
                    file_dialog,
                } => self.handle_save_dialog(*pane_id, file_dialog),
                Dialog::Text(text_dialog) =>
                {
                    if let crate::dialog::Response::Complete { text } = text_dialog.get_response()
                    {
                        self.process_text_dialog_input(text_dialog, text);
                    }
                }
            }

            if dialog.visible()
            {
                self.dialog = Some(dialog);
            }
        }
    }

    fn has_visible_dialog(&self) -> bool
    {
        self.dialog.as_ref().map(|x| x.visible()).unwrap_or(false)
    }

    fn handle_input(&mut self, ctx: &Context)
    {
        // Don't process input if the user is in a dialog
        if self.has_visible_dialog()
        {
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
            shortcut.as_ref().map(|s| {
                if shortcut_used!(ctx, s)
                {
                    self.process_action(action);
                }
            });
        }
        self.handle_mouse(ctx);
    }

    fn process_action(&mut self, action: &Action)
    {
        match action
        {
            Action::Quit => self.schedule_quit(),
            Action::Close => self.schedule_close(),
            Action::SaveImage(pane_id) => self.prompt_save(*pane_id),
            Action::SaveActiveImage => self.prompt_save_active_pane(),
            Action::ToggleSelectionMarker =>
            {
                if let Some(pane) = self.get_active_pane_mut()
                {
                    pane.marking_mut().toggle_selection();
                    pane.schedule_redraw();
                }
            }
            Action::ToggleCritical(pane_id) =>
            {
                let pane = self.get_pane_mut(*pane_id);
                pane.marking_mut().toggle_critical();
                pane.schedule_redraw();
            }
            Action::ToggleCycles(pane_id, period) =>
            {
                let pane = self.get_pane_mut(*pane_id);
                pane.marking_mut().toggle_cycles_of_period(*period);
                pane.schedule_redraw();
            }
            Action::FindPeriodicPoint =>
            {
                if let Some(pane_id) = self.active_pane
                {
                    let input_type = TextInputType::FindPeriodic { pane_id };
                    self.prompt_text(input_type);
                }
            }
            Action::MapSelection =>
            {
                let plane = self.child_mut();
                plane.map_selection();
            }
            Action::DrawOrbit =>
            {
                let plane = self.child_mut();
                let selection = plane.get_selection();
                plane.mark_orbit_and_info(selection);
            }
            Action::ClearOrbit =>
            {
                if let Some(pane) = self.get_active_pane_mut()
                {
                    pane.clear_marked_orbit();
                }
            }
            Action::DrawExternalRay {
                select_landing_point,
            } =>
            {
                if let Some(pane_id) = self.active_pane
                {
                    let input_type = TextInputType::ExternalRay {
                        pane_id,
                        follow: *select_landing_point,
                    };
                    self.prompt_text(input_type);
                }
            }
            Action::DrawEquipotential =>
            {
                self.get_active_pane_mut().map(|p| p.draw_equipotential());
            }
            Action::ClearRays =>
            {
                self.get_active_pane_mut().map(|p| p.clear_marked_rays());
            }
            Action::ClearEquipotentials =>
            {
                self.get_active_pane_mut().map(|p| p.clear_equipotentials());
            }
            Action::ClearCurves =>
            {
                self.get_active_pane_mut().map(|p| p.clear_curves());
            }
            Action::ResetSelection => match self.active_pane
            {
                Some(PaneID::Parent) => self.parent.reset_selection(),
                Some(PaneID::Child) =>
                {
                    self.child.reset_selection();
                    self.child.clear_marked_orbit();
                }
                None =>
                {}
            },
            Action::ToggleLiveMode => self.toggle_live_mode(),
            Action::CycleActivePlane =>
            {
                self.parent_mut().cycle_active_plane();
                self.child_mut().cycle_active_plane();
            }
            Action::PromptImageHeight =>
            {
                // TODO: Fill in with actual handling
            }
            Action::Pan(x, y) =>
            {
                self.get_active_pane_mut().map(|p| p.pan_relative(*x, *y));
            }
            Action::Zoom(scale) =>
            {
                self.get_active_pane_mut()
                    .map(|p| p.zoom(*scale, p.get_selection()));
            }
            Action::CenterOnSelection =>
            {
                if let Some(pane) = self.get_active_pane_mut()
                {
                    let selection = pane.get_selection();
                    pane.grid_mut().recenter(selection);
                    pane.schedule_recompute();
                }
            }
            Action::ScaleMaxIter(factor) =>
            {
                self.get_active_pane_mut()
                    .map(|p| p.scale_max_iter(*factor));
            }
            Action::RandomizePalette => self.randomize_palette(),
            Action::SetPalette(palette) =>
            {
                self.set_palette(*palette);
            }
            Action::SetPaletteWhite =>
            {
                let white_palette = ColorPalette::white(32.);
                self.set_palette(white_palette);
            }
            Action::SetPaletteBlack =>
            {
                let black_palette = ColorPalette::black(32.);
                self.set_palette(black_palette);
            }
            Action::SetColoring(algorithm) =>
            {
                self.get_active_pane_mut()
                    .map(|p| p.set_coloring_algorithm(algorithm.clone()));
            }
            Action::ScalePalettePeriod(factor) =>
            {
                self.get_active_pane_mut().map(|p| p.scale_palette(*factor));
            }
            Action::ShiftPalettePhase(phase) =>
            {
                self.get_active_pane_mut().map(|p| p.shift_palette(*phase));
            }
        }
    }

    fn show(&mut self, ui: &mut Ui)
    {
        TableBuilder::new(ui)
            .column(Column::auto().resizable(true))
            .column(Column::remainder())
            .vscroll(false)
            .stick_to_bottom(true)
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.heading(self.parent.name());
                });
                header.col(|ui| {
                    ui.heading(self.child.name());
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
                        let selection_desc = self.parent.describe_selection();
                        let orbit_desc = self.parent.describe_orbit_info();
                        ui.label(selection_desc);
                        ui.label(orbit_desc);
                    });
                    row.col(|ui| {
                        let selection_desc = self.child.describe_selection();
                        let orbit_desc = self.child.describe_orbit_info();
                        ui.label(selection_desc);
                        ui.label(orbit_desc);
                    });
                });
            });
    }

    fn schedule_close(&mut self)
    {
        self.message = UIMessage::CloseWindow;
    }

    fn schedule_quit(&mut self)
    {
        self.message = UIMessage::Quit;
    }

    #[inline]
    fn get_message(&self) -> UIMessage
    {
        self.message
    }

    #[inline]
    fn pop_message(&mut self) -> UIMessage
    {
        let msg = self.get_message();
        self.message = UIMessage::DoNothing;
        msg
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
}

pub trait Interface: PanePair + Interactive {}

impl<T> Interface for T where T: PanePair + Interactive {}
