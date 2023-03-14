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

enum RedrawMessage {
    DoNothing,
    Redraw,
    Recompute,
}

enum PlaneID {
    Parent,
    Child,
}

struct ImageFrame {
    image: RetainedImage,
    frame: Rect,
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
    fn reposition(&mut self, anchor: Pos2) {
        self.frame.min = anchor;
        self.frame.max = anchor + self.image_dims();
    }
    fn to_local_coords(&self, absolute_pos: Pos2) -> Vec2 {
        absolute_pos - self.frame.min
    }
    fn from_local_coords(&self, local_pos: Vec2) -> Pos2 {
        self.frame.min + local_pos
    }
}

struct FractalApp {
    parent_plane: Box<dyn ParameterPlane>,
    palette: ColorPalette,
    iter_plane_parent: IterPlane,
    iter_plane_child: IterPlane,
    frame_parent: ImageFrame,
    frame_child: ImageFrame,
    task_parent: RedrawMessage,
    task_child: RedrawMessage,
    selection_parent: ComplexNum,
    selection_child: ComplexNum,
    live_mode: bool,
}

impl FractalApp {
    fn grid_parent(&self) -> PointGrid {
        self.parent_plane.point_grid()
    }

    fn grid_child(&self) -> PointGrid {
        self.parent_plane.point_grid_child()
    }

    fn grid_parent_mut(&mut self) -> &mut PointGrid {
        self.parent_plane.point_grid_mut()
    }

    fn grid_child_mut(&mut self) -> &mut PointGrid {
        self.parent_plane.point_grid_child_mut()
    }

    fn select_point_parent(&mut self, point: ComplexNum) {
        self.selection_parent = point
    }

    fn select_point_child(&mut self, point: ComplexNum) {
        self.selection_child = point
    }

    fn rescale_parent(&mut self, new_bounds: Bounds) {
        self.grid_parent().rescale(new_bounds);
        self.task_parent = RedrawMessage::Recompute;
    }

    fn rescale_child(&mut self, new_bounds: Bounds) {
        self.grid_child().rescale(new_bounds);
        self.task_child = RedrawMessage::Recompute;
    }

    fn toggle_live_mode(&mut self) {
        self.live_mode = !self.live_mode;
    }

    fn schedule_redraw(&mut self, do_parent: bool, do_child: bool) {
        if do_parent {
            if let RedrawMessage::DoNothing = self.task_parent {
                self.task_parent = RedrawMessage::Redraw;
            }
        }
        if do_child {
            if let RedrawMessage::DoNothing = self.task_child {
                self.task_child = RedrawMessage::Redraw;
            }
        }
    }

    fn resize_x(&mut self, width: usize) {
        self.grid_parent().resize_x(width);
        self.grid_child().resize_x(width);
        self.task_parent = RedrawMessage::Recompute;
        self.task_child = RedrawMessage::Recompute;
    }

    fn resize_y(&mut self, height: usize) {
        self.grid_parent().resize_y(height);
        self.grid_child().resize_y(height);
        self.task_parent = RedrawMessage::Recompute;
        self.task_child = RedrawMessage::Recompute;
    }

    fn change_palette(&mut self, palette: ColorPalette) {
        self.palette = palette;
        self.schedule_redraw(true, true);
    }

    fn randomize_palette(&mut self) {
        let palette = ColorPalette::new_random(0.45, 0.38);
        self.change_palette(palette);
    }

    fn recompute_parent(&mut self) {
        self.iter_plane_parent = self.parent_plane.compute();
    }

    fn recompute_child(&mut self) {
        self.iter_plane_child = self.parent_plane.compute_child(self.selection_parent);
    }

    fn redraw_parent(&mut self) {
        let image = self.iter_plane_parent.render(self.palette);
        self.frame_parent.image = RetainedImage::from_color_image("Parameter Plane", image);
    }

    fn redraw_child(&mut self) {
        let image = self.iter_plane_child.render(self.palette);
        self.frame_child.image = RetainedImage::from_color_image("Dynamical Plane", image);
    }

    fn zoom_parent(&mut self, scale: Float, base_point: ComplexNum) {
        self.grid_parent_mut().zoom(scale, base_point);
        self.task_parent = RedrawMessage::Recompute;
    }

    fn zoom_child(&mut self, scale: Float, base_point: ComplexNum) {
        self.grid_child_mut().zoom(scale, base_point);
        self.task_child = RedrawMessage::Recompute;
    }

    fn zoom(&mut self, scale: Float, base_point: ComplexNum, plane_id: PlaneID) {
        match plane_id {
            PlaneID::Parent => {
                self.zoom_parent(scale, base_point);
            }
            PlaneID::Child => {
                self.zoom_child(scale, base_point);
            }
        }
    }

