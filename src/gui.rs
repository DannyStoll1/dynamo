use crate::palette::ColorPalette;
use crate::point_grid::*;
use crate::primitive_types::*;
use crate::profiles::*;
use crate::traits::*;
use eframe::egui;
use egui::*;
use egui_extras::{RetainedImage, TableBuilder, Column};

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

struct FractalApp {
    parent_plane: Box<dyn ParameterPlane>,
    palette: ColorPalette,
    iter_plane_parent: IterPlane,
    iter_plane_child: IterPlane,
    image_parent: RetainedImage,
    image_child: RetainedImage,
    task_parent: RedrawMessage,
    task_child: RedrawMessage,
    selection_parent: ComplexNum,
    selection_child: ComplexNum,
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

    fn select_point_parent(&mut self, xval: usize, yval: usize) {}

    fn select_point_child(&mut self, xval: usize, yval: usize) {}

    fn rescale_parent(&mut self, x0: f64, x1: f64, y0: f64, y1: f64) {
        self.grid_parent().rescale(x0, x1, y0, y1);
        self.task_parent = RedrawMessage::Recompute;
    }

    fn rescale_child(&mut self, x0: f64, x1: f64, y0: f64, y1: f64) {
        self.grid_child().rescale(x0, x1, y0, y1);
        self.task_child = RedrawMessage::Recompute;
    }

    fn schedule_redraw(&mut self, do_parent: bool, do_child: bool) {
        if do_parent {
            match self.task_parent {
                RedrawMessage::DoNothing => {
                    self.task_parent = RedrawMessage::Redraw;
                }
                _ => {}
            }
        }
        if do_child {
            match self.task_child {
                RedrawMessage::DoNothing => {
                    self.task_child = RedrawMessage::Redraw;
                }
                _ => {}
            }
        }
    }

    fn resize(&mut self, width: usize) {
        self.grid_parent().resize(width);
        self.grid_child().resize(width);
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
        self.image_parent = RetainedImage::from_color_image("Parameter Plane", image);
    }

    fn redraw_child(&mut self) {
        let image = self.iter_plane_child.render(self.palette);
        self.image_child = RetainedImage::from_color_image("Dynamical Plane", image);
    }

    fn zoom_parent(&mut self, scale: f64) {
        let base_point = self.selection_parent;
        self.grid_parent_mut().zoom(scale, base_point);
        self.task_parent = RedrawMessage::Recompute;
    }

    fn zoom_child(&mut self, scale: f64) {
        let base_point = self.selection_parent;
        self.grid_child_mut().zoom(scale, base_point);
        self.task_child = RedrawMessage::Recompute;
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
}

impl Default for FractalApp {
    fn default() -> Self {
        let image_width = 768;
        let default_plane = Mandelbrot::new_default(image_width, 2048).marked_cycle_curve(4);
        let palette = ColorPalette::new_with_contrast(3.0, 8.0, 5.0, 0.45, 0.38);

        let selection_param = ComplexNum::new(0., 1.);
        let selection_julia = ComplexNum::new(0., 0.);

        let iter_plane_parent = default_plane.compute();
        let iter_plane_child = default_plane.compute_child(selection_param);

        let parent_image =
            RetainedImage::from_color_image("parameter plane", iter_plane_parent.render(palette));
        let child_image =
            RetainedImage::from_color_image("dynamical plane", iter_plane_child.render(palette));

        Self {
            palette,
            parent_plane: Box::from(default_plane),
            iter_plane_parent,
            iter_plane_child,
            image_parent: parent_image,
            image_child: child_image,
            task_parent: RedrawMessage::Recompute,
            task_child: RedrawMessage::Recompute,
            selection_parent: selection_param,
            selection_child: selection_julia,
        }
    }
}

impl eframe::App for FractalApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.key_pressed(Key::R)) {
            self.randomize_palette();
        }

        if ctx.input(|i| i.key_pressed(Key::Z)) {
            self.zoom_parent(0.8);
        }

        if ctx.input(|i| i.key_pressed(Key::V)) {
            self.zoom_parent(1.25);
        }

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
                        ui.heading(format!("Child (c={})", self.selection_parent));
                    });
                })
                .body(|mut body| {
                    body.row(self.image_parent.height() as f32, |mut row| {
                        row.col(|ui| {
                            self.image_parent.show(ui);
                        });
                        row.col(|ui| {
                            self.image_child.show(ui);
                        });
                    });
                });
        });

        // egui::CentralPanel::default().show(ctx, |ui| {
        //     ui.heading("This is an image:");
        //     self.image.show(ui);
        //
        //     // ui.heading("This is a rotated image with a tint:");
        //     // ui.add(
        //     //     egui::Image::new(self.image.texture_id(ctx), self.image.size_vec2())
        //     //         .rotate(45.0_f32.to_radians(), egui::Vec2::splat(0.5))
        //     //         .tint(self.tint),
        //     // );
        //     //
        //     // ui.horizontal(|ui| {
        //     //     ui.label("Tint:");
        //     //     egui::color_picker::color_edit_button_srgba(
        //     //         ui,
        //     //         &mut self.tint,
        //     //         egui::color_picker::Alpha::BlendOrAdditive,
        //     //     );
        //     // });
        //
        //     ui.heading("This is an image you can click:");
        //     ui.add(egui::ImageButton::new(
        //         self.image.texture_id(ctx),
        //         self.image.size_vec2(),
        //     ));
        // });
    }
}
