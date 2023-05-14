use crate::coloring::{coloring_algorithm::ColoringAlgorithm, palette::ColorPalette};
use crate::dynamics::{covering_maps::HasDynamicalCovers, julia::JuliaSet, ParameterPlane};
use crate::profiles::*;
use crate::types::Period;

type DefaultProfile = QuadRatPer2;
// type DefaultProfile = CubicMarked2Cycle;
// type DefaultProfile = Rulkov;

use eframe::egui;
use egui::Key;
use egui_extras::{Column, TableBuilder};
use input_macro::input;

pub mod image_frame;
pub mod marked_points;
pub mod pane;
use image_frame::ImageFrame;
use pane::*;

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
    pane_pair: Box<dyn PanePair>,
    click_used: bool,
}

impl FractalApp
{
    pub fn parent(&self) -> &dyn Pane
    {
        self.pane_pair.parent()
    }
    pub fn parent_mut(&mut self) -> &mut dyn Pane
    {
        self.pane_pair.parent_mut()
    }
    pub fn child(&self) -> &dyn Pane
    {
        self.pane_pair.child()
    }
    pub fn child_mut(&mut self) -> &mut dyn Pane
    {
        self.pane_pair.child_mut()
    }
    pub fn randomize_palette(&mut self)
    {
        let palette = ColorPalette::new_random(0.45, 0.38);
        self.parent_mut().change_palette(palette);
        self.child_mut().change_palette(palette);
    }

    pub fn set_palette(&mut self, palette: ColorPalette)
    {
        self.parent_mut().change_palette(palette);
        self.child_mut().change_palette(palette);
    }

    pub fn set_coloring_algorithm(&mut self, coloring_algorithm: ColoringAlgorithm)
    {
        self.parent_mut().set_coloring_algorithm(coloring_algorithm);
        self.child_mut().set_coloring_algorithm(coloring_algorithm);
    }

