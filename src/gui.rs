use crate::palette::ColorPalette;
use crate::primitive_types::*;
use crate::profiles::Mandelbrot;
use crate::traits::*;
use eframe::egui;
use egui_extras::RetainedImage;

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

struct FractalApp {
    image_width: usize,
    parameter_plane: Mandelbrot,
    parameter_image: RetainedImage,
    dynamical_image: RetainedImage,
    palette: ColorPalette,
    redraw_par: bool,
    redraw_dyn: bool,
    selection_param: ComplexNum,
    selection_julia: ComplexNum,
}

impl FractalApp {
    fn select_point_param(xval: usize, yval: usize) {}

    fn select_point_julia(xval: usize, yval: usize) {}

    fn rescale_param(x0: usize, x1: usize, y0: usize, y1: usize) {}

    fn rescale_julia(x0: usize, x1: usize, y0: usize, y1: usize) {}
}

impl Default for FractalApp {
    fn default() -> Self {
        let image_width = 768;
        let mandelbrot = Mandelbrot::new_default(image_width, 1024);
        let palette = ColorPalette::new_with_contrast(3.0, 8.0, 5.0, 0.45, 0.38);

        let selection_param = ComplexNum::new(-1., 0.);
        let selection_julia = ComplexNum::new(0., 0.);

        let parameter_iters = mandelbrot.compute();
        let dynamical_iters = mandelbrot.compute_julia(selection_param);

        let parameter_image =
            RetainedImage::from_color_image("parameter plane", parameter_iters.render(palette));
        let dynamical_image =
            RetainedImage::from_color_image("dynamical plane", dynamical_iters.render(palette));
        Self {
            image_width,
            palette,
            parameter_plane: mandelbrot,
            parameter_image,
            dynamical_image,
            redraw_par: true,
            redraw_dyn: true,
            selection_param,
            selection_julia,
        }
    }
}

impl eframe::App for FractalApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Parameter Plane");
            if self.redraw_par {
                let iter_plane = self.parameter_plane.compute();
                let image = iter_plane.render(self.palette);
                self.parameter_image = RetainedImage::from_color_image("Parameter Plane", image);
                self.redraw_par = false;
            }
            self.parameter_image.show(ui);
        });

        egui::SidePanel::right("dynamical plane").show(ctx, |ui| {
            ui.heading("Dynamical Plane");
            if self.redraw_dyn {
                let iter_plane = self.parameter_plane.compute_julia(self.selection_param);
                let image = iter_plane.render(self.palette);
                self.dynamical_image = RetainedImage::from_color_image("Dynamical Plane", image);
                self.redraw_dyn = false;
            }
            self.dynamical_image.show(ui);
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
