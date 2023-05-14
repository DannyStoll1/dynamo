use crate::coloring::{coloring_algorithm::ColoringAlgorithm, palette::ColorPalette};
use crate::dynamics::{covering_maps::HasDynamicalCovers, julia::JuliaSet, ParameterPlane};
use crate::profiles::*;
use crate::types::Period;

type DefaultProfile = QuadRatPer2;
// type DefaultProfile = CubicMarked2Cycle;
// type DefaultProfile = Rulkov;

use eframe::egui;
use egui::Key;

pub mod image_frame;
pub mod marked_points;
pub mod pane;
pub mod keyboard_shortcuts;
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
                self.pane_pair.save_active_pane();
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
                    self.pane_pair.set_palette(black_palette);
                }
                else if ui.button("White").clicked()
                {
                    let white_palette = ColorPalette::white(32.);
                    self.pane_pair.set_palette(white_palette);
                }
                else if ui.button("Random").clicked()
                {
                    self.pane_pair.randomize_palette();
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
                    self.pane_pair.set_coloring_algorithm(ColoringAlgorithm::Solid);
                }
                else if ui.button("Period").clicked()
                {
                    self.pane_pair.set_coloring_algorithm(ColoringAlgorithm::Period);
                }
                else if ui.button("Period and Multiplier").clicked()
                {
                    self.pane_pair.set_coloring_algorithm(ColoringAlgorithm::PeriodMultiplier);
                }
                else if ui.button("Multiplier").clicked()
                {
                    self.pane_pair.set_coloring_algorithm(ColoringAlgorithm::Multiplier);
                }
                else if ui.button("Preperiod").clicked()
                {
                    self.pane_pair.set_coloring_algorithm(ColoringAlgorithm::Preperiod);
                }
                else if ui.button("Preperiod (smooth)").clicked()
                {
                    self.pane_pair.parent_mut().select_preperiod_smooth_coloring();
                    self.pane_pair.child_mut().select_preperiod_smooth_coloring();
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
        let res_y = self.pane_pair.parent().grid().res_y;
        let max_iters = 2048;

        let parent_plane = create_plane(res_y, max_iters);
        let selected_point = parent_plane.default_selection();
        let child_plane = JuliaSet::new(
            parent_plane.clone(),
            parent_plane.param_map(selected_point),
            256,
        );

        self.pane_pair = Box::new(WindowPanePair::new(parent_plane, child_plane));
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

        let dynamical_plane = JuliaSet::new(parameter_plane.clone(), (0.).into(), 1024);
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
            self.pane_pair.handle_input(ctx);
            self.pane_pair.process_tasks();
            self.pane_pair.show(ui);
        });
        self.click_used = false;
    }
}
