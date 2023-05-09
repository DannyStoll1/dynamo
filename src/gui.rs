use crate::coloring::{coloring_algorithm::ColoringAlgorithm, palette::ColorPalette};
use crate::dynamics::{covering_maps::HasDynamicalCovers, julia::JuliaSet, ParameterPlane};
use crate::profiles::*;
use crate::types::{ComplexNum, Period};

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

pub struct FractalApp
{
    parent: Parent,
    child: Child,
    live_mode: bool,
    active_pane: PaneID,
    click_used: bool,
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
        let old_bounds = &self.child.grid().bounds;
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
            let coloring_algorithm = pane.plane().preperiod_smooth_coloring();
            pane.set_coloring_algorithm(coloring_algorithm);
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
        let clicked = (!self.click_used) && ctx.input(|i| i.pointer.any_click());
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
                    self.consume_click();
                    self.parent.clear_marked_curves();
                    dbg!(pointer_value);
                    dbg!(self.parent.plane.periodicity_tolerance());
                    let pointer_param = self.parent.plane.param_map(pointer_value);
                    let (orbit, info) = self.parent.plane.get_orbit_and_info(pointer_param);
                    self.parent.mark_curve(orbit, Color32::GREEN);
                    self.parent.set_marked_info(info);
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
                    self.consume_click();
                    // if let Some(ray) = self.child.plane.external_ray(pointer_value.arg(), 200, 25, 600) {
                    //     self.child.mark_curve(ray, Color32::GREEN);
                    // }
                    self.child.clear_marked_curves();
                    let pointer_param = self.child.plane.param_map(pointer_value);
                    let (orbit, info) = self.child.plane.get_orbit_and_info(pointer_param);
                    self.child.mark_curve(orbit, Color32::GREEN);
                    self.child.set_marked_info(info);
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

    fn consume_click(&mut self)
    {
        self.click_used = true;
    }

