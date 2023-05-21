use super::ImageFrame;
use crate::coloring::{algorithms::ColoringAlgorithm, palette::ColorPalette, Coloring};
use crate::dynamics::julia::JuliaSet;
use crate::dynamics::ParameterPlane;
use crate::iter_plane::{FractalImage, IterPlane};
use crate::point_grid::{Bounds, PointGrid};
use crate::profiles::QuadRatPer2;
use crate::types::*;
use input_macro::input;

use super::keyboard_shortcuts::*;
use super::marked_points::MarkingMode;

use eframe::egui::{Color32, Context, CursorIcon, InputState, Key, Pos2, Rect, Stroke, Ui};
use egui_extras::{Column, RetainedImage, TableBuilder};
use epaint::{CircleShape, PathShape};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub type ColoredPoint = (ComplexNum, Color32);
pub type ColoredPoints = Vec<ColoredPoint>;
pub type ColoredCurve = (Vec<ComplexNum>, Color32);

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum RedrawMessage
{
    DoNothing,
    Redraw,
    Recompute,
    Compute,
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
    fn get_task(&self) -> &RedrawMessage;
    fn set_task(&mut self, new_task: RedrawMessage);

    fn get_frame(&self) -> &ImageFrame;
    fn get_frame_mut(&mut self) -> &mut ImageFrame;

    fn get_coloring(&self) -> &Coloring;
    fn get_coloring_mut(&mut self) -> &mut Coloring;

    fn select_point(&mut self, point: ComplexNum);
    fn get_selection(&self) -> ComplexNum;

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

    fn mark_point(&mut self, z: ComplexNum, color: Color32)
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
        self.set_task(RedrawMessage::Compute);
    }

    fn schedule_recompute(&mut self)
    {
        if !matches!(self.get_task(), RedrawMessage::Compute)
        {
            self.set_task(RedrawMessage::Recompute);
        }
    }

    fn schedule_redraw(&mut self)
    {
        if matches!(self.get_task(), RedrawMessage::DoNothing)
        {
            self.set_task(RedrawMessage::Redraw);
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
    // {
    //     let image = self.get_iter_plane().render(self.get_coloring());
    //     let image_frame = self.get_frame_mut();
    //     image_frame.image = RetainedImage::from_color_image("Parameter Plane", image);
    // }

    fn zoom(&mut self, scale: RealNum, base_point: ComplexNum);
    // {
    //     self.grid_mut().zoom(scale, base_point);
    //     self.schedule_recompute();
    // }

    fn process_task(&mut self)
    {
        let task = self.get_task();
        match task
        {
            RedrawMessage::Recompute =>
            {
                self.recompute();
                self.redraw();
            }
            RedrawMessage::Redraw =>
            {
                self.redraw();
            }
            RedrawMessage::DoNothing =>
            {}
            RedrawMessage::Compute =>
            {
                self.compute();
                self.redraw();
            }
        }
        self.set_task(RedrawMessage::DoNothing);
    }

    fn frame_contains_pixel(&self, pointer_pos: Pos2) -> bool
    {
        self.get_frame().region.contains(pointer_pos)
    }

    fn map_pixel(&self, pointer_pos: Pos2) -> ComplexNum
    {
        let relative_pos = self.get_frame().to_local_coords(pointer_pos);
        self.grid().map_vec2(relative_pos)
    }

    fn process_mouse_input(
        &mut self,
        pointer_value: ComplexNum,
        zoom_factor: f32,
        reselect_point: bool,
    )
    {
        if (zoom_factor - 1.0).abs() > f32::EPSILON
        {
            self.zoom((1. / zoom_factor).into(), pointer_value);
        }

        if reselect_point
        {
            self.select_point(pointer_value);
        }
    }

    fn select_preperiod_smooth_coloring(&mut self);
    fn select_preperiod_period_smooth_coloring(&mut self);

    fn marking_mode(&self) -> &MarkingMode;
    fn marking_mode_mut(&mut self) -> &mut MarkingMode;

    fn increase_max_iter(&mut self);
    fn decrease_max_iter(&mut self);

    // TODO: remove unnecessry mutation
    fn save_image(&mut self, img_width: usize, filename: &str);

    fn change_height(&mut self, new_height: usize);

    fn mark_orbit_and_info(&mut self, pointer_value: ComplexNum);
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
    task: RedrawMessage,
    selection: ComplexNum,
    marked_curves: Vec<ColoredCurve>,
    marked_points: ColoredPoints,
    marked_info: Option<OrbitInfo<P::Var, P::Param, P::Deriv>>,
    pub marking_mode: MarkingMode,
    pub zoom_factor: RealNum,
}
impl<P> WindowPane<P>
where
    P: ParameterPlane + 'static,
{
    pub fn set_param(&mut self, new_param: P::Param)
    {
        self.plane.set_param(new_param);
        self.schedule_recompute();
        self.clear_marked_curves();
    }

    #[must_use]
    pub fn new(plane: P, coloring: Coloring) -> Self
    {
        let iter_plane = plane.compute();
        let task = RedrawMessage::Redraw;
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
    fn get_task(&self) -> &RedrawMessage
    {
        &self.task
    }
    #[inline]
    fn set_task(&mut self, new_task: RedrawMessage)
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
    fn get_selection(&self) -> ComplexNum
    {
        self.selection
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
    fn select_point(&mut self, point: ComplexNum)
    {
        self.selection = point;
        dbg!(self.selection);
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
        self.marked_points = self.marking_mode.compute(self.plane());
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
    fn zoom(&mut self, scale: RealNum, base_point: ComplexNum)
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
        let iter_plane = self.plane().compute();
        let filepath = format!("images/{filename}");
        iter_plane.save(self.get_coloring(), filepath);
        self.grid_mut().resize_x(orig_width);
    }

    fn mark_orbit_and_info(&mut self, pointer_value: ComplexNum)
    {
        let (orbit, info) = self.plane.get_orbit_and_info(pointer_value);
        let orbit_pts = orbit.iter().map(|x| (*x).into()).collect();
        self.mark_curve(orbit_pts, Color32::GREEN);
        self.set_marked_info(info);
    }

    fn describe_marked_info(&self) -> String
    {
        self.get_marked_info()
            .map_or_else(String::new, |orbit_info| orbit_info.to_string())
    }

    fn name(&self) -> String
    {
        format!("{}: c = {}", self.plane.name(), self.plane.get_param())
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

    fn handle_mouse(&mut self, ctx: &Context);
    fn handle_input(&mut self, ctx: &Context);

    fn change_height(&mut self, new_height: usize);

    fn toggle_live_mode(&mut self);
    fn set_active_pane(&mut self, pane_id: PaneID);
    fn get_active_pane(&self) -> &dyn Pane;
    fn get_active_pane_mut(&mut self) -> &mut dyn Pane;
    fn process_tasks(&mut self);
    fn save_active_pane(&mut self);
    fn show(&mut self, ui: &mut Ui);
}

pub struct WindowPanePair<P, J>
where
    P: ParameterPlane + 'static,
    J: ParameterPlane<Param = P::Param> + 'static,
{
    parent: WindowPane<P>,
    child: WindowPane<J>,
    active_pane: PaneID,
    live_mode: bool,
}

impl<P, J> WindowPanePair<P, J>
where
    P: ParameterPlane,
    J: ParameterPlane<Param = P::Param> + 'static,
{
    pub fn new(parent: P, child: J) -> Self
    {
        Self {
            parent: parent.into(),
            child: child.into(),
            active_pane: PaneID::Parent,
            live_mode: false,
        }
    }

    fn set_child_param(&mut self, point: ComplexNum, new_param: P::Param)
    {
        let mut new_bounds = self.parent.plane.default_julia_bounds(point, new_param);
        new_bounds.zoom(self.child.zoom_factor, new_bounds.center());
        self.child.grid_mut().change_bounds(new_bounds);
        self.child.set_param(new_param);
        self.child.schedule_recompute();
    }
}

impl<P, J> PanePair for WindowPanePair<P, J>
where
    P: ParameterPlane,
    J: ParameterPlane<Param = P::Param> + 'static,
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

    fn handle_mouse(&mut self, ctx: &Context)
    {
        let clicked = ctx.input(|i| i.pointer.any_click());
        let zoom_factor = ctx.input(InputState::zoom_delta);

        if let Some(pointer_pos) = ctx.pointer_latest_pos()
        {
            if self.parent().frame_contains_pixel(pointer_pos)
            {
                ctx.set_cursor_icon(CursorIcon::Crosshair);
                self.set_active_pane(PaneID::Parent);
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
                    // self.consume_click();
                    self.parent_mut().clear_marked_curves();
                    self.parent_mut().mark_orbit_and_info(pointer_value);
                }
            }
            else if self.child().frame_contains_pixel(pointer_pos)
            {
                ctx.set_cursor_icon(CursorIcon::Crosshair);
                self.set_active_pane(PaneID::Child);
                let pointer_value = self.child().map_pixel(pointer_pos);
                self.child_mut()
                    .process_mouse_input(pointer_value, zoom_factor, clicked);

                if clicked
                {
                    // self.consume_click();
                    self.child_mut().clear_marked_curves();
                    self.child_mut().mark_orbit_and_info(pointer_value);
                }
            }
            else
            {
                ctx.set_cursor_icon(CursorIcon::Default);
            }
        }
    }

    fn toggle_live_mode(&mut self)
    {
        self.live_mode ^= true;
    }

    fn set_active_pane(&mut self, pane_id: PaneID)
    {
        self.active_pane = pane_id;
    }

    fn get_active_pane(&self) -> &dyn Pane
    {
        match self.active_pane
        {
            PaneID::Parent => self.parent(),
            PaneID::Child => self.child(),
        }
    }
    fn get_active_pane_mut(&mut self) -> &mut dyn Pane
    {
        match self.active_pane
        {
            PaneID::Parent => self.parent_mut(),
            PaneID::Child => self.child_mut(),
        }
    }
    fn process_tasks(&mut self)
    {
        self.parent.process_task();
        self.child.process_task();
    }

    fn change_height(&mut self, new_height: usize)
    {
        self.parent.change_height(new_height);
        self.child.change_height(new_height);
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

    fn save_active_pane(&mut self)
    {
        let pane = self.get_active_pane_mut();
        let filename = input!("Enter image filename to save: ");
        match input!("Enter width of image: ").parse::<usize>()
        {
            Ok(width) =>
            {
                pane.save_image(width, &filename);
                println!("Image saved to images/{}", &filename);
            }
            Err(e) => println!("Error parsing width: {:?}", e),
        }
    }

    fn handle_input(&mut self, ctx: &Context)
    {
        if ctx.input_mut(|i| i.consume_shortcut(&KEY_R))
        {
            self.randomize_palette();
        }

        if ctx.input_mut(|i| i.consume_shortcut(&KEY_B))
        {
            let black_palette = ColorPalette::black(32.);
            self.set_palette(black_palette);
        }

        if ctx.input_mut(|i| i.consume_shortcut(&KEY_W))
        {
            let white_palette = ColorPalette::white(32.);
            self.set_palette(white_palette);
        }

        if ctx.input_mut(|i| i.consume_shortcut(&KEY_P))
        {
            self.child_mut().marking_mode_mut().toggle_critical();
            self.child_mut().schedule_redraw();
        }

        if ctx.input_mut(|i| i.consume_shortcut(&KEY_Y))
        {
            self.child_mut().marking_mode_mut().toggle_cycles(1);
            self.child_mut().schedule_redraw();
        }

        if ctx.input_mut(|i| i.consume_shortcut(&KEY_U))
        {
            self.child_mut().marking_mode_mut().toggle_cycles(2);
            self.child_mut().schedule_redraw();
        }

        if ctx.input_mut(|i| i.consume_shortcut(&KEY_H))
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

        if ctx.input_mut(|i| i.consume_shortcut(&KEY_UP))
        {
            let pane = self.get_active_pane_mut();
            pane.scale_palette(1.25);
        }

        if ctx.input_mut(|i| i.consume_shortcut(&KEY_DOWN))
        {
            let pane = self.get_active_pane_mut();
            pane.scale_palette(0.8);
        }

        if ctx.input_mut(|i| i.consume_shortcut(&KEY_LEFT))
        {
            let pane = self.get_active_pane_mut();
            pane.shift_palette(-0.02);
        }

        if ctx.input_mut(|i| i.consume_shortcut(&KEY_RIGHT))
        {
            let pane = self.get_active_pane_mut();
            pane.shift_palette(0.02);
        }

        if ctx.input_mut(|i| i.consume_shortcut(&KEY_Z))
        {
            let pane = self.get_active_pane_mut();
            pane.zoom(0.8, pane.get_selection());
        }

        if ctx.input_mut(|i| i.consume_shortcut(&CTRL_Z))
        {
            let pane = self.get_active_pane_mut();
            pane.zoom(0.125, pane.get_selection());
        }

        if ctx.input_mut(|i| i.consume_shortcut(&KEY_V))
        {
            let pane = self.get_active_pane_mut();
            pane.zoom(1.25, pane.get_selection());
        }

        if ctx.input_mut(|i| i.consume_shortcut(&CTRL_V))
        {
            let pane = self.get_active_pane_mut();
            pane.zoom(8., pane.get_selection());
        }

        if ctx.input_mut(|i| i.consume_shortcut(&KEY_SPACE))
        {
            let pane = self.get_active_pane_mut();
            let selection = pane.get_selection();
            pane.grid_mut().recenter(selection);
            pane.schedule_recompute();
        }

        if ctx.input_mut(|i| i.consume_shortcut(&KEY_0))
        {
            let pane = self.get_active_pane_mut();
            pane.set_coloring_algorithm(ColoringAlgorithm::Solid);
        }

        if ctx.input_mut(|i| i.consume_shortcut(&KEY_1))
        {
            let pane = self.get_active_pane_mut();
            pane.set_coloring_algorithm(ColoringAlgorithm::Period);
        }

        if ctx.input_mut(|i| i.consume_shortcut(&KEY_2))
        {
            let pane = self.get_active_pane_mut();
            pane.set_coloring_algorithm(ColoringAlgorithm::PeriodMultiplier);
        }

        if ctx.input_mut(|i| i.consume_shortcut(&KEY_3))
        {
            let pane = self.get_active_pane_mut();
            pane.set_coloring_algorithm(ColoringAlgorithm::Multiplier);
        }

        if ctx.input_mut(|i| i.consume_shortcut(&KEY_4))
        {
            let pane = self.get_active_pane_mut();
            pane.set_coloring_algorithm(ColoringAlgorithm::Preperiod);
        }

        if ctx.input_mut(|i| i.consume_shortcut(&KEY_5))
        {
            let pane = self.get_active_pane_mut();
            pane.select_preperiod_smooth_coloring();
        }

        if ctx.input_mut(|i| i.consume_shortcut(&KEY_C))
        {
            let pane = self.get_active_pane_mut();
            pane.clear_marked_curves();
            // pane.clear_marked_points();
        }

        if ctx.input_mut(|i| i.consume_shortcut(&KEY_L))
        {
            self.toggle_live_mode();
        }

        if ctx.input_mut(|i| i.consume_shortcut(&CTRL_S))
        {
            let pane = self.get_active_pane_mut();
            let filename = input!("Enter image filename to save: ");
            match input!("Enter width of image: ").parse::<usize>()
            {
                Ok(width) =>
                {
                    pane.save_image(width, &filename);
                    println!("Image saved to images/{}", &filename);
                }
                Err(e) => println!("Error parsing width: {e:?}"),
            }
        }

        if ctx.input_mut(|i| i.consume_shortcut(&KEY_EQUALS))
        {
            let pane = self.get_active_pane_mut();
            pane.increase_max_iter();
        }

        if ctx.input_mut(|i| i.consume_shortcut(&KEY_MINUS))
        {
            let pane = self.get_active_pane_mut();
            pane.decrease_max_iter();
        }
        self.handle_mouse(ctx);
    }

    fn show(&mut self, ui: &mut Ui)
    {
        TableBuilder::new(ui)
            .column(Column::auto().resizable(true))
            .column(Column::remainder())
            .vscroll(false)
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
}