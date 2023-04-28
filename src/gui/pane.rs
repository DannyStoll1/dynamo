use super::ImageFrame;
use crate::coloring::{coloring_algorithm::ColoringAlgorithm, palette::ColorPalette, Coloring};
use crate::dynamics::ParameterPlane;
use crate::iter_plane::{FractalImage, IterPlane};
use crate::point_grid::{Bounds, PointGrid};
use crate::types::{ComplexNum, ComplexVec, RealNum, OrbitInfo};
use crate::profiles::QuadRatPer2;

use eframe::egui::{Color32, Pos2, Rect, Stroke, Ui};
use egui_extras::RetainedImage;
use epaint::PathShape;

pub type ColoredPoint = (ComplexNum, Color32);
pub type ColoredPoints = Vec<ColoredPoint>;
pub type ColoredCurve = (Vec<ComplexNum>, Color32);

#[derive(Clone, Copy, Debug)]
pub(super) enum RedrawMessage
{
    DoNothing,
    Redraw,
    Recompute,
}

pub(super) enum PaneID
{
    Parent,
    Child,
}

pub(super) trait Pane
{
    fn plane(&self) -> &Box<dyn ParameterPlane>;
    fn plane_mut(&mut self) -> &mut Box<dyn ParameterPlane>;

    fn get_task(&self) -> RedrawMessage;
    fn set_task(&mut self, new_task: RedrawMessage);

    fn get_frame(&self) -> &ImageFrame;
    fn get_frame_mut(&mut self) -> &mut ImageFrame;

    fn get_coloring(&self) -> &Coloring;
    fn get_coloring_mut(&mut self) -> &mut Coloring;

    fn get_iter_plane(&self) -> &IterPlane;
    fn get_iter_plane_mut(&mut self) -> &mut IterPlane;

    fn select_point(&mut self, point: ComplexNum);
    fn get_selection(&self) -> ComplexNum;

    fn get_marked_curves(&self) -> &Vec<ColoredCurve>;
    fn get_marked_curves_mut(&mut self) -> &mut Vec<ColoredCurve>;

    fn get_marked_info(&self) -> &Option<OrbitInfo>;
    fn get_marked_info_mut(&mut self) -> &mut Option<OrbitInfo>;
    fn set_marked_info(&mut self, info: OrbitInfo);
    fn del_marked_info(&mut self);

    fn mark_curve(&mut self, zs: ComplexVec, color: Color32)
    {
        let curves = self.get_marked_curves_mut();
        curves.push((zs, color));
    }

    fn get_marked_points(&self) -> &ColoredPoints;
    fn get_marked_points_mut(&mut self) -> &mut ColoredPoints;

    fn mark_point(&mut self, z: ComplexNum, color: Color32)
    {
        let points = self.get_marked_points_mut();
        points.push((z, color));
    }

    fn clear_marked_curves(&mut self)
    {
        let curves = self.get_marked_curves_mut();
        *curves = vec![];
    }