    fn change_fractal<T>(&mut self, create_parent_plane: fn(usize, Period) -> T)
    where
        T: ParameterPlane + Clone + 'static,
    {
        let parent_plane_ref = self.parent.plane_mut();
        let child_plane_ref = self.child.plane_mut();

        let res_y = parent_plane_ref.point_grid().res_y;
        let max_iters = 2048;

        let parent_plane = create_parent_plane(res_y, max_iters);
        let selected_point = parent_plane.default_julia_param();
        let child_plane = JuliaSet::new(parent_plane.clone(), selected_point, 256);

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
            self.file_menu(ui);
            self.fractal_menu(ui);
            self.coloring_menu(ui);
        });
    }

    fn file_menu(&mut self, ui: &mut egui::Ui)
    {
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
                self.consume_click();
            }
        });
    }

    fn fractal_menu(&mut self, ui: &mut egui::Ui)
    {
        ui.menu_button("Fractal", |ui| {
            self.polynomials_menu(ui);
            self.quadratic_rational_maps_menu(ui);
        });
    }

    fn coloring_menu(&mut self, ui: &mut egui::Ui)
    {
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
                self.consume_click();
                ui.close_menu();
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
                    let parent_coloring_algorithm = self.parent.plane().preperiod_smooth_coloring();
                    self.parent
                        .set_coloring_algorithm(parent_coloring_algorithm);

                    let child_coloring_algorithm = self.child.plane().preperiod_smooth_coloring();
                    self.child.set_coloring_algorithm(child_coloring_algorithm);
                }
                else
                {
                    return;
                }
                self.consume_click();
                ui.close_menu();
            });
        });
    }

    fn polynomials_menu(&mut self, ui: &mut egui::Ui)
    {
        ui.menu_button("Polynomials", |ui| {
            ui.menu_button("Quadratic Family", |ui| {
                if ui.button("Base Curve").clicked()
                {
                    self.change_fractal(Mandelbrot::new_default);
                }
                ui.menu_button("Marked Cycle", |ui| {
                    if ui.button("Marked 1-cycle").clicked()
                    {
                        self.change_fractal(|res, iter| {
                            Mandelbrot::new_default(res, iter).marked_cycle_curve(1)
                        });
                    }
                    else if ui.button("Marked 3-cycle").clicked()
                    {
                        self.change_fractal(|res, iter| {
                            Mandelbrot::new_default(res, iter).marked_cycle_curve(3)
                        });
                    }
                    else if ui.button("Marked 4-cycle").clicked()
                    {
                        self.change_fractal(|res, iter| {
                            Mandelbrot::new_default(res, iter).marked_cycle_curve(4)
                        });
                    }
                    else
                    {
                        return;
                    }
                    self.consume_click();
                    ui.close_menu();
                });
                ui.menu_button("Marked Periodic Point", |ui| {
                    if ui.button("Period 1").clicked()
                    {
                        self.change_fractal(|res, iter| {
                            Mandelbrot::new_default(res, iter).marked_cycle_curve(1)
                        });
                    }
                    else if ui.button("Period 3").clicked()
                    {
                        self.change_fractal(|res, iter| {
                            Mandelbrot::new_default(res, iter).dynatomic_curve(3)
                        });
                    }
                    else
                    {
                        return;
                    }
                    self.consume_click();
                    ui.close_menu();
                });
                ui.menu_button("Marked Preperiodic Point", |ui| {
                    if ui.button("Preperiod 2, Period 1").clicked()
                    {
                        self.change_fractal(|res, iter| {
                            Mandelbrot::new_default(res, iter).misiurewicz_curve(2, 1)
                        });
                    }
                    if ui.button("Preperiod 2, Period 2").clicked()
                    {
                        self.change_fractal(|res, iter| {
                            Mandelbrot::new_default(res, iter).misiurewicz_curve(2, 2)
                        });
                    }
                    else
                    {
                        return;
                    }
                    self.consume_click();
                    ui.close_menu();
                });
            });
            if ui.button("Odd Cubics").clicked()
            {
                self.change_fractal(OddCubic::new_default);
            }
            else if ui.button("Cubic Per(2, 0)").clicked()
            {
                self.change_fractal(|res, iter| CubicPer2CritMarked::new_default(res, iter));
            }
            else if ui.button("Cubic Per(1, 1)").clicked()
            {
                self.change_fractal(CubicPer1_1::new_default);
            }
            else
            {
                return;
            }
            self.consume_click();
            ui.close_menu();
        });
    }
    fn quadratic_rational_maps_menu(&mut self, ui: &mut egui::Ui)
    {
        ui.menu_button("Quadratic Rational Maps", |ui| {
            ui.menu_button("QuadRat Per(2)", |ui| {
                if ui.button("Base Curve").clicked()
                {
                    self.change_fractal(QuadRatPer2::new_default);
                    self.consume_click();
                    ui.close_menu();
                }
                ui.menu_button("Marked Cycle", |ui| {
                    if ui.button("Marked 1-cycle").clicked()
                    {
                        self.change_fractal(|res, iter| {
                            QuadRatPer2::new_default(res, iter).marked_cycle_curve(1)
                        });
                    }
                    else if ui.button("Marked 4-cycle").clicked()
                    {
                        self.change_fractal(|res, iter| {
                            QuadRatPer2::new_default(res, iter).marked_cycle_curve(4)
                        });
                    }
                    else if ui.button("Marked 5-cycle").clicked()
                    {
                        self.change_fractal(|res, iter| {
                            QuadRatPer2::new_default(res, iter).marked_cycle_curve(5)
                        });
                    }
                    else
                    {
                        return;
                    }
                    self.consume_click();
                    ui.close_menu();
                });
                ui.menu_button("Marked Periodic Point", |ui| {
                    if ui.button("Period 1").clicked()
                    {
                        self.change_fractal(|res, iter| {
                            QuadRatPer2::new_default(res, iter).marked_cycle_curve(1)
                        });
                    }
                    else
                    {
                        return;
                    }
                    self.consume_click();
                    ui.close_menu();
                });
                ui.menu_button("Marked Preperiodic Point", |ui| {
                    if ui.button("Preperiod 2, Period 1").clicked()
                    {
                        self.change_fractal(|res, iter| {
                            QuadRatPer2::new_default(res, iter).misiurewicz_curve(2, 1)
                        });
                    }
                    if ui.button("Preperiod 2, Period 2").clicked()
                    {
                        self.change_fractal(|res, iter| {
                            QuadRatPer2::new_default(res, iter).misiurewicz_curve(2, 2)
                        });
                    }
                    else
                    {
                        return;
                    }
                    self.consume_click();
                    ui.close_menu();
                });
            });
            ui.menu_button("QuadRat Per(3)", |ui| {
                if ui.button("Base Curve").clicked()
                {
                    self.change_fractal(QuadRatPer3::new_default);
                    self.consume_click();
                    ui.close_menu();
                }
                ui.menu_button("Marked Cycle curves", |ui| {
                    if ui.button("Marked 1-cycle").clicked()
                    {
                        self.change_fractal(|res, iter| {
                            QuadRatPer3::new_default(res, iter).marked_cycle_curve(1)
                        });
                    }
                    else if ui.button("Marked 4-cycle").clicked()
                    {
                        self.change_fractal(|res, iter| {
                            QuadRatPer3::new_default(res, iter).marked_cycle_curve(4)
                        });
                    }
                    else
                    {
                        return;
                    }
                    self.consume_click();
                    ui.close_menu();
                });
            });
            ui.menu_button("QuadRat Per(4)", |ui| {
                if ui.button("Base Curve").clicked()
                {
                    self.change_fractal(QuadRatPer4::new_default);
                    self.consume_click();
                    ui.close_menu();
                }
                ui.menu_button("Marked Cycle curves", |ui| {
                    if ui.button("Marked 3-cycle").clicked()
                    {
                        self.change_fractal(|res, iter| {
                            QuadRatPer4::new_default(res, iter).marked_cycle_curve(3)
                        });
                    }
                    else
                    {
                        return;
                    }
                    self.consume_click();
                    ui.close_menu();
                });
            });
            ui.menu_button("QuadRat Preper(2, 1)", |ui| {
                if ui.button("Base Curve").clicked()
                {
                    self.change_fractal(QuadRatPreper21::new_default);
                    self.consume_click();
                    ui.close_menu();
                }
                ui.menu_button("Marked Cycle", |ui| {
                    if ui.button("Marked 3-cycle").clicked()
                    {
                        self.change_fractal(|res, iter| {
                            QuadRatPreper21::new_default(res, iter).marked_cycle_curve(3)
                        });
                    }
                    else if ui.button("Marked 4-cycle").clicked()
                    {
                        self.change_fractal(|res, iter| {
                            QuadRatPreper21::new_default(res, iter).marked_cycle_curve(4)
                        });
                    }
                    else
                    {
                        return;
                    }
                    self.consume_click();
                    ui.close_menu();
                });
            });
        });
    }
}