    fn process_tasks(&mut self) {
        match self.task_parent {
            RedrawMessage::Recompute => {
                self.recompute_parent();
                self.redraw_parent();
                self.task_parent = RedrawMessage::DoNothing;
            }
            RedrawMessage::Redraw => {
                self.redraw_parent();
                self.task_parent = RedrawMessage::DoNothing;
            }
            RedrawMessage::DoNothing => {}
        }

        match self.task_child {
            RedrawMessage::Recompute => {
                self.recompute_child();
                self.redraw_child();
                self.task_child = RedrawMessage::DoNothing;
            }
            RedrawMessage::Redraw => {
                self.redraw_child();
                self.task_child = RedrawMessage::DoNothing;
            }
            RedrawMessage::DoNothing => {}
        }
    }

    fn handle_input(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.key_pressed(Key::R)) {
            self.randomize_palette();
        }

        if ctx.input(|i| i.key_pressed(Key::Z)) {
            self.zoom_parent(0.8, self.selection_parent);
        }

        if ctx.input(|i| i.key_pressed(Key::V)) {
            self.zoom_parent(1.25, self.selection_parent);
        }

        if ctx.input(|i| i.key_pressed(Key::L)) {
            self.toggle_live_mode();
        }

        if ctx.input(|i| i.key_pressed(Key::N)) {
            let iters = self.parent_plane.max_iter_mut();
            *iters *= 2;
            self.task_parent = RedrawMessage::Recompute;
            self.task_child = RedrawMessage::Recompute;
        }
        self.handle_mouse(ctx);
    }

    fn handle_mouse(&mut self, ctx: &egui::Context) {
        let clicked = ctx.input(|i| i.pointer.any_click());
        let zoom_factor = ctx.input(|i| i.zoom_delta());

        if let Some(pointer_pos) = ctx.pointer_latest_pos() {
            if self.frame_parent.frame.contains(pointer_pos) {
                let relative_pos = self.frame_parent.to_local_coords(pointer_pos);
                let pointer_value = self.grid_parent().map_vec2(relative_pos);

                if clicked || self.live_mode {
                    self.select_point_parent(pointer_value);
                    self.task_child = RedrawMessage::Recompute;
                }
                if zoom_factor != 1.0 {
                    self.zoom_parent((1. / zoom_factor).into(), pointer_value);
                }
            }

            if self.frame_child.frame.contains(pointer_pos) {
                let relative_pos = self.frame_child.to_local_coords(pointer_pos);
                let pointer_value = self.grid_child().map_vec2(relative_pos);

                if clicked {
                    self.select_point_child(pointer_value);
                }
                if zoom_factor != 1.0 {
                    self.zoom_child((1. / zoom_factor).into(), pointer_value);
                }
            }
        }
    }
}

impl Default for FractalApp {
    fn default() -> Self {
        let image_height = 768;
        let default_plane = Mandelbrot::new_default(image_height, 2048).marked_cycle_curve(4);
        // let default_plane = Mandelbrot::new_default(image_height, 2048);
        let palette = ColorPalette::new_with_contrast(3.0, 8.0, 5.0, 0.45, 0.38);

        let selection_parent = ComplexNum::new(0., 1.);
        let selection_child = ComplexNum::new(0., 0.);

        let iter_plane_parent = default_plane.compute();
        let iter_plane_child = default_plane.compute_child(selection_parent);

        let image_parent =
            RetainedImage::from_color_image("parameter plane", iter_plane_parent.render(palette));
        let frame_parent = ImageFrame {
            image: image_parent,
            frame: Rect::NOTHING,
        };
        let image_child =
            RetainedImage::from_color_image("dynamical plane", iter_plane_child.render(palette));
        let frame_child = ImageFrame {
            image: image_child,
            frame: Rect::NOTHING,
        };

        Self {
            palette,
            parent_plane: Box::from(default_plane),
            iter_plane_parent,
            iter_plane_child,
            frame_parent,
            frame_child,
            task_parent: RedrawMessage::Recompute,
            task_child: RedrawMessage::Recompute,
            selection_parent,
            selection_child,
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
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.heading(self.parent_plane.name());
                    });
                    header.col(|ui| {
                        ui.heading(format!("Child: c = {}", self.selection_parent));
                    });
                })
                .body(|mut body| {
                    body.row(self.frame_parent.height() as f32, |mut row| {
                        row.col(|ui| {
                            let anchor = ui.cursor().min;
                            self.frame_parent.reposition(anchor);
                            self.frame_parent.show(ui);
                        });
                        row.col(|ui| {
                            let anchor = ui.cursor().min;
                            self.frame_child.reposition(anchor);
                            self.frame_child.show(ui);
                        });
                    });
                });
        });
    }
}
