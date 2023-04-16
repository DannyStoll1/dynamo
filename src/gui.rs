use crate::dynamics::{julia::JuliaSet, orbit::Orbit, HasDynamicalCovers, ParameterPlane};
use crate::iter_plane::{FractalImage, IterPlane};
use crate::palette::ColorPalette;
use crate::point_grid::{Bounds, PointGrid};
use crate::primitive_types::{ComplexNum, RealNum};
use crate::profiles::*;
use eframe::egui;
use egui::{epaint, Color32, InputState, Key, Pos2, Rect, Stroke, Ui, Vec2};
use egui_extras::{Column, RetainedImage, TableBuilder};
use epaint::PathShape;

pub type ColoredCurve = (Vec<ComplexNum>, Color32);

pub fn run_app() -> Result<(), eframe::Error>
{
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(300.0, 900.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Show an image with eframe/egui",
        options,
        Box::new(|_cc| Box::<FractalApp>::default()),
    )
}

#[derive(Clone, Copy, Debug)]
enum RedrawMessage
{
    DoNothing,
    Redraw,
    Recompute,
}

struct ImageFrame
{
    image: RetainedImage,
    region: Rect,
}
impl ImageFrame
{
    fn show(&self, ui: &mut Ui)
    {
        self.image.show(ui);
    }
    fn height(&self) -> usize
    {
        self.image.height()
    }
    fn width(&self) -> usize
    {
        self.image.height()
    }
    fn image_dims(&self) -> Vec2
    {
        Vec2 {
            x: self.image.width() as f32,
            y: self.image.height() as f32,
        }
    }
    fn set_position(&mut self, anchor: Pos2)
    {
        self.region.min = anchor;
        self.region.max = anchor + self.image_dims();
    }
    fn put(&mut self, ui: &mut Ui)
    {
        let anchor = ui.cursor().min;
        self.set_position(anchor);
        self.show(ui);
    }
    fn to_local_coords(&self, absolute_pos: Pos2) -> Vec2
    {
        absolute_pos - self.region.min
    }
    fn to_global_coords(&self, local_pos: Vec2) -> Pos2
    {
        self.region.min + local_pos
    }
}

trait GuiPlane
{
    fn plane(&self) -> &Box<dyn ParameterPlane>;
    fn plane_mut(&mut self) -> &mut Box<dyn ParameterPlane>;

    fn get_task(&self) -> RedrawMessage;
    fn set_task(&mut self, new_task: RedrawMessage);

    fn get_frame(&self) -> &ImageFrame;
    fn get_frame_mut(&mut self) -> &mut ImageFrame;

    fn get_palette(&self) -> ColorPalette;
    fn get_palette_mut(&mut self) -> &mut ColorPalette;

    fn get_iter_plane(&self) -> &IterPlane;
    fn get_iter_plane_mut(&mut self) -> &mut IterPlane;

    fn get_marked_curves(&self) -> &Vec<ColoredCurve>;
    fn get_marked_curves_mut(&mut self) -> &mut Vec<ColoredCurve>;
    fn get_marked_points(&self) -> &Vec<ComplexNum>;
    fn get_marked_points_mut(&mut self) -> &mut Vec<ComplexNum>;
    fn mark_curve(&mut self, zs: Vec<ComplexNum>, color: Color32)
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

    fn select_point(&mut self, point: ComplexNum);

    fn rescale(&mut self, new_bounds: Bounds)
    {
        self.grid_mut().rescale(new_bounds);
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
        *self.get_palette_mut() = palette;
        self.schedule_redraw();
    }

    fn recompute(&mut self);

    fn redraw(&mut self)
    {
        let image = self.get_iter_plane().render(self.get_palette());
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
        if zoom_factor != 1.0
        {
            self.zoom((1. / zoom_factor).into(), pointer_value);
        }

        if reselect_point
        {
            self.select_point(pointer_value);
        }
    }
}

