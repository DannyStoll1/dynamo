use crate::coloring::{algorithms::*, palette::*};
use crate::dynamics::covering_maps::*;
use crate::dynamics::julia::JuliaSet;
use crate::dynamics::ParameterPlane;
use crate::gui::pane::*;
use crate::macros::{
    fractal_menu_button, fractal_menu_button_dyn, fractal_menu_button_mc, fractal_menu_button_mis,
};
use crate::profiles::*;
use crate::types::*;
use egui::Ui;
use egui_dock::NodeIndex;

pub struct FractalTab
{
    pub interface: Box<dyn Interface>,
    pub node: NodeIndex,
}

// {{{impl FractalTab
impl FractalTab
{
    pub const fn with_node_index(mut self, node: NodeIndex) -> Self
    {
        self.node = node;
        self
    }

    pub fn show_menu(&mut self, ui: &mut Ui)
    {
        egui::menu::bar(ui, |ui| {
            self.file_menu(ui);
            self.fractal_menu(ui);
            self.coloring_menu(ui);
            self.image_menu(ui);
            self.annotations_menu(ui);
        });
    }

    fn file_menu(&mut self, ui: &mut Ui)
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

    fn fractal_menu(&mut self, ui: &mut Ui)
    {
        ui.menu_button("Fractal", |ui| {
            self.polynomials_menu(ui);
            self.rational_maps_menu(ui);
            self.transcendental_menu(ui);
            self.non_analytic_menu(ui);
        });
    }