    fn put_marked_curves(&self, ui: &mut Ui)
    {
        let frame = self.get_frame();
        let grid = self.plane().point_grid();
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

    fn name(&self) -> String
    {
        self.plane().name()
    }

    fn grid(&self) -> PointGrid
    {
        self.plane().point_grid()
    }

    fn grid_mut(&mut self) -> &mut PointGrid
    {
        self.plane_mut().point_grid_mut()
    }

    fn rescale(&mut self, new_bounds: Bounds)
    {
        self.grid_mut().change_bounds(new_bounds);
        self.schedule_recompute();
    }

    fn schedule_recompute(&mut self)
    {
        self.set_task(RedrawMessage::Recompute);
    }

    fn schedule_redraw(&mut self)
    {
        if let RedrawMessage::DoNothing = self.get_task()
        {
            self.set_task(RedrawMessage::Redraw);
        }
    }

    fn resize_x(&mut self, width: usize)
    {
        self.grid_mut().resize_x(width);
        self.schedule_recompute();
    }

    fn resize_y(&mut self, height: usize)
    {
        self.grid_mut().resize_y(height);
        self.schedule_recompute();
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

    fn recompute(&mut self);

    fn redraw(&mut self)
    {
        let image = self.get_iter_plane().render(self.get_coloring());
        let image_frame = self.get_frame_mut();
        image_frame.image = RetainedImage::from_color_image("Parameter Plane", image);
    }

    fn zoom(&mut self, scale: RealNum, base_point: ComplexNum)
    {
        self.grid_mut().zoom(scale, base_point);
        self.schedule_recompute();
    }

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

    // TODO: remove unnecessry mutation
    fn save_image(&mut self, img_width: usize, filename: &str)
    {
        let orig_width = self.grid().res_x;
        self.grid_mut().resize_x(img_width);
        let iter_plane = self.plane().compute();
        let filepath = format!("images/{}", filename);
        iter_plane.save(self.get_coloring(), filepath);
        self.grid_mut().resize_x(orig_width);
    }
}

pub(super) struct Parent
{
    pub plane: Box<dyn ParameterPlane>,
    pub coloring: Coloring,
    iter_plane: IterPlane,
    pub image_frame: ImageFrame,
    task: RedrawMessage,
    selection: ComplexNum,
    marked_curves: Vec<ColoredCurve>,
    marked_points: ColoredPoints,
    marked_info: Option<OrbitInfo>,
}
impl Parent
{
    #[must_use]
    pub fn new(plane: Box<dyn ParameterPlane>, coloring: Coloring) -> Self
    {
        let iter_plane = plane.compute();
        let task = RedrawMessage::Redraw;
        let selection = ComplexNum::new(-1., 0.);

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
        }
    }
}

impl Pane for Parent
{
    #[inline]
    fn plane(&self) -> &Box<dyn ParameterPlane>
    {
        &self.plane
    }
    #[inline]
    fn plane_mut(&mut self) -> &mut Box<dyn ParameterPlane>
    {
        &mut self.plane
    }
    #[inline]
    fn get_task(&self) -> RedrawMessage
    {
        self.task
    }
    #[inline]
    fn set_task(&mut self, new_task: RedrawMessage)
    {
        self.task = new_task;
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
    fn get_iter_plane(&self) -> &IterPlane
    {
        &self.iter_plane
    }
    #[inline]
    fn get_iter_plane_mut(&mut self) -> &mut IterPlane
    {
        &mut self.iter_plane
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
    fn get_marked_info(&self) -> &Option<OrbitInfo>
    {
        &self.marked_info
    }
    #[inline]
    fn get_marked_info_mut(&mut self) -> &mut Option<OrbitInfo>
    {
        &mut self.marked_info
    }
    #[inline]
    fn set_marked_info(&mut self, info: OrbitInfo) {
        self.marked_info = Some(info);
    }
    #[inline]
    fn del_marked_info(&mut self) {
        self.marked_info = None;
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
    fn get_selection(&self) -> ComplexNum
    {
        self.selection
    }
    #[inline]
    fn select_point(&mut self, point: ComplexNum)
    {
        self.selection = point;
    }
    #[inline]
    fn recompute(&mut self)
    {
        self.iter_plane = self.plane.compute();
    }
}

impl Default for Parent
{
    fn default() -> Self
    {
        // let plane = Box::new(QuadRatPer2::new_default(1024, 1024).misiurewicz_curve(2,1));
        let plane = Box::new(QuadRatPer2::new_default(1024, 1024));
        let coloring = Coloring::default();

        Self::new(plane, coloring)
    }
}

pub(super) struct Child
{
    pub plane: Box<dyn ParameterPlane>,
    pub coloring: Coloring,
    iter_plane: IterPlane,
    pub image_frame: ImageFrame,
    task: RedrawMessage,
    selection: ComplexNum,
    marked_curves: Vec<ColoredCurve>,
    marked_points: ColoredPoints,
    marked_info: Option<OrbitInfo>,
}
impl Child
{
    pub fn set_param(&mut self, new_param: ComplexNum)
    {
        self.plane.set_param(new_param);
        self.schedule_recompute();
        self.clear_marked_curves();
    }

    #[must_use]
    pub fn new(plane: Box<dyn ParameterPlane>, coloring: Coloring) -> Self
    {
        let iter_plane = plane.compute();
        let task = RedrawMessage::Redraw;
        let selection = ComplexNum::new(0., 0.);

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
        }
    }
}

impl Pane for Child
{
    #[inline]
    fn plane(&self) -> &Box<dyn ParameterPlane>
    {
        &self.plane
    }
    #[inline]
    fn plane_mut(&mut self) -> &mut Box<dyn ParameterPlane>
    {
        &mut self.plane
    }
    #[inline]
    fn get_task(&self) -> RedrawMessage
    {
        self.task
    }
    #[inline]
    fn set_task(&mut self, new_task: RedrawMessage)
    {
        self.task = new_task;
    }
    #[inline]
    fn grid(&self) -> PointGrid
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
    fn get_iter_plane(&self) -> &IterPlane
    {
        &self.iter_plane
    }
    #[inline]
    fn get_iter_plane_mut(&mut self) -> &mut IterPlane
    {
        &mut self.iter_plane
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
    fn get_marked_info(&self) -> &Option<OrbitInfo>
    {
        &self.marked_info
    }
    #[inline]
    fn get_marked_info_mut(&mut self) -> &mut Option<OrbitInfo>
    {
        &mut self.marked_info
    }
    #[inline]
    fn set_marked_info(&mut self, info: OrbitInfo) {
        self.marked_info = Some(info);
    }
    #[inline]
    fn del_marked_info(&mut self) {
        self.marked_info = None;
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
    fn get_selection(&self) -> ComplexNum
    {
        self.selection
    }
    #[inline]
    fn select_point(&mut self, point: ComplexNum)
    {
        self.selection = point;
        self.schedule_recompute();
    }
    #[inline]
    fn recompute(&mut self)
    {
        self.iter_plane = self.plane.compute();
    }
    fn name(&self) -> String
    {
        format!("{}: c = {}", self.plane.name(), self.plane.get_param())
    }
}
