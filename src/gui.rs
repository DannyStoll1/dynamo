use crate::coloring::{coloring_algorithm::ColoringAlgorithm, palette::ColorPalette, Coloring};
use crate::dynamics::{julia::JuliaSet, HasDynamicalCovers, ParameterPlane};
use crate::primitive_types::{ComplexNum, Period};
use crate::profiles::*;

use eframe::egui;
use egui::{Color32, InputState, Key};
use egui_extras::{Column, TableBuilder};
use input_macro::input;

pub mod image_frame;
pub mod pane;
use image_frame::ImageFrame;
use pane::{Child, Pane, PaneID, Parent};

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

macro_rules! change_plane {
    ($fractal: expr) => {
        let parent_plane_ref = self.parent.plane_mut();

        let res_y = parent_plane_ref.point_grid().res_y;
        let max_iters = 2048;
        let selected_point = ComplexNum::new(0., 0.);

        let parent_plane = $fractal::new_default(res_y, max_iters);
        *parent_plane_ref = Box::new(parent_plane);

        let child_plane = JuliaSet::new(parent_plane, selected_point);
        let child_plane_ref = self.child.plane_mut();
        *child_plane_ref = Box::new(child_plane);

        self.parent.select_point(selected_point);
        self.parent.schedule_recompute();
        self.child.schedule_recompute();
    };
}

pub struct FractalApp
{
    parent: Parent,
    child: Child,
    live_mode: bool,
    active_pane: PaneID,
}

impl FractalApp
{
    pub fn toggle_live_mode(&mut self)
    {
        self.live_mode = !self.live_mode;
    }

    pub fn randomize_palette(&mut self)
    {
        let palette = ColorPalette::new_random(0.45, 0.38);
        self.parent.change_palette(palette);
        self.child.change_palette(palette);
    }

    pub fn set_palette(&mut self, palette: ColorPalette)
    {
        self.parent.change_palette(palette);
        self.child.change_palette(palette);
    }

    pub fn set_coloring_algorithm(&mut self, coloring_algorithm: ColoringAlgorithm)
    {
        self.parent.set_coloring_algorithm(coloring_algorithm);
        self.child.set_coloring_algorithm(coloring_algorithm);
    }

    fn process_tasks(&mut self)
    {
        self.parent.process_task();
        self.child.process_task();
    }

    fn set_child_param(&mut self, new_param: ComplexNum)
    {
        let old_bounds = self.child.grid().bounds;
        let mut new_bounds = self.parent.plane.default_julia_bounds(new_param);
        let zoom_factor = old_bounds.range_x() / new_bounds.range_x();
        new_bounds.zoom(zoom_factor, new_bounds.center());
        self.child.grid_mut().change_bounds(new_bounds);
        self.child.set_param(new_param);
    }