impl Default for FractalApp
{
    fn default() -> Self
    {
        let height = 1024;
        // let parameter_plane = QuadRatPer2::new_default(height, 2048).marked_cycle_curve(5);
        // let parameter_plane = QuadRatPer2::new_default(height, 2048);
        // let parameter_plane = Mandelbrot::new_default(height, 2048);
        // let parameter_plane = QuadRatPreper21::new_default(height, 2048);
        // let parameter_plane = QuadRatPer4::new_default(height, 2048).marked_cycle_curve(3);
        let parameter_plane = QuadRatSymmetryLocus::new_default(height, 2048);
        // let parameter_plane = Mandelbrot::new_default(height, 2048).marked_cycle_curve(4);
        // let biquadratic_param = ComplexNum::new(0., -0.5);
        // let parameter_plane = Biquadratic::new_default(height, 2048, biquadratic_param);
        // let parameter_plane = CubicPer2CritMarked::new_default(height, 2048);
        // let parameter_plane = CubicPer1_1::new_default(height, 2048);

        let dynamical_plane =
            JuliaSet::new(parameter_plane.clone(), ((2_f64).powf(0.5)).into(), 1024);

        let parent = Parent::from(parameter_plane);
        let child = Child::from(dynamical_plane);

        Self {
            parent,
            child,
            live_mode: false,
            active_pane: PaneID::Parent,
            click_used: false,
        }
    }
}

impl eframe::App for FractalApp
{
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame)
    {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.show_menu(ui);
            self.handle_input(ctx);
            self.process_tasks();

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
                    body.row(80., |mut row| {
                        row.col(|ui| {
                            let orbit_desc = if let Some(orbit_info) = self.parent.get_marked_info()
                            {
                                orbit_info.summarize()
                            }
                            else
                            {
                                "".to_owned()
                            };
                            ui.label(orbit_desc);
                        });
                        row.col(|ui| {
                            let orbit_desc = if let Some(orbit_info) = self.child.get_marked_info()
                            {
                                orbit_info.summarize()
                            }
                            else
                            {
                                "".to_owned()
                            };
                            ui.label(orbit_desc);
                        });
                    });
                });
        });
        self.click_used = false;
    }
}
