use super::image_frame::ImageFrame;
use fractal_common::coloring::{algorithms::ColoringAlgorithm, palette::ColorPalette, Coloring};
use fractal_common::iter_plane::{FractalImage, IterPlane};
use fractal_common::point_grid::{Bounds, PointGrid};
use fractal_common::types::param_stack::Summarize;
use fractal_common::types::{ComplexVec, Cplx, OrbitInfo, ParamList, Real};
use fractal_core::dynamics::ParameterPlane;
use input_macro::input;
use seq_macro::seq;

use super::keyboard_shortcuts::*;
use super::marked_points::MarkingMode;

use egui::{Color32, Context, CursorIcon, InputState, Pos2, Rect, Stroke, Ui};
use egui_extras::{Column, RetainedImage, TableBuilder};
use egui_file::FileDialog;
use epaint::{CircleShape, PathShape};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub type ColoredPoint = (Cplx, Color32);
pub type ColoredPoints = Vec<ColoredPoint>;
pub type ColoredCurve = (Vec<Cplx>, Color32);

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ComputeTask
{
    DoNothing,
    Redraw,
    Recompute,
    Compute,
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ResizeTask
{
    DoNothing,
    ShowDialog,
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PaneID
{
    Parent,
    Child,
}

pub trait Pane
{
    fn get_task(&self) -> &ComputeTask;
    fn set_task(&mut self, new_task: ComputeTask);

    fn get_frame(&self) -> &ImageFrame;
    fn get_frame_mut(&mut self) -> &mut ImageFrame;

    fn get_coloring(&self) -> &Coloring;
    fn get_coloring_mut(&mut self) -> &mut Coloring;

    fn select_point(&mut self, point: Cplx);
    fn get_selection(&self) -> Cplx;
    fn reset_selection(&mut self);

    fn get_marked_curves(&self) -> &Vec<ColoredCurve>;
    fn get_marked_curves_mut(&mut self) -> &mut Vec<ColoredCurve>;

    fn get_image_frame(&self) -> &ImageFrame;
    fn get_image_frame_mut(&mut self) -> &mut ImageFrame;

    fn mark_curve(&mut self, zs: ComplexVec, color: Color32)
    {
        let curves = self.get_marked_curves_mut();
        curves.push((zs, color));
    }

    fn clear_marked_curves(&mut self)
    {
        let curves = self.get_marked_curves_mut();
        *curves = vec![];
    }

    fn put_marked_curves(&self, ui: &mut Ui)
    {
        let frame = self.get_frame();
        let grid = self.grid();
        let painter = ui.painter().with_clip_rect(frame.region);
        for (zs, color) in self.get_marked_curves().iter()
        {
            let points = zs
                .iter()
                .map(|z| {
                    let pt = grid.locate_point(*z);
                    frame.to_global_coords(pt.to_vec2())
                })
                .collect();
            let stroke = Stroke::new(1.0, *color);
            let path = PathShape::line(points, stroke);
            painter.add(path);
        }
    }

    fn get_marked_points(&self) -> &ColoredPoints;
    fn get_marked_points_mut(&mut self) -> &mut ColoredPoints;

    fn mark_point(&mut self, z: Cplx, color: Color32)
    {
        let points = self.get_marked_points_mut();
        points.push((z, color));
    }

    fn clear_marked_points(&mut self)
    {
        let points = self.get_marked_points_mut();
        *points = vec![];
    }

    fn put_marked_points(&self, ui: &mut Ui)
    {
        let frame = self.get_frame();
        let grid = self.grid();
        let painter = ui.painter().with_clip_rect(frame.region);
        for (z, color) in self.get_marked_points().iter()
        {
            let point = frame.to_global_coords(grid.locate_point(*z).to_vec2());
            let patch = CircleShape::filled(point, 4., *color);
            painter.add(patch);
        }
    }

    fn name(&self) -> String;

    fn grid(&self) -> &PointGrid;

    fn grid_mut(&mut self) -> &mut PointGrid;

    fn rescale(&mut self, new_bounds: Bounds)
    {
        self.grid_mut().change_bounds(new_bounds);
        self.schedule_compute();
    }

    fn schedule_compute(&mut self)
    {
        self.set_task(ComputeTask::Compute);
    }

    fn schedule_recompute(&mut self)
    {
        if !matches!(self.get_task(), ComputeTask::Compute)
        {
            self.set_task(ComputeTask::Recompute);
        }
    }

    fn schedule_redraw(&mut self)
    {
        if matches!(self.get_task(), ComputeTask::DoNothing)
        {
            self.set_task(ComputeTask::Redraw);
        }
    }

    fn resize_x(&mut self, width: usize)
    {
        self.grid_mut().resize_x(width);
        self.schedule_compute();
    }

    fn resize_y(&mut self, height: usize)
    {
        self.grid_mut().resize_y(height);
        self.schedule_compute();
    }

    fn change_palette(&mut self, palette: ColorPalette)
    {
        self.get_coloring_mut().set_palette(palette);
        self.schedule_redraw();
    }

    fn scale_palette(&mut self, scale_factor: f64)
    {
        self.get_coloring_mut().scale_period(scale_factor);
        self.schedule_redraw();
    }

    fn set_coloring_algorithm(&mut self, coloring_algorithm: ColoringAlgorithm)
    {
        self.get_coloring_mut().set_algorithm(coloring_algorithm);
        self.schedule_redraw();
    }

    fn shift_palette(&mut self, shift: f64)
    {
        self.get_coloring_mut().adjust_phase(shift);
        self.schedule_redraw();
    }

    fn compute(&mut self);

    fn recompute(&mut self);

    fn redraw(&mut self);

    fn zoom(&mut self, scale: Real, base_point: Cplx);

    #[inline]
    fn pan(&mut self, offset_vector: Cplx)
    {
        self.grid_mut().translate(offset_vector);
        self.schedule_recompute();
    }

    #[inline]
    fn nudge_horizontal(&mut self, relative_offset: f64)
    {
        let grid_width = self.grid().range_x();
        let translation_vector = Cplx::from(grid_width * relative_offset);
        self.pan(translation_vector);
    }

    #[inline]
    fn nudge_vertical(&mut self, relative_offset: f64)
    {
        let grid_height = self.grid().range_y();
        let translation_vector = Cplx::new(0., grid_height * relative_offset);
        self.pan(translation_vector);
    }

    fn process_task(&mut self)
    {
        let task = self.get_task();
        match task
        {
            ComputeTask::Recompute =>
            {
                self.recompute();
                self.redraw();
            }
            ComputeTask::Redraw =>
            {
                self.redraw();
            }
            ComputeTask::DoNothing =>
            {}
            ComputeTask::Compute =>
            {
                self.compute();
                self.redraw();
            }
        }
        self.set_task(ComputeTask::DoNothing);
    }

    fn frame_contains_pixel(&self, pointer_pos: Pos2) -> bool
    {
        self.get_frame().region.contains(pointer_pos)
    }

    fn map_pixel(&self, pointer_pos: Pos2) -> Cplx
    {
        let relative_pos = self.get_frame().to_local_coords(pointer_pos);
        self.grid().map_vec2(relative_pos)
    }

    fn process_mouse_input(&mut self, pointer_value: Cplx, zoom_factor: f32, reselect_point: bool)
    {
        if (zoom_factor - 1.0).abs() > f32::EPSILON
        {
            self.zoom((1. / zoom_factor).into(), pointer_value);
        }

        if reselect_point
        {
            self.select_point(pointer_value);
            self.schedule_redraw();
        }
    }

    fn select_preperiod_smooth_coloring(&mut self);
    fn select_preperiod_period_smooth_coloring(&mut self);

    fn marking_mode(&self) -> &MarkingMode;
    fn marking_mode_mut(&mut self) -> &mut MarkingMode;

    fn cycle_active_plane(&mut self);

    fn increase_max_iter(&mut self);
    fn decrease_max_iter(&mut self);

    // TODO: remove unnecessry mutation
    fn save_image(&mut self, img_width: usize, filename: &str);

    fn change_height(&mut self, new_height: usize);

    fn mark_orbit_and_info(&mut self, pointer_value: Cplx);
    fn mark_external_ray(&mut self, angle: Real);
    fn describe_marked_info(&self) -> String;
}

pub(super) struct WindowPane<P>
where
    P: ParameterPlane + 'static,
{
    pub plane: P,
    pub coloring: Coloring,
    iter_plane: IterPlane<P::Deriv>,
    pub image_frame: ImageFrame,
    task: ComputeTask,
    selection: Cplx,
    marked_curves: Vec<ColoredCurve>,
    marked_points: ColoredPoints,
    marked_info: Option<OrbitInfo<P::Var, P::Param, P::Deriv>>,
    pub marking_mode: MarkingMode,
    pub zoom_factor: Real,
}
impl<P> WindowPane<P>
where
    P: ParameterPlane + 'static,
{
    pub fn set_param(&mut self, new_param: <P::MetaParam as ParamList>::Param)
    {
        let old_param = self.plane.get_param();
        if old_param != new_param
        {
            self.plane.set_param(new_param);
            self.schedule_recompute();
        }
        self.clear_marked_curves();
    }

    #[must_use]
    pub fn new(plane: P, coloring: Coloring) -> Self
    {
        let iter_plane = plane.compute();
        let task = ComputeTask::Redraw;
        let selection = plane.default_selection();

        let image =
            RetainedImage::from_color_image("parameter plane", iter_plane.render(&coloring));
        let frame = ImageFrame {
            image,
            region: Rect::NOTHING,
        };

        let marked_curves = vec![];
        let marked_points = vec![];

        Self {
            plane,
            coloring,
            iter_plane,
            image_frame: frame,
            task,
            selection,
            marked_curves,
            marked_points,
            marked_info: None,
            marking_mode: MarkingMode::default(),
            zoom_factor: 1.,
        }
    }

    #[inline]
    const fn plane(&self) -> &P
    {
        &self.plane
    }
    #[inline]
    fn plane_mut(&mut self) -> &mut P
    {
        &mut self.plane
    }

    #[inline]
    const fn get_marked_info(&self) -> &Option<OrbitInfo<P::Var, P::Param, P::Deriv>>
    {
        &self.marked_info
    }
    #[inline]
    fn get_marked_info_mut(&mut self) -> &mut Option<OrbitInfo<P::Var, P::Param, P::Deriv>>
    {
        &mut self.marked_info
    }
    #[inline]
    fn set_marked_info(&mut self, info: OrbitInfo<P::Var, P::Param, P::Deriv>)
    {
        self.marked_info = Some(info);
    }
    #[inline]
    fn del_marked_info(&mut self)
    {
        self.marked_info = None;
    }
}

impl<P> From<P> for WindowPane<P>
where
    P: ParameterPlane + 'static,
{
    fn from(plane: P) -> Self
    {
        let coloring = plane.default_coloring();
        Self::new(plane, coloring)
    }
}

impl<P> Pane for WindowPane<P>
where
    P: ParameterPlane + 'static,
{
    #[inline]
    fn get_task(&self) -> &ComputeTask
    {
        &self.task
    }
    #[inline]
    fn set_task(&mut self, new_task: ComputeTask)
    {
        self.task = new_task;
    }
    #[inline]
    fn grid(&self) -> &PointGrid
    {
        self.plane.point_grid()
    }
    #[inline]
    fn grid_mut(&mut self) -> &mut PointGrid
    {
        self.plane.point_grid_mut()
    }
    #[inline]
    fn get_frame(&self) -> &ImageFrame
    {
        &self.image_frame
    }
    #[inline]
    fn get_frame_mut(&mut self) -> &mut ImageFrame
    {
        &mut self.image_frame
    }
    #[inline]
    fn get_marked_curves(&self) -> &Vec<ColoredCurve>
    {
        &self.marked_curves
    }
    #[inline]
    fn get_marked_curves_mut(&mut self) -> &mut Vec<ColoredCurve>
    {
        &mut self.marked_curves
    }
    #[inline]
    fn get_marked_points(&self) -> &ColoredPoints
    {
        &self.marked_points
    }
    #[inline]
    fn get_marked_points_mut(&mut self) -> &mut ColoredPoints
    {
        &mut self.marked_points
    }
    #[inline]
    fn get_coloring(&self) -> &Coloring
    {
        &self.coloring
    }
    #[inline]
    fn get_coloring_mut(&mut self) -> &mut Coloring
    {
        &mut self.coloring
    }
    #[inline]
    fn get_image_frame(&self) -> &ImageFrame
    {
        &self.image_frame
    }
    #[inline]
    fn get_image_frame_mut(&mut self) -> &mut ImageFrame
    {
        &mut self.image_frame
    }
    #[inline]
    fn get_selection(&self) -> Cplx
    {
        self.selection
    }
    #[inline]
    fn reset_selection(&mut self)
    {
        self.select_point(self.plane.default_selection());
    }
    #[inline]
    fn marking_mode(&self) -> &MarkingMode
    {
        &self.marking_mode
    }
    #[inline]
    fn marking_mode_mut(&mut self) -> &mut MarkingMode
    {
        &mut self.marking_mode
    }
    #[inline]
    fn select_point(&mut self, point: Cplx)
    {
        self.selection = point;
        //
        // #[cfg(not(target_arch = "wasm32"))]
        // dbg!(self.selection);
    }

    #[inline]
    fn cycle_active_plane(&mut self)
    {
        self.plane.cycle_active_plane();
        self.schedule_recompute();
    }

    fn increase_max_iter(&mut self)
    {
        let iters = self.plane.max_iter_mut();
        *iters *= 2;
        self.schedule_recompute();
    }
    fn decrease_max_iter(&mut self)
    {
        let iters = self.plane.max_iter_mut();
        *iters /= 2;
        self.schedule_recompute();
    }
    #[inline]
    fn redraw(&mut self)
    {
        self.marked_points = self.marking_mode.compute(&self.plane, self.get_selection());
        let image = self.iter_plane.render(self.get_coloring());
        let image_frame = self.get_frame_mut();
        image_frame.image = RetainedImage::from_color_image("Parameter Plane", image);
    }

    fn change_height(&mut self, new_height: usize)
    {
        self.plane.point_grid_mut().resize_y(new_height);
        self.schedule_compute();
    }

    #[inline]
    fn compute(&mut self)
    {
        self.iter_plane = self.plane.compute();
    }

    #[inline]
    fn recompute(&mut self)
    {
        self.plane.compute_into(&mut self.iter_plane);
    }

    #[inline]
    fn zoom(&mut self, scale: Real, base_point: Cplx)
    {
        self.zoom_factor *= scale;
        self.grid_mut().zoom(scale, base_point);
        self.schedule_recompute();
    }

    fn select_preperiod_smooth_coloring(&mut self)
    {
        let coloring_algorithm = self.plane.preperiod_smooth_coloring();
        self.set_coloring_algorithm(coloring_algorithm);
    }

    fn select_preperiod_period_smooth_coloring(&mut self)
    {
        let coloring_algorithm = self.plane.preperiod_period_smooth_coloring();
        self.set_coloring_algorithm(coloring_algorithm);
    }

    fn save_image(&mut self, img_width: usize, filename: &str)
    {
        let orig_width = self.grid().res_x;
        self.grid_mut().resize_x(img_width);
        let iter_plane = self.plane.compute();
        iter_plane.save(self.get_coloring(), filename.to_owned());
        self.grid_mut().resize_x(orig_width);
    }

    fn mark_orbit_and_info(&mut self, pointer_value: Cplx)
    {
        let (orbit, info) = self.plane.get_orbit_and_info(pointer_value);
        let orbit_pts = orbit.iter().map(|x| (*x).into()).collect();
        self.mark_curve(orbit_pts, Color32::GREEN);
        self.set_marked_info(info);
    }

    fn mark_external_ray(&mut self, angle: Real)
    {
        if let Some(cs) = self.plane.external_ray(angle, 50, 50, 128)
        {
            self.mark_curve(cs, Color32::BLUE);
        }
    }

    fn describe_marked_info(&self) -> String
    {
        self.get_marked_info()
            .map_or_else(String::new, |orbit_info| orbit_info.to_string())
    }

    fn name(&self) -> String
    {
        self.plane.get_param().summarize().map_or_else(
            || self.plane.name(),
            |local| format!("{}: {}", self.plane.name(), local),
        )
    }
}

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

pub trait PanePair
{
    fn parent(&self) -> &dyn Pane;
    fn parent_mut(&mut self) -> &mut dyn Pane;
    fn child(&self) -> &dyn Pane;
    fn child_mut(&mut self) -> &mut dyn Pane;
    fn randomize_palette(&mut self);
    fn set_palette(&mut self, palette: ColorPalette);
    fn set_coloring_algorithm(&mut self, coloring_algorithm: ColoringAlgorithm);

    fn get_pane(&self, pane_id: PaneID) -> &dyn Pane;
    fn get_pane_mut(&mut self, pane_id: PaneID) -> &mut dyn Pane;
    fn set_active_pane(&mut self, pane_id: Option<PaneID>);
    fn get_active_pane(&self) -> Option<&dyn Pane>;
    fn get_active_pane_mut(&mut self) -> Option<&mut dyn Pane>;
    fn save_active_pane(&mut self);
    fn save_pane(&mut self, pane_id: PaneID);
    fn get_image_height(&self) -> usize;
    fn change_height(&mut self, new_height: usize);

    // fn descend(self) -> Box<dyn PanePair>;
}

pub trait Interactive
{
    fn handle_mouse(&mut self, ctx: &Context);
    fn handle_input(&mut self, ctx: &Context);

    fn toggle_live_mode(&mut self);
    fn show_save_dialog(&mut self, ctx: &Context);
    fn process_tasks(&mut self);
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
    save_dialog: Option<FileDialog>,
    pane_to_save: PaneID,
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
            save_dialog: None,
            pane_to_save: PaneID::Parent,
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

    // fn to_child(self) -> MainInterface<J, C> {
    //     let new_parent = self.child.plane;
    //     let new_child = C::from(new_parent.clone());
    //     MainInterface::new(new_parent, new_child)
    // }
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

    fn save_pane(&mut self, pane_id: PaneID)
    {
        self.pane_to_save = pane_id;
        let mut dialog = FileDialog::save_file(None)
            .default_filename(format!("{}.png", self.parent.name()))
            .show_rename(false)
            .show_new_folder(true);
        dialog.open();
        self.save_dialog = Some(dialog);
    }

    fn save_active_pane(&mut self)
    {
        if let Some(pane_id) = self.active_pane
        {
            self.save_pane(pane_id);
        }
    }

    // fn save_active_pane(&mut self)
    // {
    //     if let Some(pane) = self.get_active_pane_mut() {
    //     // Open a save file dialog if it's not already open
    //     if self.save_dialog.is_none()
    //     {
    //         let dialog = FileDialog::save_file(None)
    //             .default_filename("fractal.png")
    //             .show_new_folder(true); // if you want to allow creating new folders
    //         self.save_dialog = Some(dialog);
    //     }
    //
    // }
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

    fn set_coloring_algorithm(&mut self, coloring_algorithm: ColoringAlgorithm)
    {
        self.parent_mut().set_coloring_algorithm(coloring_algorithm);
        self.child_mut().set_coloring_algorithm(coloring_algorithm);
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
            if reselect_point
            {
                let child_param = self.parent.plane.param_map(pointer_value);
                self.set_child_param(pointer_value, child_param);
            }

            if clicked
            {
                self.consume_click();
                self.child_mut().clear_marked_curves();
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
                self.child_mut().clear_marked_curves();
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
    }

    fn process_tasks(&mut self)
    {
        self.parent.process_task();
        self.child.process_task();
    }

    fn show_save_dialog(&mut self, ctx: &Context)
    {
        let Some(save_dialog) = self.save_dialog.as_mut() else {return};

        save_dialog.show(ctx); // show the dialog

        // Check if a file has been selected
        if save_dialog.selected()
        {
            if let Some(path) = save_dialog.path()
            {
                let filename = path.to_string_lossy().into_owned();

                let image_width: usize = 4096;

                // // Use a slider for image width input
                // SidePanel::left("side_panel").show(ctx, |ui| {
                //     ui.heading("Enter image width:");
                //     ui.add(Slider::new(&mut image_width, 1..=1000));
                // });

                let pane = self.get_pane_mut(self.pane_to_save);
                pane.save_image(image_width, &filename);
                self.save_dialog = None;
            }
            self.set_active_pane(None);
        }
    }

    #[allow(clippy::cognitive_complexity)]
    fn handle_input(&mut self, ctx: &Context)
    {
        if let Some(save_dialog) = self.save_dialog.as_ref()
        {
            if save_dialog.visible()
            {
                ctx.set_cursor_icon(CursorIcon::Default);
                return;
            }
        }

        if shortcut_used!(ctx, &CTRL_Q)
        {
            self.schedule_quit();
        }

        if shortcut_used!(ctx, &CTRL_W)
        {
            self.schedule_close();
        }

        if shortcut_used!(ctx, &KEY_R)
        {
            self.randomize_palette();
        }

        if shortcut_used!(ctx, &KEY_B)
        {
            let black_palette = ColorPalette::black(32.);
            self.set_palette(black_palette);
        }

        if shortcut_used!(ctx, &KEY_W)
        {
            let white_palette = ColorPalette::white(32.);
            self.set_palette(white_palette);
        }

        if shortcut_used!(ctx, &CTRL_P)
        {
            self.parent_mut().cycle_active_plane();
            self.child_mut().cycle_active_plane();
        }

        if shortcut_used!(ctx, &KEY_I)
        {
            if let Some(pane) = self.get_active_pane_mut()
            {
                pane.marking_mode_mut().toggle_selection();
                pane.schedule_redraw();
            }
        }

        if shortcut_used!(ctx, &KEY_P)
        {
            self.child_mut().marking_mode_mut().toggle_critical();
            self.child_mut().schedule_redraw();
        }

        if shortcut_used!(ctx, &KEY_O)
        {
            self.parent_mut().marking_mode_mut().toggle_critical();
            self.parent_mut().schedule_redraw();
        }

        seq!(n in 1..=6 {
            if shortcut_used!(ctx, &CTRL_~n)
            {
                self.child_mut().marking_mode_mut().toggle_cycles(n);
                self.child_mut().schedule_redraw();
            }
            if shortcut_used!(ctx, &CTRL_SHIFT_~n)
            {
                self.parent_mut().marking_mode_mut().toggle_cycles(n);
                self.parent_mut().schedule_redraw();
            }
        });

        // if shortcut_used!(ctx, &KEY_E)
        // {
        //     if let Some(pane) = self.get_active_pane_mut()
        //     {
        //         pane.mark_external_ray(1./7.);
        //     }
        // }

        if shortcut_used!(ctx, &KEY_H)
        {
            match input!("Enter new image height: ").parse::<usize>()
            {
                Ok(new_height) =>
                {
                    self.change_height(new_height);
                }
                Err(e) => println!("Error parsing height: {e:?}"),
            }
        }
        //
        // if shortcut_used!(ctx, &CTRL_C)
        // {
        //     if let Some(pane) = self.get_active_pane_mut() {
        //     match input!("Enter new param: ").parse::<Cplx>()
        //     {
        //         Ok(new_param) =>
        //         {
        //             pane.set_param(new_param);
        //         }
        //         Err(e) => println!("Error parsing param: {e:?}"),
        //     }
        // }

        if shortcut_used!(ctx, &KEY_UP)
        {
            if let Some(pane) = self.get_active_pane_mut()
            {
                pane.scale_palette(1.25);
            }
        }

        if shortcut_used!(ctx, &KEY_DOWN)
        {
            if let Some(pane) = self.get_active_pane_mut()
            {
                pane.scale_palette(0.8);
            }
        }

        if shortcut_used!(ctx, &KEY_LEFT)
        {
            if let Some(pane) = self.get_active_pane_mut()
            {
                pane.shift_palette(-0.02);
            }
        }

        if shortcut_used!(ctx, &KEY_RIGHT)
        {
            if let Some(pane) = self.get_active_pane_mut()
            {
                pane.shift_palette(0.02);
            }
        }

        if shortcut_used!(ctx, &SHIFT_LEFT)
        {
            if let Some(pane) = self.get_active_pane_mut()
            {
                pane.nudge_horizontal(-0.01);
            }
        }

        if shortcut_used!(ctx, &SHIFT_RIGHT)
        {
            if let Some(pane) = self.get_active_pane_mut()
            {
                pane.nudge_horizontal(0.01);
            }
        }

        if shortcut_used!(ctx, &SHIFT_UP)
        {
            if let Some(pane) = self.get_active_pane_mut()
            {
                pane.nudge_vertical(0.01);
            }
        }

        if shortcut_used!(ctx, &SHIFT_DOWN)
        {
            if let Some(pane) = self.get_active_pane_mut()
            {
                pane.nudge_vertical(-0.01);
            }
        }

        if shortcut_used!(ctx, &KEY_Z)
        {
            if let Some(pane) = self.get_active_pane_mut()
            {
                pane.zoom(0.8, pane.get_selection());
            }
        }

        if shortcut_used!(ctx, &CTRL_Z)
        {
            if let Some(pane) = self.get_active_pane_mut()
            {
                pane.zoom(0.125, pane.get_selection());
            }
        }

        if shortcut_used!(ctx, &KEY_V)
        {
            if let Some(pane) = self.get_active_pane_mut()
            {
                pane.zoom(1.25, pane.get_selection());
            }
        }

        if shortcut_used!(ctx, &CTRL_V)
        {
            if let Some(pane) = self.get_active_pane_mut()
            {
                pane.zoom(8., pane.get_selection());
            }
        }

        if shortcut_used!(ctx, &KEY_SPACE)
        {
            if let Some(pane) = self.get_active_pane_mut()
            {
                let selection = pane.get_selection();
                pane.grid_mut().recenter(selection);
                pane.schedule_recompute();
            }
        }

        if shortcut_used!(ctx, &SHIFT_SPACE)
        {
            match self.active_pane
            {
                Some(PaneID::Parent) =>
                {
                    self.parent.reset_selection();
                    let new_child_param = self.parent.plane.param_map(self.parent.selection);
                    self.set_child_param(self.parent.selection, new_child_param);
                }
                Some(PaneID::Child) =>
                {
                    self.child.reset_selection();
                    self.child.clear_marked_curves();
                }
                None =>
                {}
            }
        }

        if shortcut_used!(ctx, &KEY_0)
        {
            if let Some(pane) = self.get_active_pane_mut()
            {
                pane.set_coloring_algorithm(ColoringAlgorithm::Solid);
            }
        }

        if shortcut_used!(ctx, &KEY_1)
        {
            if let Some(pane) = self.get_active_pane_mut()
            {
                pane.set_coloring_algorithm(ColoringAlgorithm::Period);
            }
        }

        if shortcut_used!(ctx, &KEY_2)
        {
            if let Some(pane) = self.get_active_pane_mut()
            {
                pane.set_coloring_algorithm(ColoringAlgorithm::PeriodMultiplier);
            }
        }

        if shortcut_used!(ctx, &KEY_3)
        {
            if let Some(pane) = self.get_active_pane_mut()
            {
                pane.set_coloring_algorithm(ColoringAlgorithm::Multiplier);
            }
        }

        if shortcut_used!(ctx, &KEY_4)
        {
            if let Some(pane) = self.get_active_pane_mut()
            {
                pane.set_coloring_algorithm(ColoringAlgorithm::Preperiod);
            }
        }

        if shortcut_used!(ctx, &KEY_5)
        {
            if let Some(pane) = self.get_active_pane_mut()
            {
                pane.select_preperiod_smooth_coloring();
            }
        }

        if shortcut_used!(ctx, &KEY_C)
        {
            if let Some(pane) = self.get_active_pane_mut()
            {
                pane.clear_marked_curves();
            }
            // pane.clear_marked_points();
        }

        if shortcut_used!(ctx, &KEY_L)
        {
            self.toggle_live_mode();
        }

        if shortcut_used!(ctx, &CTRL_S)
        {
            self.save_active_pane();
            // if let Some(pane) = self.get_active_pane_mut() {
            // let filename = input!("Enter image filename to save: ");
            // match input!("Enter width of image: ").parse::<usize>()
            // {
            //     Ok(width) =>
            //     {
            //         pane.save_image(width, &filename);
            //         println!("Image saved to images/{}", &filename);
            //     }
            //     Err(e) => println!("Error parsing width: {e:?}"),
            // }
        }

        if shortcut_used!(ctx, &KEY_EQUALS)
        {
            if let Some(pane) = self.get_active_pane_mut()
            {
                pane.increase_max_iter();
            }
        }

        if shortcut_used!(ctx, &KEY_MINUS)
        {
            if let Some(pane) = self.get_active_pane_mut()
            {
                pane.decrease_max_iter();
            }
        }
        self.handle_mouse(ctx);
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
                        let orbit_desc = self.parent.describe_marked_info();
                        ui.label(orbit_desc);
                    });
                    row.col(|ui| {
                        let orbit_desc = self.child.describe_marked_info();
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
