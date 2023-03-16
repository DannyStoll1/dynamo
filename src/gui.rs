use crate::palette::ColorPalette;
use crate::point_grid::*;
use crate::primitive_types::*;
use crate::profiles::*;
use crate::traits::*;
use eframe::egui;
use egui::*;
use egui_extras::{Column, RetainedImage, TableBuilder};

pub fn run_gui() -> Result<(), eframe::Error> {
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
enum RedrawMessage {
    DoNothing,
    Redraw,
    Recompute,
}

struct ImageFrame {
    image: RetainedImage,
    region: Rect,
}
impl ImageFrame {
    fn show(&self, ui: &mut Ui) {
        self.image.show(ui);
    }
    fn height(&self) -> usize {
        self.image.height()
    }
    fn width(&self) -> usize {
        self.image.height()
    }
    fn image_dims(&self) -> Vec2 {
        Vec2 {
            x: self.image.width() as f32,
            y: self.image.height() as f32,
        }
    }
    fn set_position(&mut self, anchor: Pos2) {
        self.region.min = anchor;
        self.region.max = anchor + self.image_dims();
    }
    fn put(&mut self, ui: &mut Ui) {
        let anchor = ui.cursor().min;
        self.set_position(anchor);
        self.show(ui);
    }
    fn to_local_coords(&self, absolute_pos: Pos2) -> Vec2 {
        absolute_pos - self.region.min
    }
    fn to_global_coords(&self, local_pos: Vec2) -> Pos2 {
        self.region.min + local_pos
    }
}

trait GuiPlane {
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

    fn name(&self) -> String {
        self.plane().name()
    }

    fn grid(&self) -> PointGrid {
        self.plane().point_grid()
    }

    fn grid_mut(&mut self) -> &mut PointGrid {
        self.plane_mut().point_grid_mut()
    }

    fn select_point(&mut self, point: ComplexNum);

    fn rescale(&mut self, new_bounds: Bounds) {
        self.grid_mut().rescale(new_bounds);
        self.schedule_recompute();
    }

    fn schedule_recompute(&mut self) {
        self.set_task(RedrawMessage::Recompute);
    }

    fn schedule_redraw(&mut self) {
        if let RedrawMessage::DoNothing = self.get_task() {
            self.set_task(RedrawMessage::Redraw);
        }
    }

    fn resize_x(&mut self, width: usize) {
        self.grid_mut().resize_x(width);
        self.schedule_recompute();
    }

    fn resize_y(&mut self, height: usize) {
        self.grid_mut().resize_y(height);
        self.schedule_recompute();
    }

    fn change_palette(&mut self, palette: ColorPalette) {
        *self.get_palette_mut() = palette;
        self.schedule_redraw();
    }

    fn recompute(&mut self);

    fn redraw(&mut self) {
        let image = self.get_iter_plane().render(self.get_palette());
        let image_frame = self.get_frame_mut();
        (*image_frame).image = RetainedImage::from_color_image("Parameter Plane", image);
    }

    fn zoom(&mut self, scale: RealNum, base_point: ComplexNum) {
        self.grid_mut().zoom(scale, base_point);
        self.schedule_recompute();
    }

    fn process_task(&mut self) {
        let task = self.get_task();
        match task {
            RedrawMessage::Recompute => {
                self.recompute();
                self.redraw();
            }
            RedrawMessage::Redraw => {
                self.redraw();
            }
            RedrawMessage::DoNothing => {}
        }
        self.set_task(RedrawMessage::DoNothing);
    }

    fn frame_contains_pixel(&self, pointer_pos: Pos2) -> bool {
        self.get_frame().region.contains(pointer_pos)
    }

    fn map_pixel(&self, pointer_pos: Pos2) -> ComplexNum {
        let relative_pos = self.get_frame().to_local_coords(pointer_pos);
        self.grid().map_vec2(relative_pos)
    }

    fn process_mouse_input(
        &mut self,
        pointer_value: ComplexNum,
        zoom_factor: f32,
        reselect_point: bool,
    ) {
        if zoom_factor != 1.0 {
            self.zoom((1. / zoom_factor).into(), pointer_value);
        }

        if reselect_point {
            self.select_point(pointer_value);
        }
    }
}

struct Parent {
    plane: Box<dyn ParameterPlane>,
    palette: ColorPalette,
    iter_plane: IterPlane,
    image_frame: ImageFrame,
    task: RedrawMessage,
    selection: ComplexNum,
}
impl Parent {
    fn make_child(&self) -> Child {
        let iter_plane = self.plane.compute_child(self.selection);
        let task = RedrawMessage::Redraw;
        let selection_child = ComplexNum::new(0., 0.);

        let image =
            RetainedImage::from_color_image("dynamical plane", iter_plane.render(self.palette));
        let frame = ImageFrame {
            image,
            region: Rect::NOTHING,
        };
        let plane = dyn_clone::clone_box(&*self.plane);
        Child {
            plane,
            palette: self.palette,
            iter_plane,
            image_frame: frame,
            task,
            selection: selection_child,
            selection_parent: self.selection,
        }
    }
}

impl GuiPlane for Parent {
    fn plane(&self) -> &Box<dyn ParameterPlane> {
        &self.plane
    }
    fn plane_mut(&mut self) -> &mut Box<dyn ParameterPlane> {
        &mut self.plane
    }
    fn get_task(&self) -> RedrawMessage {
        self.task
    }
    fn set_task(&mut self, new_task: RedrawMessage) {
        self.task = new_task;
    }
    fn get_frame(&self) -> &ImageFrame {
        &self.image_frame
    }
    fn get_frame_mut(&mut self) -> &mut ImageFrame {
        &mut self.image_frame
    }
    fn get_iter_plane(&self) -> &IterPlane {
        &self.iter_plane
    }
    fn get_iter_plane_mut(&mut self) -> &mut IterPlane {
        &mut self.iter_plane
    }
    fn get_palette(&self) -> ColorPalette {
        self.palette
    }
    fn get_palette_mut(&mut self) -> &mut ColorPalette {
        &mut self.palette
    }
    fn select_point(&mut self, point: ComplexNum) {
        self.selection = point
    }
    fn recompute(&mut self) {
        self.iter_plane = self.plane.compute();
    }
}

impl Default for Parent {
    fn default() -> Self {
        let plane = Box::new(Mandelbrot::new_default(1024, 1024));
        let iter_plane = plane.compute();
        let task = RedrawMessage::Redraw;
        let selection = ComplexNum::new(-1., 0.);
        let palette = ColorPalette::default();

        let image = RetainedImage::from_color_image("parameter plane", iter_plane.render(palette));
        let frame = ImageFrame {
            image,
            region: Rect::NOTHING,
        };
        Self {
            plane,
            palette,
            iter_plane,
            image_frame: frame,
            task,
            selection,
        }
    }
}

struct Child {
    plane: Box<dyn ParameterPlane>,
    palette: ColorPalette,
    iter_plane: IterPlane,
    image_frame: ImageFrame,
    task: RedrawMessage,
    selection: ComplexNum,
    selection_parent: ComplexNum,
}
impl Child {
    fn set_param(&mut self, new_param: ComplexNum) {
        self.selection_parent = new_param;
        self.schedule_recompute();
    }
}

impl GuiPlane for Child {
    fn plane(&self) -> &Box<dyn ParameterPlane> {
        &self.plane
    }
    fn plane_mut(&mut self) -> &mut Box<dyn ParameterPlane> {
        &mut self.plane
    }
    fn get_task(&self) -> RedrawMessage {
        self.task
    }
    fn set_task(&mut self, new_task: RedrawMessage) {
        self.task = new_task;
    }
    fn grid(&self) -> PointGrid {
        self.plane.point_grid_child()
    }
    fn grid_mut(&mut self) -> &mut PointGrid {
        self.plane.point_grid_child_mut()
    }
    fn get_frame(&self) -> &ImageFrame {
        &self.image_frame
    }
    fn get_frame_mut(&mut self) -> &mut ImageFrame {
        &mut self.image_frame
    }
    fn get_iter_plane(&self) -> &IterPlane {
        &self.iter_plane
    }
    fn get_iter_plane_mut(&mut self) -> &mut IterPlane {
        &mut self.iter_plane
    }
    fn get_palette(&self) -> ColorPalette {
        self.palette
    }
    fn get_palette_mut(&mut self) -> &mut ColorPalette {
        &mut self.palette
    }
    fn select_point(&mut self, point: ComplexNum) {
        self.selection = point;
        self.schedule_recompute();
    }
    fn recompute(&mut self) {
        self.iter_plane = self.plane.compute_child(self.selection_parent);
    }
    fn name(&self) -> String {
        format!("Child: c = {}", self.selection_parent)
    }
}

struct FractalApp {
    parent: Parent,
    child: Child,
    live_mode: bool,
}

impl FractalApp {
    fn toggle_live_mode(&mut self) {
        self.live_mode = !self.live_mode;
    }

    fn randomize_palette(&mut self) {
        let palette = ColorPalette::new_random(0.45, 0.38);
        self.parent.change_palette(palette);
        self.child.change_palette(palette);
    }

    fn process_tasks(&mut self) {
        self.parent.process_task();
        self.child.process_task();
    }

    fn handle_input(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.key_pressed(Key::R)) {
            self.randomize_palette();
        }

        if ctx.input(|i| i.key_pressed(Key::Z)) {
            self.parent.zoom(0.8, self.parent.selection);
        }

        if ctx.input(|i| i.key_pressed(Key::V)) {
            self.parent.zoom(1.25, self.parent.selection);
        }

        if ctx.input(|i| i.key_pressed(Key::L)) {
            self.toggle_live_mode();
        }

        if ctx.input(|i| i.key_pressed(Key::N)) {
            let iters = self.parent.plane.max_iter_mut();
            *iters *= 2;
            self.parent.schedule_recompute();
            self.child.schedule_recompute();
        }
        self.handle_mouse(ctx);
    }

    fn handle_mouse(&mut self, ctx: &egui::Context) {
        let clicked = ctx.input(|i| i.pointer.any_click());
        let zoom_factor = ctx.input(|i| i.zoom_delta());

        if let Some(pointer_pos) = ctx.pointer_latest_pos() {
            if self.parent.frame_contains_pixel(pointer_pos) {
                let reselect_point = self.live_mode || clicked;
                let pointer_value = self.parent.map_pixel(pointer_pos);
                self.parent
                    .process_mouse_input(pointer_value, zoom_factor, reselect_point);
                if reselect_point {
                    self.child.set_param(pointer_value);
                }

                if clicked {
                    let result = self.parent.plane.run_point((0.).into(), pointer_value);
                    dbg!(result);
                }
            } else if self.child.frame_contains_pixel(pointer_pos) {
                let pointer_value = self.child.map_pixel(pointer_pos);
                self.child
                    .process_mouse_input(pointer_value, zoom_factor, false);
            }
        }
    }
}

impl Default for FractalApp {
    fn default() -> Self {
        let parent = Parent::default();
        let child = parent.make_child();

        Self {
            parent,
            child,
            live_mode: false,
        }
    }
}

impl eframe::App for FractalApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
                        });
                        row.col(|ui| {
                            self.child.image_frame.put(ui);
                        });
                    });
                });
        });
    }
}
