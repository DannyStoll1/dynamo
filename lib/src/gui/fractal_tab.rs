use crate::coloring::{algorithms::*, palette::*};
use crate::dynamics::covering_maps::*;
use crate::dynamics::julia::JuliaSet;
use crate::dynamics::ParameterPlane;
use crate::gui::pane::*;
use crate::profiles::*;
use crate::types::*;
use egui_dock::NodeIndex;

pub struct FractalTab
{
    pub interface: Box<dyn Interface>,
    pub node: NodeIndex,
}

// {{{impl FractalTab
impl FractalTab
{
    pub fn with_node_index(mut self, node: NodeIndex) -> Self
    {
        self.node = node;
        self
    }

    pub fn show_menu(&mut self, ui: &mut egui::Ui)
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
            if ui.button("Save Parent").clicked()
            {
                self.interface.save_pane(PaneID::Parent);
                self.interface.consume_click();
                ui.close_menu();
            }
            else if ui.button("Save Child").clicked()
            {
                self.interface.save_pane(PaneID::Child);
                self.interface.consume_click();
                ui.close_menu();
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
                    self.interface.set_palette(black_palette);
                }
                else if ui.button("White").clicked()
                {
                    let white_palette = ColorPalette::white(32.);
                    self.interface.set_palette(white_palette);
                }
                else if ui.button("Random").clicked()
                {
                    self.interface.randomize_palette();
                }
                else
                {
                    return;
                }
                self.interface.consume_click();
                ui.close_menu();
            });
            ui.menu_button("Algorithm", |ui| {
                if ui.button("Solid").clicked()
                {
                    self.interface
                        .set_coloring_algorithm(ColoringAlgorithm::Solid);
                }
                else if ui.button("Period").clicked()
                {
                    self.interface
                        .set_coloring_algorithm(ColoringAlgorithm::Period);
                }
                else if ui.button("Period and Multiplier").clicked()
                {
                    self.interface
                        .set_coloring_algorithm(ColoringAlgorithm::PeriodMultiplier);
                }
                else if ui.button("Multiplier").clicked()
                {
                    self.interface
                        .set_coloring_algorithm(ColoringAlgorithm::Multiplier);
                }
                else if ui.button("Preperiod").clicked()
                {
                    self.interface
                        .set_coloring_algorithm(ColoringAlgorithm::Preperiod);
                }
                else if ui.button("Internal potential").clicked()
                {
                    self.interface
                        .parent_mut()
                        .select_preperiod_smooth_coloring();
                    self.interface
                        .child_mut()
                        .select_preperiod_smooth_coloring();
                }
                else if ui.button("Preperiod and Period").clicked()
                {
                    self.interface
                        .set_coloring_algorithm(ColoringAlgorithm::PreperiodPeriod);
                }
                else if ui.button("Internal potential and Period").clicked()
                {
                    self.interface
                        .parent_mut()
                        .select_preperiod_period_smooth_coloring();
                    self.interface
                        .child_mut()
                        .select_preperiod_period_smooth_coloring();
                }
                else
                {
                    return;
                }
                self.interface.consume_click();
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
                    self.change_fractal(Mandelbrot::default, JuliaSet::from);
                }
                ui.menu_button("Marked Cycle", |ui| {
                    if ui.button("Marked 1-cycle").clicked()
                    {
                        self.change_fractal(
                            || Mandelbrot::default().marked_cycle_curve(1),
                            JuliaSet::from,
                        );
                    }
                    else if ui.button("Marked 3-cycle").clicked()
                    {
                        self.change_fractal(
                            || Mandelbrot::default().marked_cycle_curve(3),
                            JuliaSet::from,
                        );
                    }
                    else if ui.button("Marked 4-cycle").clicked()
                    {
                        self.change_fractal(
                            || Mandelbrot::default().marked_cycle_curve(4),
                            JuliaSet::from,
                        );
                    }
                    else
                    {
                        return;
                    }
                    self.interface.consume_click();
                    ui.close_menu();
                });
                ui.menu_button("Marked Periodic Point", |ui| {
                    if ui.button("Period 1").clicked()
                    {
                        self.change_fractal(
                            || Mandelbrot::default().marked_cycle_curve(1),
                            JuliaSet::from,
                        );
                    }
                    else if ui.button("Period 3").clicked()
                    {
                        self.change_fractal(
                            || Mandelbrot::default().dynatomic_curve(3),
                            JuliaSet::from,
                        );
                    }
                    else
                    {
                        return;
                    }
                    self.interface.consume_click();
                    ui.close_menu();
                });
                ui.menu_button("Marked Preperiodic Point", |ui| {
                    if ui.button("Preperiod 2, Period 1").clicked()
                    {
                        self.change_fractal(
                            || Mandelbrot::default().misiurewicz_curve(2, 1),
                            JuliaSet::from,
                        );
                    }
                    if ui.button("Preperiod 2, Period 2").clicked()
                    {
                        self.change_fractal(
                            || Mandelbrot::default().misiurewicz_curve(2, 2),
                            JuliaSet::from,
                        );
                    }
                    else
                    {
                        return;
                    }
                    self.interface.consume_click();
                    ui.close_menu();
                });
            });
            if ui.button("Odd Cubics").clicked()
            {
                self.change_fractal(OddCubic::default, JuliaSet::from);
            }
            else if ui.button("Cubic Per(1, 1)").clicked()
            {
                self.change_fractal(CubicPer1_1::default, JuliaSet::from);
            }
            else if ui.button("Cubic Per(2, 0)").clicked()
            {
                self.change_fractal(CubicPer2CritMarked::default, JuliaSet::from);
            }
            else if ui.button("Cubic Per(3, 0)").clicked()
            {
                self.change_fractal(CubicPer3_0::default, JuliaSet::from);
            }
            else if ui.button("Cubic Per(1, lambda)").clicked()
            {
                self.change_fractal(CubicPer1LambdaParam::default, CubicPer1Lambda::from);
            }
            else if ui.button("Biquadratic").clicked()
            {
                self.change_fractal(BiquadraticMultParam::default, BiquadraticMult::from);
            }
            else
            {
                return;
            }
            self.interface.consume_click();
            ui.close_menu();
        });
    }
    fn quadratic_rational_maps_menu(&mut self, ui: &mut egui::Ui)
    {
        ui.menu_button("Quadratic Rational Maps", |ui| {
            ui.menu_button("QuadRat Per(2)", |ui| {
                if ui.button("Base Curve").clicked()
                {
                    self.change_fractal(QuadRatPer2::default, JuliaSet::from);
                    self.interface.consume_click();
                    ui.close_menu();
                }
                ui.menu_button("Marked Cycle", |ui| {
                    if ui.button("Marked 1-cycle").clicked()
                    {
                        self.change_fractal(
                            || QuadRatPer2::default().marked_cycle_curve(1),
                            JuliaSet::from,
                        );
                    }
                    else if ui.button("Marked 4-cycle").clicked()
                    {
                        self.change_fractal(
                            || QuadRatPer2::default().marked_cycle_curve(4),
                            JuliaSet::from,
                        );
                    }
                    else if ui.button("Marked 5-cycle").clicked()
                    {
                        self.change_fractal(
                            || QuadRatPer2::default().marked_cycle_curve(5),
                            JuliaSet::from,
                        );
                    }
                    else
                    {
                        return;
                    }
                    self.interface.consume_click();
                    ui.close_menu();
                });
                ui.menu_button("Marked Periodic Point", |ui| {
                    if ui.button("Period 1").clicked()
                    {
                        self.change_fractal(
                            || QuadRatPer2::default().marked_cycle_curve(1),
                            JuliaSet::from,
                        );
                    }
                    else
                    {
                        return;
                    }
                    self.interface.consume_click();
                    ui.close_menu();
                });
                ui.menu_button("Marked Preperiodic Point", |ui| {
                    if ui.button("Preperiod 2, Period 1").clicked()
                    {
                        self.change_fractal(
                            || QuadRatPer2::default().misiurewicz_curve(2, 1),
                            JuliaSet::from,
                        );
                    }
                    if ui.button("Preperiod 2, Period 2").clicked()
                    {
                        self.change_fractal(
                            || QuadRatPer2::default().misiurewicz_curve(2, 2),
                            JuliaSet::from,
                        );
                    }
                    else
                    {
                        return;
                    }
                    self.interface.consume_click();
                    ui.close_menu();
                });
            });
            ui.menu_button("QuadRat Per(3)", |ui| {
                if ui.button("Base Curve").clicked()
                {
                    self.change_fractal(QuadRatPer3::default, JuliaSet::from);
                    self.interface.consume_click();
                    ui.close_menu();
                }
                ui.menu_button("Marked Cycle curves", |ui| {
                    if ui.button("Marked 1-cycle").clicked()
                    {
                        self.change_fractal(
                            || QuadRatPer3::default().marked_cycle_curve(1),
                            JuliaSet::from,
                        );
                    }
                    else if ui.button("Marked 4-cycle").clicked()
                    {
                        self.change_fractal(
                            || QuadRatPer3::default().marked_cycle_curve(4),
                            JuliaSet::from,
                        );
                    }
                    else
                    {
                        return;
                    }
                    self.interface.consume_click();
                    ui.close_menu();
                });
            });
            ui.menu_button("QuadRat Per(4)", |ui| {
                if ui.button("Base Curve").clicked()
                {
                    self.change_fractal(QuadRatPer4::default, JuliaSet::from);
                    self.interface.consume_click();
                    ui.close_menu();
                }
                ui.menu_button("Marked Cycle curves", |ui| {
                    if ui.button("Marked 3-cycle").clicked()
                    {
                        self.change_fractal(
                            || QuadRatPer4::default().marked_cycle_curve(3),
                            JuliaSet::from,
                        );
                    }
                    else
                    {
                        return;
                    }
                    self.interface.consume_click();
                    ui.close_menu();
                });
            });
            if ui.button("QuadRat Per(5)").clicked()
            {
                self.change_fractal(QuadRatPer5::default, JuliaSet::from);
                self.interface.consume_click();
                ui.close_menu();
            }
            ui.menu_button("QuadRat Preper(2, 1)", |ui| {
                if ui.button("Base Curve").clicked()
                {
                    self.change_fractal(QuadRatPreper21::default, JuliaSet::from);
                    self.interface.consume_click();
                    ui.close_menu();
                }
                ui.menu_button("Marked Cycle", |ui| {
                    if ui.button("Marked 3-cycle").clicked()
                    {
                        self.change_fractal(
                            || QuadRatPreper21::default().marked_cycle_curve(3),
                            JuliaSet::from,
                        );
                    }
                    else if ui.button("Marked 4-cycle").clicked()
                    {
                        self.change_fractal(
                            || QuadRatPreper21::default().marked_cycle_curve(4),
                            JuliaSet::from,
                        );
                    }
                    else
                    {
                        return;
                    }
                    self.interface.consume_click();
                    ui.close_menu();
                });
            });
        });
    }

    fn change_fractal<P, J, C, M, T>(&mut self, create_plane: fn() -> P, create_child: fn(P) -> J)
    where
        P: ParameterPlane + Clone + 'static,
        J: ParameterPlane<Param = P::Param, MetaParam = M, Child = C> + Clone + 'static,
        C: ParameterPlane<Param = P::Param> + From<J>,
        M: ParamList<Param = T>,
        T: From<P::Param> + std::fmt::Display,
    {
        let image_height = self.interface.get_image_height();
        let max_iters = 2048;

        let parent_plane = create_plane()
            .with_max_iter(max_iters)
            .with_res_y(image_height);
        let child_plane = create_child(parent_plane.clone());

        self.interface = Box::new(MainInterface::new(parent_plane, child_plane, image_height));
    }
}
// }}}

impl Default for FractalTab
{
    fn default() -> Self
    {
        let height = 768;

        type Profile = Mandelbrot;
        let parent_plane = Profile::default().with_res_y(height).with_max_iter(1024);
        let child_plane = <Profile as ParameterPlane>::Child::from(parent_plane.clone());

        let interface = Box::new(MainInterface::new(parent_plane, child_plane, height));

        Self {
            interface,
            node: NodeIndex(0),
        }
    }
}

impl eframe::App for FractalTab
{
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame)
    {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.show_menu(ui);
            self.interface.handle_input(ctx);
            self.interface.show_save_dialog(ctx);
            self.interface.process_tasks();
            self.interface.show(ui);
        });
    }
}
