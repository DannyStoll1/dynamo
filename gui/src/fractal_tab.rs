use crate::macros::{
    fractal_menu_button, fractal_menu_button_dyn, fractal_menu_button_mc, fractal_menu_button_mis,
};
use crate::pane::{Interface, MainInterface, PaneID};
use egui::Ui;
use egui_dock::NodeIndex;
use fractal_common::coloring::{algorithms::ColoringAlgorithm, palette::ColorPalette};
use fractal_common::consts::{OMEGA, ONE};
use fractal_common::types::{Cplx, ParamList};
use fractal_core::dynamics::covering_maps::HasDynamicalCovers;
use fractal_core::dynamics::julia::JuliaSet;
use fractal_core::dynamics::ParameterPlane;
use fractal_profiles::*;
use seq_macro::seq;

pub struct FractalTab
{
    pub interface: Box<dyn Interface>,
    pub node: NodeIndex,
}

// {{{impl FractalTab
impl FractalTab
{
    #[must_use]
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
            else if ui.button("[P] Toggle critical points (Julia)").clicked()
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
            else if ui.button("[^1] Toggle fixed points (Julia)").clicked()
            {
                self.interface
                    .child_mut()
                    .marking_mode_mut()
                    .toggle_cycles(1);
                self.interface.child_mut().schedule_redraw();
                self.interface.consume_click();
                ui.close_menu();
            }
            else if ui.button("[^2] Toggle 2-cycles (Julia)").clicked()
            {
                self.interface
                    .child_mut()
                    .marking_mode_mut()
                    .toggle_cycles(2);
                self.interface.child_mut().schedule_redraw();
                self.interface.consume_click();
                ui.close_menu();
            }
            else if ui.button("[^3] Toggle 3-cycles (Julia)").clicked()
            {
                self.interface
                    .child_mut()
                    .marking_mode_mut()
                    .toggle_cycles(3);
                self.interface.child_mut().schedule_redraw();
                self.interface.consume_click();
                ui.close_menu();
            }
            else if ui.button("[^4] Toggle 4-cycles (Julia)").clicked()
            {
                self.interface
                    .child_mut()
                    .marking_mode_mut()
                    .toggle_cycles(4);
                self.interface.child_mut().schedule_redraw();
                self.interface.consume_click();
                ui.close_menu();
            }
        });
    }

    fn polynomials_menu(&mut self, ui: &mut Ui)
    {
        ui.menu_button("Polynomials", |ui| {
            ui.set_max_width(250.);
            ui.menu_button("Quadratic Family", |ui| {
                fractal_menu_button!(self, ui, "Base Curve", Mandelbrot);
                ui.menu_button("Marked Cycle", |ui| {
                    fractal_menu_button_mc!(self, ui, Mandelbrot, 1);
                    fractal_menu_button_mc!(self, ui, Mandelbrot, 3);
                    fractal_menu_button_mc!(self, ui, Mandelbrot, 4);
                });
                ui.menu_button("Marked Periodic Point", |ui| {
                    fractal_menu_button_mc!(self, ui, Mandelbrot, 1);
                    fractal_menu_button_dyn!(self, ui, Mandelbrot, 2);
                    fractal_menu_button_dyn!(self, ui, Mandelbrot, 3);
                });
                ui.menu_button("Marked Preperiodic Point", |ui| {
                    fractal_menu_button_mis!(self, ui, Mandelbrot, 2, 1);
                    fractal_menu_button_mis!(self, ui, Mandelbrot, 2, 2);
                });
            });
            ui.menu_button("Cubic Family", |ui| {
                ui.menu_button("Odd Cubics", |ui| {
                    fractal_menu_button!(self, ui, "Base curve", OddCubic);
                    ui.menu_button("Marked Cycle", |ui| {
                        fractal_menu_button_mc!(self, ui, OddCubic, 1);
                        fractal_menu_button_mc!(self, ui, OddCubic, 2);
                    });
                    ui.menu_button("Marked Periodic Point", |ui| {
                        fractal_menu_button_dyn!(self, ui, OddCubic, 1);
                        fractal_menu_button_dyn!(self, ui, OddCubic, 2);
                    });
                    ui.menu_button("Marked Preperiodic Point", |ui| {
                        fractal_menu_button_mis!(self, ui, OddCubic, 1, 1);
                        fractal_menu_button_mis!(self, ui, OddCubic, 1, 2);
                    });
                });
                ui.menu_button("Cubic Per(1)", |ui| {
                    fractal_menu_button!(self, ui, "Base Curve", CubicPer1_0);
                    ui.menu_button("Marked Cycle", |ui| {
                        fractal_menu_button_mc!(self, ui, CubicPer1_0, 1);
                        fractal_menu_button_mc!(self, ui, CubicPer1_0, 2);
                    });
                    ui.menu_button("Marked Periodic Point", |ui| {
                        fractal_menu_button_dyn!(self, ui, CubicPer1_0, 1);
                        fractal_menu_button_dyn!(self, ui, CubicPer1_0, 2);
                    });
                    ui.menu_button("Marked Preperiodic Point", |ui| {
                        fractal_menu_button_mis!(self, ui, CubicPer1_0, 1, 1);
                    });
                });
                ui.menu_button("Cubic Per(2)", |ui| {
                    fractal_menu_button!(self, ui, "Base curve", CubicPer2CritMarked);
                    ui.menu_button("Marked Cycle", |ui| {
                        fractal_menu_button_mc!(self, ui, CubicPer2CritMarked, 1);
                        fractal_menu_button_mc!(self, ui, CubicPer2CritMarked, 2);
                    });
                });
                fractal_menu_button!(self, ui, "Per(3)", CubicPer3_0);
                ui.menu_button("Cubic Per(1, 1)", |ui| {
                    fractal_menu_button!(self, ui, "Base Curve", CubicPer1_1);
                    ui.menu_button("Marked Cycle", |ui| {
                        fractal_menu_button_mc!(self, ui, CubicPer1_1, 2);
                    });
                    ui.menu_button("Marked Periodic Point", |ui| {
                        fractal_menu_button_dyn!(self, ui, CubicPer1_1, 2);
                    });
                    ui.menu_button("Marked Preperiodic Point", |ui| {
                        fractal_menu_button_mis!(self, ui, CubicPer1_1, 1, 1);
                    });
                });
                ui.menu_button("Cubic Per(1, λ)", |ui| {
                    fractal_menu_button!(self, ui, "λ-plane", CubicPer1LambdaParam);
                    fractal_menu_button!(
                        self,
                        ui,
                        "λ=0.3",
                        CubicPer1Lambda,
                        with_param,
                        Cplx::from(0.3)
                    );
                    fractal_menu_button!(
                        self,
                        ui,
                        "λ=0.3 moduli",
                        CubicPer1LambdaModuli,
                        with_param,
                        Cplx::from(0.3)
                    );
                    fractal_menu_button!(
                        self,
                        ui,
                        "λ=0.2+0.7i moduli",
                        CubicPer1LambdaModuli,
                        with_param,
                        Cplx::new(0.2, 0.7)
                    );
                    fractal_menu_button!(
                        self,
                        ui,
                        "λ=0.99 moduli",
                        CubicPer1LambdaModuli,
                        with_param,
                        Cplx::from(0.99)
                    );
                    fractal_menu_button!(
                        self,
                        ui,
                        "λ=0.99i",
                        CubicPer1Lambda,
                        with_param,
                        Cplx::new(0., 0.99)
                    );
                });
                ui.menu_button("Per(2, λ)", |ui| {
                    fractal_menu_button!(self, ui, "λ-plane", CubicPer2LambdaParam);
                    fractal_menu_button!(
                        self,
                        ui,
                        "λ=0.3",
                        CubicPer2Lambda,
                        with_param,
                        Cplx::from(0.3)
                    );
                    fractal_menu_button!(
                        self,
                        ui,
                        "λ=0.99i",
                        CubicPer2Lambda,
                        with_param,
                        Cplx::new(0., 0.99)
                    );
                });
                ui.menu_button("2-cycle 0 <-> 1", |ui| {
                    fractal_menu_button!(self, ui, "Base curve", CubicMarked2Cycle);
                    ui.menu_button("Marked Cycle", |ui| {
                        fractal_menu_button_mc!(self, ui, CubicMarked2Cycle, 1);
                    });
                    ui.menu_button("Marked Periodic Point", |ui| {
                        fractal_menu_button_dyn!(self, ui, CubicMarked2Cycle, 2);
                    });
                    ui.menu_button("Marked Preperiodic Point", |ui| {
                        fractal_menu_button_mis!(self, ui, CubicMarked2Cycle, 1, 1);
                        fractal_menu_button_mis!(self, ui, CubicMarked2Cycle, 1, 2);
                    });
                });
            });
            ui.menu_button("Unicritical Maps: z -> c*(1+z/d)^d", |ui| {
                ui.menu_button("Degree 3", |ui| {
                    fractal_menu_button!(self, ui, "Base curve", Unicritical<3>);
                    ui.menu_button("Marked Cycle", |ui| {
                        fractal_menu_button_mc!(self, ui, Unicritical<3>, 1);
                        fractal_menu_button_mc!(self, ui, Unicritical<3>, 2);
                        fractal_menu_button_mc!(self, ui, Unicritical<3>, 3);
                    });
                    ui.menu_button("Marked Periodic Point", |ui| {
                        fractal_menu_button_mc!(self, ui, Unicritical<3>, 1);
                        fractal_menu_button_dyn!(self, ui, Unicritical<3>, 2);
                    });
                });
                seq!(D in 4..=8 {
                    fractal_menu_button!(self, ui, format!("Degree {}", D), Unicritical<D>);
                });
            });
            #[allow(clippy::identity_op)]
            ui.menu_button("Chebyshev family: z -> (-1)^k * c * cheb2k(z/2)", |ui| {
                seq!(D in 1..=5 {
                    fractal_menu_button!(self, ui, format!("Degree {}", 2*D), Chebyshev<D>);
                });
            });
            ui.menu_button("Biquadratic Maps", |ui| {
                fractal_menu_button!(self, ui, "λ-plane", BiquadraticMultParam);
                fractal_menu_button!(
                    self,
                    ui,
                    "λ=0.3",
                    BiquadraticMult,
                    with_param,
                    Cplx::from(0.3)
                );
                fractal_menu_button!(
                    self,
                    ui,
                    "λ=0.2+0.7j",
                    BiquadraticMult,
                    with_param,
                    Cplx::new(0.2, 0.7)
                );
                fractal_menu_button!(
                    self,
                    ui,
                    "λ=0.99i",
                    BiquadraticMult,
                    with_param,
                    Cplx::new(0., 0.99)
                );
                fractal_menu_button!(self, ui, "Section (b=1): λ-plane", BiquadraticMultSection);
            });
        });
    }
    fn rational_maps_menu(&mut self, ui: &mut Ui)
    {
        ui.menu_button("Rational Maps", |ui| {
            ui.set_max_width(250.);
            ui.menu_button("QuadRat Per(2)", |ui| {
                fractal_menu_button!(self, ui, "Base Curve", QuadRatPer2);
                ui.menu_button("Marked Cycle", |ui| {
                    fractal_menu_button_mc!(self, ui, QuadRatPer2, 1);
                    fractal_menu_button_mc!(self, ui, QuadRatPer2, 4);
                    fractal_menu_button_mc!(self, ui, QuadRatPer2, 5);
                });
                ui.menu_button("Marked Periodic Point", |ui| {
                    fractal_menu_button_mc!(self, ui, QuadRatPer2, 1);
                    fractal_menu_button_dyn!(self, ui, QuadRatPer2, 3);
                    fractal_menu_button_dyn!(self, ui, QuadRatPer2, 4);
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
            fractal_menu_button!(self, ui, "QuadRat Preper(2, 2)", QuadRatPreper22);
            ui.menu_button("QuadRat Per(1, λ)", |ui| {
                fractal_menu_button!(self, ui, "λ-plane", QuadRatPer1LambdaParam);
                fractal_menu_button!(
                    self,
                    ui,
                    "λ=1",
                    QuadRatPer1Lambda,
                    with_param,
                    ONE
                );
                fractal_menu_button!(
                    self,
                    ui,
                    "λ=-1",
                    QuadRatPer1Lambda,
                    with_param,
                    -ONE
                );
                fractal_menu_button!(
                    self,
                    ui,
                    "λ=ω",
                    QuadRatPer1Lambda,
                    with_param,
                    OMEGA
                );
                fractal_menu_button!(
                    self,
                    ui,
                    "λ=i",
                    QuadRatPer1Lambda,
                    with_param,
                    Cplx::new(0., 1.)
                );
            });
            ui.menu_button("QuadRat Per(2, λ)", |ui| {
                fractal_menu_button!(self, ui, "λ-plane", QuadRatPer2LambdaParam);
                fractal_menu_button!(
                    self,
                    ui,
                    "λ=1",
                    QuadRatPer2Lambda,
                    with_param,
                    ONE
                );
                fractal_menu_button!(
                    self,
                    ui,
                    "λ=i",
                    QuadRatPer2Lambda,
                    with_param,
                    Cplx::new(0., 1.)
                );
                fractal_menu_button!(
                    self,
                    ui,
                    "λ=-3",
                    QuadRatPer2Lambda,
                    with_param,
                    Cplx::from(-3.)
                );
                fractal_menu_button!(
                    self,
                    ui,
                    "λ=-27",
                    QuadRatPer2Lambda,
                    with_param,
                    Cplx::from(-27.)
                );
            });

            fractal_menu_button!(self, ui, "QuadRat Symmetry Locus", QuadRatSymmetryLocus);
            fractal_menu_button!(self, ui, "Newton Cubic", NewtonCubic);
            ui.menu_button("McMullen Family: z -> z^m + 1/(c*z^n)", |ui| {
                seq!(N in 2..=8 {
                    fractal_menu_button!(self, ui, format!("(m=2, n={})", N), McMullenFamily<2, N>);
                });
                seq!(M in 2..=8 {
                    fractal_menu_button!(self, ui, format!("(m={}, n={})", M, M), McMullenFamily<M, M>);
                });
            });
            ui.menu_button("Minsik Han Φ: z -> az/(z^d+d-1)", |ui| {
                seq!(D in 2..=8 {
                    fractal_menu_button!(self, ui, format!("Degree {}", D), MinsikHanPhi<D>);
                });
            });
        });
    }

    fn transcendental_menu(&mut self, ui: &mut Ui)
    {
        ui.menu_button("Transcendental maps", |ui| {
            fractal_menu_button!(self, ui, "z -> λexp(z)", Exponential);
            fractal_menu_button!(self, ui, "z -> λcos(z)", Cosine);
            fractal_menu_button!(self, ui, "z -> cos(z) + c", CosineAdd);
            fractal_menu_button!(self, ui, "z -> sin(z) + z + τc", SineWander);
            fractal_menu_button!(self, ui, "Riemann Xi Newton [SLOW!]", RiemannXi);
        });
    }

    fn non_analytic_menu(&mut self, ui: &mut Ui)
    {
        ui.menu_button("Non-analytic maps", |ui| {
            ui.menu_button("Tricorne", |ui| {
                seq!(D in 2..=5 {
                    fractal_menu_button!(self, ui, format!("Degree {}", D), Tricorne<D>);
                });
            });
            ui.menu_button("Burning Ship", |ui| {
                seq!(D in 2..=5 {
                    fractal_menu_button!(self, ui, format!("Degree {}", D), BurningShip<D>);
                });
            });
            fractal_menu_button!(self, ui, "Sailboat Param", SailboatParam);
            fractal_menu_button!(self, ui, "Rulkov Map", Rulkov);
        });
    }

    fn change_fractal<P, J, C, M, T>(&mut self, create_plane: fn() -> P, create_child: fn(P) -> J)
    where
        P: ParameterPlane + Clone + 'static,
        J: ParameterPlane<MetaParam = M, Child = C> + Clone + 'static,
        C: ParameterPlane + From<J>,
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

    pub fn process_interface_message(&mut self, _ui: &mut Ui)
    {
        use super::pane::UIMessage::{CloseWindow, DoNothing, Quit};
        match self.interface.pop_message()
        {
            DoNothing =>
            {}
            CloseWindow =>
            {
                // ui.close();
            }
            Quit =>
            {
                std::process::exit(0);
            }
        }
    }
}
// }}}

impl Default for FractalTab
{
    fn default() -> Self
    {
        type Profile = Mandelbrot;

        let height = 768;

        let parent_plane = Profile::default().with_res_y(height).with_max_iter(1024);
        let child_plane = <Profile as ParameterPlane>::Child::from(parent_plane.clone());

        let interface = Box::new(MainInterface::new(parent_plane, child_plane, height));

        Self {
            interface,
            node: NodeIndex(0),
        }
    }
}
// (-1)^k * c * chebyshev(z/2k)