    fn process_tasks(&mut self)
    {
        self.parent_mut().process_task();
        self.child_mut().process_task();
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

        if ctx.input(|i| i.key_pressed(Key::P))
        {
            self.child_mut().marking_mode_mut().toggle_critical();
            self.child_mut().schedule_redraw();
        }

        if ctx.input(|i| i.key_pressed(Key::Y))
        {
            self.child_mut().marking_mode_mut().toggle_cycles(1);
            self.child_mut().schedule_redraw();
        }

        if ctx.input(|i| i.key_pressed(Key::U))
        {
            self.child_mut().marking_mode_mut().toggle_cycles(2);
            self.child_mut().schedule_redraw();
        }

        if ctx.input(|i| i.key_pressed(Key::ArrowUp))
        {
            let pane = self.pane_pair.get_active_pane_mut();
            pane.scale_palette(1.25);
        }

        if ctx.input(|i| i.key_pressed(Key::ArrowDown))
        {
            let pane = self.pane_pair.get_active_pane_mut();
            pane.scale_palette(0.8);
        }

        if ctx.input(|i| i.key_pressed(Key::ArrowLeft))
        {
            let pane = self.pane_pair.get_active_pane_mut();
            pane.shift_palette(-0.02);
        }

        if ctx.input(|i| i.key_pressed(Key::ArrowRight))
        {
            let pane = self.pane_pair.get_active_pane_mut();
            pane.shift_palette(0.02);
        }

        if ctx.input(|i| i.key_pressed(Key::Z))
        {
            let pane = self.pane_pair.get_active_pane_mut();
            pane.zoom(0.8, pane.get_selection());
        }

        if ctx.input(|i| i.key_pressed(Key::V))
        {
            let pane = self.pane_pair.get_active_pane_mut();
            pane.zoom(1.25, pane.get_selection());
        }

        if ctx.input(|i| i.key_pressed(Key::PlusEquals))
        {
            let pane = self.pane_pair.get_active_pane_mut();
            let selection = pane.get_selection();
            pane.grid_mut().recenter(selection);
            pane.schedule_recompute();
        }

        if ctx.input(|i| i.key_pressed(Key::Num0))
        {
            let pane = self.pane_pair.get_active_pane_mut();
            pane.set_coloring_algorithm(ColoringAlgorithm::Solid);
        }

        if ctx.input(|i| i.key_pressed(Key::Num1))
        {
            let pane = self.pane_pair.get_active_pane_mut();
            pane.set_coloring_algorithm(ColoringAlgorithm::Period);
        }

        if ctx.input(|i| i.key_pressed(Key::Num2))
        {
            let pane = self.pane_pair.get_active_pane_mut();
            pane.set_coloring_algorithm(ColoringAlgorithm::PeriodMultiplier);
        }

        if ctx.input(|i| i.key_pressed(Key::Num3))
        {
            let pane = self.pane_pair.get_active_pane_mut();
            pane.set_coloring_algorithm(ColoringAlgorithm::Multiplier);
        }

        if ctx.input(|i| i.key_pressed(Key::Num4))
        {
            let pane = self.pane_pair.get_active_pane_mut();
            pane.set_coloring_algorithm(ColoringAlgorithm::Preperiod);
        }

        if ctx.input(|i| i.key_pressed(Key::Num5))
        {
            let pane = self.pane_pair.get_active_pane_mut();
            pane.select_preperiod_smooth_coloring();
        }

        if ctx.input(|i| i.key_pressed(Key::C))
        {
            let pane = self.pane_pair.get_active_pane_mut();
            pane.clear_marked_curves();
            pane.clear_marked_points();
        }

        if ctx.input(|i| i.key_pressed(Key::L))
        {
            self.pane_pair.toggle_live_mode();
        }

        if ctx.input(|i| i.key_pressed(Key::S))
        {
            let pane = self.pane_pair.get_active_pane_mut();
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

        if ctx.input(|i| i.key_pressed(Key::N))
        {
            let pane = self.pane_pair.get_active_pane_mut();
            pane.increase_max_iter();
        }

        if ctx.input(|i| i.key_pressed(Key::M))
        {
            let pane = self.pane_pair.get_active_pane_mut();
            pane.decrease_max_iter();
        }
        self.pane_pair.handle_mouse(ctx);
    }

    fn consume_click(&mut self)
    {
        self.click_used = true;
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
                let pane = self.pane_pair.get_active_pane_mut();
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
                    self.parent_mut().select_preperiod_smooth_coloring();
                    self.child_mut().select_preperiod_smooth_coloring();
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
                self.change_fractal(CubicPer2CritMarked::new_default);
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

    fn change_fractal<T>(&mut self, create_plane: fn(usize, Period) -> T)
    where
        T: ParameterPlane + Clone + 'static,
    {
        let res_y = self.parent().grid().res_y;
        let max_iters = 2048;

        let parent_plane = create_plane(res_y, max_iters);
        let selected_point = parent_plane.default_selection();
        let child_plane = JuliaSet::new(
            parent_plane.clone(),
            parent_plane.param_map(selected_point),
            256,
        );

        self.pane_pair = Box::new(WindowPanePair::new(parent_plane, child_plane));

        self.parent_mut().select_point(selected_point);
        self.parent_mut().clear_marked_curves();
        self.child_mut().clear_marked_curves();

        self.parent_mut().schedule_recompute();
        self.child_mut().schedule_recompute();
    }
}

impl Default for FractalApp
{
    fn default() -> Self
    {
        let height = 1024;
        // let parameter_plane = QuadRatPer2::new_default(height, 2048).marked_cycle_curve(5);
        let parameter_plane = DefaultProfile::new_default(height, 2048);
        // let parameter_plane = BiquadraticMult::new_default(height, 2048, (0.5).into());

        let dynamical_plane = JuliaSet::new(parameter_plane.clone(), (2_f64.sqrt()).into(), 1024);
        // let dynamical_plane = JuliaSet::new(parameter_plane.clone(), (1.,0.).into(), 1024);

        let parent = parameter_plane;
        let child = dynamical_plane;

        let pane_pair = Box::new(WindowPanePair::new(parent, child));

        Self {
            pane_pair,
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
                        ui.heading(self.parent().name());
                    });
                    header.col(|ui| {
                        ui.heading(self.child().name());
                    });
                })
                .body(|mut body| {
                    body.row(
                        self.parent().get_image_frame().height() as f32,
                        |mut row| {
                            row.col(|ui| {
                                self.parent_mut().get_image_frame_mut().put(ui);
                                self.parent_mut().put_marked_curves(ui);
                                self.parent_mut().put_marked_points(ui);
                            });
                            row.col(|ui| {
                                self.child_mut().get_image_frame_mut().put(ui);
                                self.child_mut().put_marked_curves(ui);
                                self.child_mut().put_marked_points(ui);
                            });
                        },
                    );
                    body.row(80., |mut row| {
                        row.col(|ui| {
                            let orbit_desc = self
                                .parent()
                                .describe_marked_info();
                            ui.label(orbit_desc);
                        });
                        row.col(|ui| {
                            let orbit_desc = self
                                .child()
                                .describe_marked_info();
                            ui.label(orbit_desc);
                        });
                    });
                });
        });
        self.click_used = false;
    }
}