    fn coloring_menu(&mut self, ui: &mut Ui)
    {
        ui.menu_button("Coloring", |ui| {
            ui.menu_button("Palette", |ui| {
                if ui.button("[B]lack").clicked()
                {
                    let black_palette = ColorPalette::black(32.);
                    self.interface.set_palette(black_palette);
                }
                else if ui.button("[W]hite").clicked()
                {
                    let white_palette = ColorPalette::white(32.);
                    self.interface.set_palette(white_palette);
                }
                else if ui.button("[R]andom").clicked()
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
                if ui.button("[0] Solid").clicked()
                {
                    self.interface
                        .set_coloring_algorithm(ColoringAlgorithm::Solid);
                }
                else if ui.button("[1] Period").clicked()
                {
                    self.interface
                        .set_coloring_algorithm(ColoringAlgorithm::Period);
                }
                else if ui.button("[2] Period and Multiplier").clicked()
                {
                    self.interface
                        .set_coloring_algorithm(ColoringAlgorithm::PeriodMultiplier);
                }
                else if ui.button("[3] Multiplier").clicked()
                {
                    self.interface
                        .set_coloring_algorithm(ColoringAlgorithm::Multiplier);
                }
                else if ui.button("[4] Preperiod").clicked()
                {
                    self.interface
                        .set_coloring_algorithm(ColoringAlgorithm::Preperiod);
                }
                else if ui.button("[5] Internal potential").clicked()
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

    fn image_menu(&mut self, ui: &mut Ui)
    {
        ui.menu_button("Image", |ui| {
            if ui.button("Toggle [L]ive Julia").clicked()
            {
                self.interface.toggle_live_mode();
                self.interface.consume_click();
                ui.close_menu();
            }
            ui.menu_button("Set height", |ui| {
                if ui.button("256").clicked()
                {
                    self.interface.change_height(256);
                }
                else if ui.button("512").clicked()
                {
                    self.interface.change_height(512);
                }
                else if ui.button("768").clicked()
                {
                    self.interface.change_height(768);
                }
                else if ui.button("1024").clicked()
                {
                    self.interface.change_height(1024);
                }
                else if ui.button("1536").clicked()
                {
                    self.interface.change_height(1536);
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

    fn annotations_menu(&mut self, ui: &mut Ui)
    {
        ui.menu_button("Annotations", |ui| {
            if ui.button("[C]lear marked curves").clicked()
            {
                self.interface.child_mut().clear_marked_curves();
                self.interface.parent_mut().clear_marked_curves();
                self.interface.child_mut().schedule_redraw();
                self.interface.parent_mut().schedule_redraw();
                self.interface.consume_click();
                ui.close_menu();
            }
            else if ui.button("[I] Toggle selection").clicked()
            {
                self.interface
                    .child_mut()
                    .marking_mode_mut()
                    .toggle_selection();
                self.interface
                    .parent_mut()
                    .marking_mode_mut()
                    .toggle_selection();
                self.interface.child_mut().schedule_redraw();
                self.interface.parent_mut().schedule_redraw();
                self.interface.consume_click();
                ui.close_menu();
            }
            else if ui.button("[P] Toggle Critical points (Julia)").clicked()
            {
                self.interface
                    .child_mut()
                    .marking_mode_mut()
                    .toggle_critical();
                self.interface.child_mut().schedule_redraw();
                self.interface.consume_click();
                ui.close_menu();
            }
            else if ui.button("[O] Toggle marked points (Param)").clicked()
            {
                self.interface
                    .parent_mut()
                    .marking_mode_mut()
                    .toggle_critical();
                self.interface.parent_mut().schedule_redraw();
                self.interface.consume_click();
                ui.close_menu();
            }
            else if ui.button("[Y] Toggle fixed points (Julia)").clicked()
            {
                self.interface
                    .child_mut()
                    .marking_mode_mut()
                    .toggle_cycles(1);
                self.interface.child_mut().schedule_redraw();
                self.interface.consume_click();
                ui.close_menu();
            }
            else if ui.button("[U] Toggle 2-cycles (Julia)").clicked()
            {
                self.interface
                    .child_mut()
                    .marking_mode_mut()
                    .toggle_cycles(2);
                self.interface.child_mut().schedule_redraw();
                self.interface.consume_click();
                ui.close_menu();
            }
        });
    }

    fn polynomials_menu(&mut self, ui: &mut Ui)
    {
        ui.menu_button("Polynomials", |ui| {
            ui.menu_button("Quadratic Family", |ui| {
                fractal_menu_button!(self, ui, "Base Curve", Mandelbrot);
                ui.menu_button("Marked Cycle", |ui| {
                    fractal_menu_button_mc!(self, ui, Mandelbrot, 1);
                    fractal_menu_button_mc!(self, ui, Mandelbrot, 3);
                    fractal_menu_button_mc!(self, ui, Mandelbrot, 4);
                });
                ui.menu_button("Marked Periodic Point", |ui| {
                    fractal_menu_button_mc!(self, ui, Mandelbrot, 1);
                    fractal_menu_button_dyn!(self, ui, Mandelbrot, 3);
                });
                ui.menu_button("Marked Preperiodic Point", |ui| {
                    fractal_menu_button_mis!(self, ui, Mandelbrot, 2, 1);
                    fractal_menu_button_mis!(self, ui, Mandelbrot, 2, 2);
                });
            });
            ui.menu_button("Cubic Family", |ui| {
                fractal_menu_button!(self, ui, "Odd Cubics", OddCubic);
                fractal_menu_button!(self, ui, "Per(1)", CubicPer1_0);
                fractal_menu_button!(self, ui, "Per(2)", CubicPer2CritMarked);
                fractal_menu_button!(self, ui, "Per(3)", CubicPer3_0);
                fractal_menu_button!(self, ui, "Per(1, 1)", CubicPer1_1);
                fractal_menu_button!(self, ui, "Per(1, lambda)", CubicPer1LambdaParam);
            });
            fractal_menu_button!(self, ui, "Biquadratic Maps", BiquadraticMultParam);
        });
    }
    fn rational_maps_menu(&mut self, ui: &mut Ui)
    {
        ui.menu_button("Rational Maps", |ui| {
            ui.menu_button("QuadRat Per(2)", |ui| {
                fractal_menu_button!(self, ui, "Base Curve", QuadRatPer2);
                ui.menu_button("Marked Cycle", |ui| {
                    fractal_menu_button_mc!(self, ui, QuadRatPer2, 1);
                    fractal_menu_button_mc!(self, ui, QuadRatPer2, 4);
                    fractal_menu_button_mc!(self, ui, QuadRatPer2, 5);
                });
                ui.menu_button("Marked Periodic Point", |ui| {
                    fractal_menu_button_mc!(self, ui, QuadRatPer2, 1);
                });
                ui.menu_button("Marked Preperiodic Point", |ui| {
                    fractal_menu_button_mis!(self, ui, QuadRatPer2, 2, 1);
                    fractal_menu_button_mis!(self, ui, QuadRatPer2, 2, 2);
                });
            });
            ui.menu_button("QuadRat Per(3)", |ui| {
                fractal_menu_button!(self, ui, "Base Curve", QuadRatPer3);
                ui.menu_button("Marked Cycle curves", |ui| {
                    fractal_menu_button_mc!(self, ui, QuadRatPer3, 1);
                    fractal_menu_button_mc!(self, ui, QuadRatPer3, 4);
                });
            });
            ui.menu_button("QuadRat Per(4)", |ui| {
                fractal_menu_button!(self, ui, "Base Curve", QuadRatPer4);
                ui.menu_button("Marked Cycle curves", |ui| {
                    fractal_menu_button_mc!(self, ui, QuadRatPer4, 3);
                });
            });
            fractal_menu_button!(self, ui, "QuadRat Per(5)", QuadRatPer5);
            ui.menu_button("QuadRat Preper(2, 1)", |ui| {
                fractal_menu_button!(self, ui, "Base Curve", QuadRatPreper21);
                ui.menu_button("Marked Cycle", |ui| {
                    fractal_menu_button_mc!(self, ui, QuadRatPreper21, 3);
                    fractal_menu_button_mc!(self, ui, QuadRatPreper21, 4);
                });
            });
            fractal_menu_button!(self, ui, "QuadRat Symmetry Locus", QuadRatSymmetryLocus);
            fractal_menu_button!(self, ui, "McMullen Family", McMullenFamily);
            fractal_menu_button!(self, ui, "Minsik Han Φ", MinsikHanPhi);
        });
    }

    fn transcendental_menu(&mut self, ui: &mut Ui)
    {
        ui.menu_button("Transcendental maps", |ui| {
            fractal_menu_button!(self, ui, "z -> λexp(z)", Exponential);
            fractal_menu_button!(self, ui, "z -> λcos(z)", Cosine);
            fractal_menu_button!(self, ui, "z -> cos(z) + c", CosineAdd);
            fractal_menu_button!(self, ui, "z -> sin(z) + z + τc", SineWander);
            // fractal_menu_button!(self, ui, "Riemann Xi [SLOW!]", RiemannXi);
        });
    }

    fn non_analytic_menu(&mut self, ui: &mut Ui)
    {
        ui.menu_button("Non-analytic maps", |ui| {
            fractal_menu_button!(self, ui, "Tricorne", Tricorne);
            fractal_menu_button!(self, ui, "Burning Ship", BurningShip);
            fractal_menu_button!(self, ui, "Sailboat Param", SailboatParam);
            fractal_menu_button!(self, ui, "Rulkov Map", Rulkov);
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
