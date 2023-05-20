use crate::coloring::{algorithms::ColoringAlgorithm, palette::ColorPalette};
use crate::dynamics::{covering_maps::HasDynamicalCovers, julia::JuliaSet, ParameterPlane};
use crate::profiles::*;
use crate::types::Period;

type DefaultProfile = QuadRatPer5;
// type DefaultProfile = CubicMarked2Cycle;
// type DefaultProfile = Rulkov;

use eframe::egui;

pub mod image_frame;
pub mod keyboard_shortcuts;
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
                    self.pane_pair
                        .set_coloring_algorithm(ColoringAlgorithm::Solid);
                }
                else if ui.button("Period").clicked()
                {
                    self.pane_pair
                        .set_coloring_algorithm(ColoringAlgorithm::Period);
                }
                else if ui.button("Period and Multiplier").clicked()
                {
                    self.pane_pair
                        .set_coloring_algorithm(ColoringAlgorithm::PeriodMultiplier);
                }
                else if ui.button("Multiplier").clicked()
                {
                    self.pane_pair
                        .set_coloring_algorithm(ColoringAlgorithm::Multiplier);
                }
                else if ui.button("Preperiod").clicked()
                {
                    self.pane_pair
                        .set_coloring_algorithm(ColoringAlgorithm::Preperiod);
                }
                else if ui.button("Internal potential").clicked()
                {
                    self.pane_pair
                        .parent_mut()
                        .select_preperiod_smooth_coloring();
                    self.pane_pair
                        .child_mut()
                        .select_preperiod_smooth_coloring();
                }
                else if ui.button("Preperiod and Period").clicked()
                {
                    self.pane_pair
                        .set_coloring_algorithm(ColoringAlgorithm::PreperiodPeriod);
                }
                else if ui.button("Internal potential and Period").clicked()
                {
                    self.pane_pair
                        .parent_mut()
                        .select_preperiod_period_smooth_coloring();
                    self.pane_pair
                        .child_mut()
                        .select_preperiod_period_smooth_coloring();
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
                    self.change_fractal(Mandelbrot::new_default, JuliaSet::from);
                }
                ui.menu_button("Marked Cycle", |ui| {
                    if ui.button("Marked 1-cycle").clicked()
                    {
                        self.change_fractal(
                            |res, iter| Mandelbrot::new_default(res, iter).marked_cycle_curve(1),
                            JuliaSet::from,
                        );
                    }
                    else if ui.button("Marked 3-cycle").clicked()
                    {
                        self.change_fractal(
                            |res, iter| Mandelbrot::new_default(res, iter).marked_cycle_curve(3),
                            JuliaSet::from,
                        );
                    }
                    else if ui.button("Marked 4-cycle").clicked()
                    {
                        self.change_fractal(
                            |res, iter| Mandelbrot::new_default(res, iter).marked_cycle_curve(4),
                            JuliaSet::from,
                        );
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
                        self.change_fractal(
                            |res, iter| Mandelbrot::new_default(res, iter).marked_cycle_curve(1),
                            JuliaSet::from,
                        );
                    }
                    else if ui.button("Period 3").clicked()
                    {
                        self.change_fractal(
                            |res, iter| Mandelbrot::new_default(res, iter).dynatomic_curve(3),
                            JuliaSet::from,
                        );
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
                        self.change_fractal(
                            |res, iter| Mandelbrot::new_default(res, iter).misiurewicz_curve(2, 1),
                            JuliaSet::from,
                        );
                    }
                    if ui.button("Preperiod 2, Period 2").clicked()
                    {
                        self.change_fractal(
                            |res, iter| Mandelbrot::new_default(res, iter).misiurewicz_curve(2, 2),
                            JuliaSet::from,
                        );
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
                self.change_fractal(OddCubic::new_default, JuliaSet::from);
            }
            else if ui.button("Cubic Per(2, 0)").clicked()
            {
                self.change_fractal(CubicPer2CritMarked::new_default, JuliaSet::from);
            }
            else if ui.button("Cubic Per(1, 1)").clicked()
            {
                self.change_fractal(CubicPer1_1::new_default, JuliaSet::from);
            }
            else if ui.button("Biquadratic").clicked()
            {
                self.change_fractal(BiquadraticMultParam::new_default, BiquadraticMult::from);
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
                    self.change_fractal(QuadRatPer2::new_default, JuliaSet::from);
                    self.consume_click();
                    ui.close_menu();
                }
                ui.menu_button("Marked Cycle", |ui| {
                    if ui.button("Marked 1-cycle").clicked()
                    {
                        self.change_fractal(
                            |res, iter| QuadRatPer2::new_default(res, iter).marked_cycle_curve(1),
                            JuliaSet::from,
                        );
                    }
                    else if ui.button("Marked 4-cycle").clicked()
                    {
                        self.change_fractal(
                            |res, iter| QuadRatPer2::new_default(res, iter).marked_cycle_curve(4),
                            JuliaSet::from,
                        );
                    }
                    else if ui.button("Marked 5-cycle").clicked()
                    {
                        self.change_fractal(
                            |res, iter| QuadRatPer2::new_default(res, iter).marked_cycle_curve(5),
                            JuliaSet::from,
                        );
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
                        self.change_fractal(
                            |res, iter| QuadRatPer2::new_default(res, iter).marked_cycle_curve(1),
                            JuliaSet::from,
                        );
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
                        self.change_fractal(
                            |res, iter| QuadRatPer2::new_default(res, iter).misiurewicz_curve(2, 1),
                            JuliaSet::from,
                        );
                    }
                    if ui.button("Preperiod 2, Period 2").clicked()
                    {
                        self.change_fractal(
                            |res, iter| QuadRatPer2::new_default(res, iter).misiurewicz_curve(2, 2),
                            JuliaSet::from,
                        );
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
                    self.change_fractal(QuadRatPer3::new_default, JuliaSet::from);
                    self.consume_click();
                    ui.close_menu();
                }
                ui.menu_button("Marked Cycle curves", |ui| {
                    if ui.button("Marked 1-cycle").clicked()
                    {
                        self.change_fractal(
                            |res, iter| QuadRatPer3::new_default(res, iter).marked_cycle_curve(1),
                            JuliaSet::from,
                        );
                    }
                    else if ui.button("Marked 4-cycle").clicked()
                    {
                        self.change_fractal(
                            |res, iter| QuadRatPer3::new_default(res, iter).marked_cycle_curve(4),
                            JuliaSet::from,
                        );
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
                    self.change_fractal(QuadRatPer4::new_default, JuliaSet::from);
                    self.consume_click();
                    ui.close_menu();
                }
                ui.menu_button("Marked Cycle curves", |ui| {
                    if ui.button("Marked 3-cycle").clicked()
                    {
                        self.change_fractal(
                            |res, iter| QuadRatPer4::new_default(res, iter).marked_cycle_curve(3),
                            JuliaSet::from,
                        );
                    }
                    else
                    {
                        return;
                    }
                    self.consume_click();
                    ui.close_menu();
                });
            });
            if ui.button("QuadRat Per(5)").clicked()
            {
                self.change_fractal(QuadRatPer5::new_default, JuliaSet::from);
                self.consume_click();
                ui.close_menu();
            }
            ui.menu_button("QuadRat Preper(2, 1)", |ui| {
                if ui.button("Base Curve").clicked()
                {
                    self.change_fractal(QuadRatPreper21::new_default, JuliaSet::from);
                    self.consume_click();
                    ui.close_menu();
                }
                ui.menu_button("Marked Cycle", |ui| {
                    if ui.button("Marked 3-cycle").clicked()
                    {
                        self.change_fractal(
                            |res, iter| {
                                QuadRatPreper21::new_default(res, iter).marked_cycle_curve(3)
                            },
                            JuliaSet::from,
                        );
                    }
                    else if ui.button("Marked 4-cycle").clicked()
                    {
                        self.change_fractal(
                            |res, iter| {
                                QuadRatPreper21::new_default(res, iter).marked_cycle_curve(4)
                            },
                            JuliaSet::from,
                        );
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

    fn change_fractal<T, S>(
        &mut self,
        create_plane: fn(usize, Period) -> T,
        create_child: fn(T) -> S,
    ) where
        T: ParameterPlane + Clone + 'static,
        S: ParameterPlane<Param = T::Param> + Clone + 'static,
    {
        let res_y = self.pane_pair.parent().grid().res_y;
        let max_iters = 2048;

        let parent_plane = create_plane(res_y, max_iters);
        let child_plane = create_child(parent_plane.clone());

        self.pane_pair = Box::new(WindowPanePair::new(parent_plane, child_plane));
    }
}

impl Default for FractalApp
{
    fn default() -> Self
    {
        let height = 768;
        // let parent_plane = QuadRatPer2::new_default(height, 2048).marked_cycle_curve(5);
        // let parent_plane = DefaultProfile::new_default(height, 2048);
        // let parent_plane = BiquadraticMult::new_default(height, 2048, (0.5).into());

        // let parent_plane = BiquadraticMultParam::new_default(height, 1024);
        // let child_plane = BiquadraticMult::from(parent_plane.clone());

        use crate::types::ComplexNum;
        let multiplier = ComplexNum::new(0.3, 0.0);
        let parent_plane = BiquadraticMult::new_default(height, 1024, multiplier);
        let child_plane = JuliaSet::from(parent_plane.clone());

        // let parent_plane = CubicPer1Lambda::new_default(height, 1024);
        // let child_plane = JuliaSet::from(parent_plane.clone());

        let pane_pair = Box::new(WindowPanePair::new(parent_plane, child_plane));

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