struct Parent
{
    plane: Box<dyn ParameterPlane>,
    palette: ColorPalette,
    iter_plane: IterPlane,
    image_frame: ImageFrame,
    task: RedrawMessage,
    selection: ComplexNum,
    marked_curves: Vec<ColoredCurve>,
    marked_points: Vec<ComplexNum>,
}
impl Parent
{
    fn new(plane: Box<dyn ParameterPlane>, palette: ColorPalette) -> Self
    {
        let iter_plane = plane.compute();
        let task = RedrawMessage::Redraw;
        let selection = ComplexNum::new(-1., 0.);

        let image = RetainedImage::from_color_image("parameter plane", iter_plane.render(palette));
        let frame = ImageFrame {
            image,
            region: Rect::NOTHING,
        };
        let marked_curves = vec![];
        let marked_points = vec![];
        Self {
            plane,
            palette,
            iter_plane,
            image_frame: frame,
            task,
            selection,
            marked_curves,
            marked_points,
        }
    }
}

impl GuiPlane for Parent
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
    fn get_marked_points(&self) -> &Vec<ComplexNum>
    {
        &self.marked_points
    }
    #[inline]
    fn get_marked_points_mut(&mut self) -> &mut Vec<ComplexNum>
    {
        &mut self.marked_points
    }
    #[inline]
    fn get_palette(&self) -> ColorPalette
    {
        self.palette
    }
    #[inline]
    fn get_palette_mut(&mut self) -> &mut ColorPalette
    {
        &mut self.palette
    }
    #[inline]
    fn select_point(&mut self, point: ComplexNum)
    {
        self.selection = point
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
        let palette = ColorPalette::black(32.);

        Self::new(plane, palette)
    }
}

struct Child
{
    plane: Box<dyn ParameterPlane>,
    palette: ColorPalette,
    iter_plane: IterPlane,
    image_frame: ImageFrame,
    task: RedrawMessage,
    selection: ComplexNum,
    marked_curves: Vec<ColoredCurve>,
    marked_points: Vec<ComplexNum>,
}
impl Child
{
    fn set_param(&mut self, new_param: ComplexNum)
    {
        self.plane.set_param(new_param);
        self.schedule_recompute();
        self.clear_marked_curves();
    }

    fn new(plane: Box<dyn ParameterPlane>, palette: ColorPalette) -> Self
    {
        let iter_plane = plane.compute();
        let task = RedrawMessage::Redraw;
        let selection = ComplexNum::new(0., 0.);

        let image = RetainedImage::from_color_image("parameter plane", iter_plane.render(palette));
        let frame = ImageFrame {
            image,
            region: Rect::NOTHING,
        };

        let marked_curves = vec![];
        let marked_points = vec![];

        Self {
            plane,
            palette,
            iter_plane,
            image_frame: frame,
            task,
            selection,
            marked_curves,
            marked_points,
        }
    }
}