    fn handle_input(&mut self, ctx: &egui::Context)
    {
        if ctx.input(|i| i.key_pressed(Key::R))
        {
            self.randomize_palette();
        }

        if ctx.input(|i| i.key_pressed(Key::B))
        {
            let black_palette = ColorPalette::black(32.);
            self.set_palette(black_palette);
        }

        if ctx.input(|i| i.key_pressed(Key::W))
        {
            let white_palette = ColorPalette::white(32.);
            self.set_palette(white_palette);
        }

        if ctx.input(|i| i.key_pressed(Key::ArrowUp))
        {
            let pane = self.get_active_pane_mut();
            pane.scale_palette(1.25);
        }

        if ctx.input(|i| i.key_pressed(Key::ArrowDown))
        {
            let pane = self.get_active_pane_mut();
            pane.scale_palette(0.8);
        }

        if ctx.input(|i| i.key_pressed(Key::ArrowLeft))
        {
            let pane = self.get_active_pane_mut();
            pane.shift_palette(-0.02);
        }

        if ctx.input(|i| i.key_pressed(Key::ArrowRight))
        {
            let pane = self.get_active_pane_mut();
            pane.shift_palette(0.02);
        }

        if ctx.input(|i| i.key_pressed(Key::Z))
        {
            let pane = self.get_active_pane_mut();
            pane.zoom(0.8, pane.get_selection());
        }

        if ctx.input(|i| i.key_pressed(Key::V))
        {
            let pane = self.get_active_pane_mut();
            pane.zoom(1.25, pane.get_selection());
        }

        if ctx.input(|i| i.key_pressed(Key::PlusEquals))
        {
            let pane = self.get_active_pane_mut();
            let selection = pane.get_selection();
            pane.grid_mut().recenter(selection);
            pane.schedule_recompute();
        }

        if ctx.input(|i| i.key_pressed(Key::Num0))
        {
            let pane = self.get_active_pane_mut();
            pane.set_coloring_algorithm(ColoringAlgorithm::Solid);
        }

        if ctx.input(|i| i.key_pressed(Key::Num1))
        {
            let pane = self.get_active_pane_mut();
            pane.set_coloring_algorithm(ColoringAlgorithm::Period);
        }

        if ctx.input(|i| i.key_pressed(Key::Num2))
        {
            let pane = self.get_active_pane_mut();
            pane.set_coloring_algorithm(ColoringAlgorithm::PeriodMultiplier);
        }

        if ctx.input(|i| i.key_pressed(Key::Num3))
        {
            let pane = self.get_active_pane_mut();
            pane.set_coloring_algorithm(ColoringAlgorithm::Multiplier);
        }

        if ctx.input(|i| i.key_pressed(Key::Num4))
        {
            let pane = self.get_active_pane_mut();
            pane.set_coloring_algorithm(ColoringAlgorithm::Preperiod);
        }

        if ctx.input(|i| i.key_pressed(Key::Num5))
        {
            let pane = self.get_active_pane_mut();
            let periodicity_tolerance = pane.plane().periodicity_tolerance();
            pane.set_coloring_algorithm(ColoringAlgorithm::PreperiodSmooth {
                periodicity_tolerance,
            });
        }

        if ctx.input(|i| i.key_pressed(Key::C))
        {
            let pane = self.get_active_pane_mut();
            pane.clear_marked_curves();
        }

        if ctx.input(|i| i.key_pressed(Key::L))
        {
            self.toggle_live_mode();
        }

        if ctx.input(|i| i.key_pressed(Key::S))
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

        if ctx.input(|i| i.key_pressed(Key::N))
        {
            let pane = self.get_active_pane_mut();
            let iters = pane.plane_mut().max_iter_mut();
            *iters *= 2;
            pane.schedule_recompute();
        }

        if ctx.input(|i| i.key_pressed(Key::M))
        {
            let pane = self.get_active_pane_mut();
            let iters = pane.plane_mut().max_iter_mut();
            *iters /= 2;
            pane.schedule_recompute();
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
                self.active_pane = PaneID::Parent;
                let reselect_point = self.live_mode || clicked;
                let pointer_value = self.parent.map_pixel(pointer_pos);
                self.parent
                    .process_mouse_input(pointer_value, zoom_factor, reselect_point);
                if reselect_point
                {
                    let child_param = self.parent.plane.param_map(pointer_value);
                    self.set_child_param(child_param);
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
                self.active_pane = PaneID::Child;
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
    fn get_active_pane(&self) -> &dyn Pane
    {
        match self.active_pane
        {
            PaneID::Parent => &self.parent,
            PaneID::Child => &self.child,
        }
    }
    fn get_active_pane_mut(&mut self) -> &mut dyn Pane
    {
        match self.active_pane
        {
            PaneID::Parent => &mut self.parent,
            PaneID::Child => &mut self.child,
        }
    }

    fn change_fractal<T>(&mut self, create_parent_plane: fn(usize, Period) -> T)
    where
        T: ParameterPlane + Copy + 'static,
    {
        let parent_plane_ref = self.parent.plane_mut();
        let child_plane_ref = self.child.plane_mut();

        let res_y = parent_plane_ref.point_grid().res_y;
        let max_iters = 2048;

        let parent_plane = create_parent_plane(res_y, max_iters);
        let selected_point = parent_plane.default_julia_param();
        let child_plane = JuliaSet::new(parent_plane, selected_point);

        *parent_plane_ref = Box::new(parent_plane);
        *child_plane_ref = Box::new(child_plane);

        self.parent.select_point(selected_point);
        self.parent.clear_marked_curves();
        self.child.clear_marked_curves();

        self.parent.schedule_recompute();
        self.child.schedule_recompute();
    }

    fn show_menu(&mut self, ui: &mut egui::Ui)
    {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Save").clicked()
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
            });
            ui.menu_button("Fractal", |ui| {
                ui.menu_button("Polynomials", |ui| {
                    if ui.button("Mandelbrot").clicked()
                    {
                        self.change_fractal(|res, iter| Mandelbrot::new_default(res, iter));
                    }
                    else if ui.button("Odd Cubics").clicked()
                    {
                        self.change_fractal(|res, iter| OddCubic::new_default(res, iter));
                    }
                    else if ui.button("Cubic Per(2, 0)").clicked()
                    {
                        self.change_fractal(|res, iter| {
                            CubicPer2CritMarked::new_default(res, iter)
                        });
                    }
                    else if ui.button("Cubic Per(1, 1)").clicked()
                    {
                        self.change_fractal(|res, iter| CubicPer1_1::new_default(res, iter));
                    }
                    else
                    {
                        return;
                    }
                    ui.close_menu();
                    ui.pointer().take_click();
                });
                ui.menu_button("Quadratic Rational Maps", |ui| {
                    if ui.button("QuadRat Per(2)").clicked()
                    {
                        self.change_fractal(|res, iter| QuadRatPer2::new_default(res, iter));
                    }
                    else if ui.button("QuadRat Per(3)").clicked()
                    {
                        self.change_fractal(|res, iter| QuadRatPer3::new_default(res, iter));
                    }
                    else if ui.button("QuadRat Per(4)").clicked()
                    {
                        self.change_fractal(|res, iter| QuadRatPer4::new_default(res, iter));
                    }
                    else
                    {
                        return;
                    }
                    ui.close_menu();
                    ui.pointer().take_click();
                });
            });
            ui.menu_button("Coloring", |ui| {
                ui.menu_button("Palette", |ui| {
                    if ui.button("Black").clicked()
                    {
                        let black_palette = ColorPalette::black(32.);
                        self.set_palette(black_palette);
                    }
                    else if ui.button("White").clicked()
                    {
                        let white_palette = ColorPalette::white(32.);
                        self.set_palette(white_palette);
                    }
                    else if ui.button("Random").clicked()
                    {
                        self.randomize_palette();
                    }
                    else
                    {
                        return;
                    }
                    ui.close_menu();
                    ui.pointer().take_click();
                });
                ui.menu_button("Algorithm", |ui| {
                    if ui.button("Solid").clicked()
                    {
                        self.set_coloring_algorithm(ColoringAlgorithm::Solid);
                    }
                    else if ui.button("Period").clicked()
                    {
                        self.set_coloring_algorithm(ColoringAlgorithm::Period);
                    }
                    else if ui.button("Period and Multiplier").clicked()
                    {
                        self.set_coloring_algorithm(ColoringAlgorithm::PeriodMultiplier);
                    }
                    else if ui.button("Multiplier").clicked()
                    {
                        self.set_coloring_algorithm(ColoringAlgorithm::Multiplier);
                    }
                    else if ui.button("Preperiod").clicked()
                    {
                        self.set_coloring_algorithm(ColoringAlgorithm::Preperiod);
                    }
                    else if ui.button("Preperiod (smooth)").clicked()
                    {
                        let periodicity_tolerance = self.parent.plane.periodicity_tolerance();
                        self.set_coloring_algorithm(ColoringAlgorithm::PreperiodSmooth {
                            periodicity_tolerance,
                        });
                    }
                    else
                    {
                        return;
                    }
                    ui.close_menu();
                    ui.pointer().take_click();
                });
            });
        });
    }
}

impl Default for FractalApp
{
    fn default() -> Self
    {
        let width = 1024;
        // let parameter_plane = QuadRatPer2::new_default(width, 2048).marked_cycle_curve(5);
        // let parameter_plane = QuadRatPer2::new_default(width, 2048);
        let parameter_plane = Mandelbrot::new_default(width, 2048).marked_cycle_curve(4);
        // let biquadratic_param = ComplexNum::new(0., -0.5);
        // let parameter_plane = Biquadratic::new_default(width, 2048, biquadratic_param);
        // let parameter_plane = CubicPer2CritMarked::new_default(width, 2048);
        // let parameter_plane = CubicPer1_1::new_default(width, 2048);

        let dynamical_plane = JuliaSet::new(parameter_plane, ((2_f64).powf(0.5)).into());

        let coloring = Coloring::default();

        let parent = Parent::new(Box::new(parameter_plane), coloring);
        let child = Child::new(Box::new(dynamical_plane), coloring);

        Self {
            parent,
            child,
            live_mode: false,
            active_pane: PaneID::Parent,
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
            self.show_menu(ui);

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