impl GuiPlane for Child
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
    fn get_marked_points(&self) -> &Vec<ComplexNum>
    {
        &self.marked_points
    }
    #[inline]
    fn get_marked_points_mut(&mut self) -> &mut Vec<ComplexNum>
    {
        &mut self.marked_points
    }
    #[inline]
    fn get_palette(&self) -> ColorPalette
    {
        self.palette
    }
    #[inline]
    fn get_palette_mut(&mut self) -> &mut ColorPalette
    {
        &mut self.palette
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

struct FractalApp
{
    parent: Parent,
    child: Child,
    live_mode: bool,
}

impl FractalApp
{
    fn toggle_live_mode(&mut self)
    {
        self.live_mode = !self.live_mode;
    }

    fn randomize_palette(&mut self)
    {
        let palette = ColorPalette::new_random(0.45, 0.38);
        self.parent.change_palette(palette);
        self.child.change_palette(palette);
    }

    fn process_tasks(&mut self)
    {
        self.parent.process_task();
        self.child.process_task();
    }

    fn handle_input(&mut self, ctx: &egui::Context)
    {
        if ctx.input(|i| i.key_pressed(Key::R))
        {
            self.randomize_palette();
        }

        if ctx.input(|i| i.key_pressed(Key::Z))
        {
            self.parent.zoom(0.8, self.parent.selection);
        }

        if ctx.input(|i| i.key_pressed(Key::V))
        {
            self.parent.zoom(1.25, self.parent.selection);
        }

        if ctx.input(|i| i.key_pressed(Key::C))
        {
            self.parent.clear_marked_curves();
            self.child.clear_marked_curves();
        }

        if ctx.input(|i| i.key_pressed(Key::L))
        {
            self.toggle_live_mode();
        }

        if ctx.input(|i| i.key_pressed(Key::N))
        {
            let iters = self.parent.plane.max_iter_mut();
            *iters *= 2;
            self.parent.schedule_recompute();
            self.child.schedule_recompute();
        }
        self.handle_mouse(ctx);
    }

    fn handle_mouse(&mut self, ctx: &egui::Context)
    {
        let clicked = ctx.input(|i| i.pointer.any_click());
        let zoom_factor = ctx.input(InputState::zoom_delta);

        if let Some(pointer_pos) = ctx.pointer_latest_pos()
        {
            if self.parent.frame_contains_pixel(pointer_pos)
            {
                let reselect_point = self.live_mode || clicked;
                let pointer_value = self.parent.map_pixel(pointer_pos);
                self.parent
                    .process_mouse_input(pointer_value, zoom_factor, reselect_point);
                if reselect_point
                {
                    let child_param = self.parent.plane.param_map(pointer_value);
                    self.child.set_param(child_param);
                }

                if clicked
                {
                    // if let Some(ray) = self.parent.plane.external_ray(1. / 7., 20, 250, 500) {
                    // if let Some(theta) = self.parent.plane.external_angle(pointer_value) {
                    //     dbg!(theta);
                    // }
                    // if let Some(ray) = self.parent.plane.external_ray(pointer_value.arg(), 200, 25, 600) {
                    //     self.parent.mark_curve(ray, Color32::GREEN);
                    // }
                    self.parent.clear_marked_curves();
                    let pointer_param = self.parent.plane.param_map(pointer_value);
                    let orbit = self.parent.plane.get_orbit(pointer_param);
                    self.parent.mark_curve(orbit, Color32::GREEN);
                    // dbg!(orbit);
                }
            }
            else if self.child.frame_contains_pixel(pointer_pos)
            {
                let pointer_value = self.child.map_pixel(pointer_pos);
                self.child
                    .process_mouse_input(pointer_value, zoom_factor, false);

                if clicked
                {
                    // if let Some(ray) = self.child.plane.external_ray(pointer_value.arg(), 200, 25, 600) {
                    //     self.child.mark_curve(ray, Color32::GREEN);
                    // }
                    self.child.clear_marked_curves();
                    let orbit = self.child.plane.get_orbit(pointer_value);
                    self.child.mark_curve(orbit, Color32::GREEN);
                    // let orbit = self.child.plane.get_orbit_info(pointer_value);
                }
            }
        }
    }
}

impl Default for FractalApp
{
    fn default() -> Self
    {
        // let parameter_plane = QuadRatPer2::new_default(1024, 2048).marked_cycle_curve(5);
        // let parameter_plane = QuadRatPer2::new_default(1024, 2048);
        // let parameter_plane = Mandelbrot::new_default(1024, 2048).marked_cycle_curve(4);
        // let biquadratic_param = ComplexNum::new(-1., 0.);
        // let parameter_plane = Biquadratic::new_default(1024, 2048, biquadratic_param);
        let parameter_plane = CubicPer2CritMarked::new_default(1024, 2048);
        let mut dynamical_plane = JuliaSet::from(parameter_plane);
        dynamical_plane.set_param((1.).into());
        // let parameter_plane = dynamical_plane.clone();
        let palette = ColorPalette::black(32.);

        let parent = Parent::new(Box::new(parameter_plane), palette);
        let child = Child::new(Box::new(dynamical_plane), palette);

        Self {
            parent,
            child,
            live_mode: false,
        }
    }
}

impl eframe::App for FractalApp
{
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame)
    {
        self.handle_input(ctx);
        self.process_tasks();

        egui::CentralPanel::default().show(ctx, |ui| {
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
                    body.row(self.parent.image_frame.height() as f32, |mut row| {
                        row.col(|ui| {
                            self.parent.image_frame.put(ui);
                            self.parent.put_marked_curves(ui);
                        });
                        row.col(|ui| {
                            self.child.image_frame.put(ui);
                            self.child.put_marked_curves(ui);
                        });
                    });
                });
        });
    }
}
