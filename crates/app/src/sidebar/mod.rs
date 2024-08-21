pub mod menu;
use crate::macros::*;
use dynamo_common::prelude::*;
use dynamo_core::prelude::*;
use dynamo_gui::interface::{Interface, MainInterface, PanePair};
use dynamo_profiles::*;
use menu::{Menu, State};
use seq_macro::seq;

pub fn create_menu() -> Menu
{
    let state = State::default()
        .with_submenu("Polynomial", polynomials_menu)
        .with_submenu("Rational", rational_maps_menu)
        .with_submenu("Transcendental", transcendental_menu)
        .with_submenu("Non-Analytic", non_analytic_menu)
        .with_submenu("Arithmetic", arithmetic_menu);
    Menu::new(state)
}

#[allow(clippy::identity_op)]
fn polynomials_menu() -> State
{
    State::submenu()
        .with_submenu("Quadratic Family", || {
            State::submenu()
                .with_fractal_button("Base Curve", interface!(Mandelbrot))
                .with_submenu("Marked Cycle", || {
                    State::submenu()
                        .with_fractal_button("Period 1", interface_mc!(Mandelbrot, 1))
                        .with_fractal_button("Period 3", interface_mc!(Mandelbrot, 3))
                        .with_fractal_button("Period 4", interface_mc!(Mandelbrot, 4))
                })
                .with_submenu("Marked Periodic Point", || {
                    State::submenu()
                        .with_fractal_button("Period 1", interface_mc!(Mandelbrot, 1))
                        .with_fractal_button("Period 2", interface_dyn!(Mandelbrot, 2))
                        .with_fractal_button("Period 3", interface_dyn!(Mandelbrot, 3))
                })
                .with_submenu("Marked Preperiodic Point", || {
                    State::submenu()
                        .with_fractal_button(
                            "Preperiod 2, Period 1",
                            interface_mis!(Mandelbrot, 2, 1),
                        )
                        .with_fractal_button(
                            "Preperiod 2, Period 2",
                            interface_mis!(Mandelbrot, 2, 2),
                        )
                    // .with_fractal_button("Preperiod 3, Period 1", interface_mis!(Mandelbrot, 3, 1))
                })
        })
        .with_submenu("Cubic Family", || {
            State::submenu()
                .with_submenu("Real Slices", || {
                    State::submenu()
                        .with_fractal_button("Real critical point", interface!(RealCubicRealCrit))
                        .with_fractal_button("Imag critical point", interface!(RealCubicImagCrit))
                })
                .with_submenu("Odd Cubics", || {
                    State::submenu()
                        .with_fractal_button("Base curve", interface!(OddCubic))
                        .with_submenu("Marked Cycle", || {
                            State::submenu()
                                .with_fractal_button("Period 1", interface_mc!(OddCubic, 1))
                                .with_fractal_button("Period 2", interface_mc!(OddCubic, 2))
                        })
                        .with_submenu("Marked Periodic Point", || {
                            State::submenu()
                                .with_fractal_button("Period 1", interface_dyn!(OddCubic, 1))
                                .with_fractal_button("Period 2", interface_dyn!(OddCubic, 2))
                        })
                        .with_submenu("Marked Preperiodic Point", || {
                            State::submenu()
                                .with_fractal_button(
                                    "Preperiod 1, Period 1",
                                    interface_mis!(OddCubic, 1, 1),
                                )
                                .with_fractal_button(
                                    "Preperiod 1, Period 2",
                                    interface_mis!(OddCubic, 1, 2),
                                )
                        })
                })
                .with_submenu("Cubic Per(1)", || {
                    State::submenu()
                        .with_fractal_button("Base Curve", interface!(CubicPer1_0))
                        .with_submenu("Marked Cycle", || {
                            State::submenu()
                                .with_fractal_button("Period 1", interface_mc!(CubicPer1_0, 1))
                                .with_fractal_button("Period 2", interface_mc!(CubicPer1_0, 2))
                        })
                        .with_submenu("Marked Periodic Point", || {
                            State::submenu()
                                .with_fractal_button("Period 1", interface_dyn!(CubicPer1_0, 1))
                                .with_fractal_button("Period 2", interface_dyn!(CubicPer1_0, 2))
                        })
                        .with_submenu("Marked Preperiodic Point", || {
                            State::submenu().with_fractal_button(
                                "Preperiod 1, Period 1",
                                interface_mis!(CubicPer1_0, 1, 1),
                            )
                        })
                })
                .with_submenu("Cubic Per(2)", || {
                    State::submenu()
                        .with_fractal_button("Base curve", interface!(CubicPer2CritMarked))
                        .with_submenu("Marked Cycle", || {
                            State::submenu()
                                .with_fractal_button(
                                    "Period 1",
                                    interface_mc!(CubicPer2CritMarked, 1),
                                )
                                .with_fractal_button(
                                    "Period 2",
                                    interface_mc!(CubicPer2CritMarked, 2),
                                )
                        })
                })
                .with_fractal_button("Per(3)", interface!(CubicPer3_0))
                .with_submenu("Cubic Per(1, 1)", || {
                    State::submenu()
                        .with_fractal_button("Base Curve", interface!(CubicPer1_1))
                        .with_submenu("Marked Cycle", || {
                            State::submenu()
                                .with_fractal_button("Period 2", interface_mc!(CubicPer1_1, 2))
                        })
                        .with_submenu("Marked Periodic Point", || {
                            State::submenu()
                                .with_fractal_button("Period 2", interface_dyn!(CubicPer1_1, 2))
                        })
                        .with_submenu("Marked Preperiodic Point", || {
                            State::submenu().with_fractal_button(
                                "Preperiod 1, Period 1",
                                interface_mis!(CubicPer1_1, 1, 1),
                            )
                        })
                })
                .with_submenu("Cubic Per(1, λ)", || {
                    State::submenu()
                        .with_fractal_button(
                            "λ-plane",
                            interface!(CubicPer1LambdaParam, CubicPer1Lambda),
                        )
                        .with_fractal_button(
                            "λ=0.3",
                            interface!(CubicPer1Lambda, with_param, Cplx::from(0.3)),
                        )
                        .with_fractal_button(
                            "λ=0.3 moduli",
                            interface!(CubicPer1LambdaModuli, with_param, Cplx::from(0.3)),
                        )
                        .with_fractal_button(
                            "λ=0.2+0.7i moduli",
                            interface!(CubicPer1LambdaModuli, with_param, Cplx::new(0.2, 0.7)),
                        )
                        .with_fractal_button(
                            "λ=0.99 moduli",
                            interface!(CubicPer1LambdaModuli, with_param, Cplx::from(0.99)),
                        )
                        .with_fractal_button(
                            "λ=0.99i",
                            interface!(CubicPer1Lambda, with_param, Cplx::new(0., 0.99)),
                        )
                })
                .with_submenu("Per(2, λ)", || {
                    State::submenu()
                        .with_fractal_button(
                            "λ-plane",
                            interface!(CubicPer2LambdaParam, CubicPer2Lambda),
                        )
                        .with_fractal_button(
                            "λ=0.3",
                            interface!(CubicPer2Lambda, with_param, Cplx::from(0.3)),
                        )
                        .with_fractal_button(
                            "λ=0.99i",
                            interface!(CubicPer2Lambda, with_param, Cplx::new(0., 0.99)),
                        )
                })
                .with_submenu("2-cycle 0 <-> 1", || {
                    State::submenu()
                        .with_fractal_button("Base curve", interface!(CubicMarked2Cycle))
                        .with_submenu("Marked Cycle", || {
                            State::submenu().with_fractal_button(
                                "Period 1",
                                interface_mc!(CubicMarked2Cycle, 1),
                            )
                        })
                        .with_submenu("Marked Periodic Point", || {
                            State::submenu().with_fractal_button(
                                "Period 2",
                                interface_dyn!(CubicMarked2Cycle, 2),
                            )
                        })
                        .with_submenu("Marked Preperiodic Point", || {
                            State::submenu()
                                .with_fractal_button(
                                    "Preperiod 1, Period 1",
                                    interface_mis!(CubicMarked2Cycle, 1, 1),
                                )
                                .with_fractal_button(
                                    "Preperiod 1, Period 2",
                                    interface_mis!(CubicMarked2Cycle, 1, 2),
                                )
                        })
                })
        })
        .with_submenu("Unicritical Maps\nz -> c*(1+z/d)^d", || {
            let mut submenu = State::submenu();
            submenu.add_submenu("Degree 3", || {
                State::submenu()
                    .with_fractal_button("Base curve", interface!(Unicritical<3>))
                    .with_submenu("Marked Cycle", || {
                        State::submenu()
                            .with_fractal_button("Period 1", interface_mc!(Unicritical<3>, 1))
                            .with_fractal_button("Period 2", interface_mc!(Unicritical<3>, 2))
                            .with_fractal_button("Period 3", interface_mc!(Unicritical<3>, 3))
                    })
                    .with_submenu("Marked Periodic Point", || {
                        State::submenu()
                            .with_fractal_button("Period 1", interface_mc!(Unicritical<3>, 1))
                            .with_fractal_button("Period 2", interface_dyn!(Unicritical<3>, 2))
                    })
            });
            seq!(D in 4..=8 {
                submenu.add_fractal_button(&format!("Degree {}", D), interface!(Unicritical<D>));
            });
            submenu
        })
        .with_submenu("Chebyshev family\nz -> (-1)^k * c * T_2k(z/2)", || {
            let mut submenu = State::submenu();
            seq!(D in 1..=5 {
                submenu.add_fractal_button(&format!("Degree {}", 2*D), interface!(Chebyshev<D>));
            });
            submenu
        })
        .with_submenu("Biquadratic Maps", || {
            State::submenu()
                .with_fractal_button("λ-plane", interface!(BiquadraticMultParam, BiquadraticMult))
                .with_fractal_button(
                    "λ=0.3",
                    interface!(BiquadraticMult, with_param, Cplx::from(0.3)),
                )
                .with_fractal_button(
                    "λ=0.2+0.7j",
                    interface!(BiquadraticMult, with_param, Cplx::new(0.2, 0.7)),
                )
                .with_fractal_button(
                    "λ=0.99i",
                    interface!(BiquadraticMult, with_param, Cplx::new(0., 0.99)),
                )
                .with_fractal_button("Section (b=1): λ-plane", interface!(BiquadraticMultSection))
        })
}
fn rational_maps_menu() -> State
{
    State::submenu()
        .with_submenu("QuadRat Per(2)", || {
            State::submenu()
                .with_fractal_button("Moduli space", interface!(QuadRatPer2))
                .with_fractal_button("3-fold cover", interface!(QuadRatPer2InfPuncture))
                .with_submenu("Marked Cycle", || {
                    State::submenu()
                        .with_fractal_button("Period 1", interface_mc!(QuadRatPer2, 1))
                        .with_fractal_button("Period 4", interface_mc!(QuadRatPer2, 4))
                        .with_fractal_button("Period 5", interface_mc!(QuadRatPer2, 5))
                })
            .with_submenu("Marked Periodic Point", || {
                State::submenu()
                    .with_fractal_button("Period 1", interface_mc!(QuadRatPer2, 1))
                    .with_fractal_button("Period 3", interface_dyn!(QuadRatPer2, 3))
                    .with_fractal_button("Period 4", interface_dyn!(QuadRatPer2, 4))
            })
            .with_submenu("Marked Preperiodic Point", || {
                State::submenu()
                    .with_fractal_button("Preperiod 1, Period 1", interface_mis!(QuadRatPer2, 1, 1))
                    .with_fractal_button("Preperiod 2, Period 1", interface_mis!(QuadRatPer2, 2, 1))
                    .with_fractal_button("Preperiod 2, Period 2", interface_mis!(QuadRatPer2, 2, 2))
            })
        })
    .with_submenu("QuadRat Per(3)", || {
        State::submenu()
            .with_fractal_button("Base Curve", interface!(QuadRatPer3))
            .with_submenu("Marked Cycle curves", || {
                State::submenu()
                    .with_fractal_button("Period 1", interface_mc!(QuadRatPer3, 1))
                    .with_fractal_button("Period 4", interface_mc!(QuadRatPer3, 4))
            })
    })
    .with_submenu("QuadRat Per(4)", || {
        State::submenu()
            .with_fractal_button("Base Curve", interface!(QuadRatPer4))
            .with_submenu("Marked Cycle curves", || {
                State::submenu()
                    .with_fractal_button("Period 3", interface_mc!(QuadRatPer4, 3))
            })
    })
    .with_fractal_button("QuadRat Per(5)", interface!(QuadRatPer5))
        .with_submenu("QuadRat Preper(2, 1)", || {
            State::submenu()
                .with_fractal_button("Base Curve", interface!(QuadRatPreper21))
                .with_submenu("Marked Cycle", || {
                    State::submenu()
                        .with_fractal_button("Period 3", interface_mc!(QuadRatPreper21, 3))
                        .with_fractal_button("Period 4", interface_mc!(QuadRatPreper21, 4))
                })
        })
    .with_fractal_button("QuadRat Preper(2, 2)", interface!(QuadRatPreper22))
        .with_submenu("QuadRat Per(1, λ)", || {
            State::submenu()
                .with_fractal_button(
                    "λ-plane",
                    interface!(QuadRatPer1LambdaParam, QuadRatPer1Lambda),
                    )
                .with_fractal_button("λ=1", interface!(QuadRatPer1_1))
                .with_fractal_button("λ=-1", interface!(QuadRatPer1Lambda, with_param, -ONE))
                .with_fractal_button("λ=ω", interface!(QuadRatPer1Lambda, with_param, OMEGA))
                .with_fractal_button(
                    "λ=i",
                    interface!(QuadRatPer1Lambda, with_param, Cplx::new(0., 1.)),
                    )
                .with_fractal_button(
                    "λ=exp(φτi)",
                    interface!(
                        QuadRatPer1Lambda,
                        with_param,
                        Cplx::new(-0.737_368_878_078_320, 0.675_490_294_261_524)
                        ),
                        )
        })
    .with_submenu("QuadRat Per(2, λ)", || {
        State::submenu()
            .with_fractal_button(
                "λ-plane",
                interface!(QuadRatPer2LambdaParam, QuadRatPer2Lambda),
                )
            .with_fractal_button("λ=1", interface!(QuadRatPer2Lambda, with_param, ONE))
            .with_fractal_button(
                "λ=i",
                interface!(QuadRatPer2Lambda, with_param, Cplx::new(0., 1.)),
                )
            .with_fractal_button(
                "λ=-3",
                interface!(QuadRatPer2Lambda, with_param, Cplx::from(-3.)),
                )
            .with_fractal_button(
                "λ=-27",
                interface!(QuadRatPer2Lambda, with_param, Cplx::from(-27.)),
                )
    })
    .with_fractal_button("QuadRat Symmetry Locus", interface!(QuadRatSymmetryLocus))
        .with_fractal_button("Newton Cubic", interface!(NewtonCubic))
        .with_submenu("McMullen Family\nz -> z^m + 1/(c*z^n)", || {
            let mut submenu = State::submenu();
            seq!(N in 2..=8 {
                submenu.add_fractal_button(
                    &format!("m=2, n={}", N),
                    interface!(McMullenFamily<2, N>),
                    );
            });
            seq!(M in 2..=8 {
                submenu.add_fractal_button(
                    &format!("m={m}, n={m}", m=M),
                    interface!(McMullenFamily<M, M>),
                    );
            });
            submenu
        })
    .with_submenu("Minsik Han Φ\nz -> az/(z^d+d-1)", || {
        let mut submenu = State::submenu();
        seq!(D in 2..=8 {
            submenu.add_fractal_button(&format!("Degree {d}", d=D), interface!(MinsikHanPhi<D>));
        });
        submenu
    })
}

fn transcendental_menu() -> State
{
    State::submenu()
        .with_fractal_button("z -> λexp(z)", interface!(Exponential))
        .with_fractal_button("z -> λcos(z)", interface!(Cosine))
        .with_fractal_button("z -> cos(z) + c", interface!(CosineAdd))
        .with_fractal_button("z -> sin(z) + z + τc", interface!(SineWander))
        .with_fractal_button("Cosh Newton", interface!(CoshNewton, CoshNewton))
        .with_fractal_button("z -> λarctan(sinh(z))", interface!(Gudermannian))
        .with_fractal_button(
            "Riemann Xi Newton [SLOW!]",
            interface!(RiemannXi, RiemannXiNewton),
        )
}

fn non_analytic_menu() -> State
{
    State::submenu()
        .with_submenu("Tricorne", || {
            let mut submenu = State::submenu();
            seq!(D in 2..=5 {
                submenu.add_fractal_button(&format!("Degree {d}", d=D), interface!(Tricorne<D>));
            });
            submenu
        })
        .with_submenu("Unicorn", || {
            let mut submenu = State::submenu();
            seq!(D in 2..=5 {
                submenu.add_fractal_button(&format!("Degree {d}", d=D), interface!(Unicorn<D>));
            });
            submenu
        })
        .with_submenu("Burning Ship", || {
            let mut submenu = State::submenu();
            seq!(D in 2..=5 {
                submenu.add_fractal_button(&format!("Degree {d}", d=D), interface!(BurningShip<D>));
            });
            submenu
        })
        .with_fractal_button("Sailboat Param", interface!(BurningShip<2>, Sailboat))
        .with_fractal_button("Rulkov Map", interface!(Rulkov))
}

fn arithmetic_menu() -> State
{
    State::submenu()
        .with_submenu("Gaussian Int Mandel", || {
            State::submenu()
                .with_fractal_button("Mod 2+ω", interface!(GaussianMandel<2, 1>))
                .with_fractal_button("Mod 5+2ω", interface!(GaussianMandel<5, 2>))
                .with_fractal_button("Mod 7", interface!(GaussianMandel<7, 0>))
                .with_fractal_button("Mod 11", interface!(GaussianMandel<11, 0>))
                .with_fractal_button("Mod 19", interface!(GaussianMandel<19, 0>))
                .with_fractal_button("Mod 107", interface!(GaussianMandel<107, 0>))
                .with_fractal_button("Mod 311", interface!(GaussianMandel<311, 0>))
        })
        .with_submenu("Eisenstein Int Mandel", || {
            State::submenu()
                .with_fractal_button("Mod 2+ω", interface!(EisensteinMandel<2, 1>))
                .with_fractal_button("Mod 5", interface!(EisensteinMandel<5, 0>))
                .with_fractal_button("Mod 5+2ω", interface!(EisensteinMandel<5, 2>))
                .with_fractal_button("Mod 11", interface!(EisensteinMandel<11, 0>))
                .with_fractal_button("Mod 17", interface!(EisensteinMandel<17, 0>))
                .with_fractal_button("Mod 107", interface!(EisensteinMandel<107, 0>))
                .with_fractal_button("Mod 311", interface!(EisensteinMandel<311, 0>))
        })
}

fn create_interface<P, J>(create_parent: fn() -> P, create_child: fn(P) -> J) -> Box<dyn Interface>
where
    P: Displayable + HasChild<J> + Clone + 'static,
    J: Displayable + Clone + 'static,
{
    let max_iters = 1024;

    let parent_plane = create_parent().with_max_iter(max_iters).with_res_y(768);
    let child_plane = create_child(parent_plane.clone());

    let mut interface = MainInterface::new(parent_plane, child_plane, 768);
    interface.update_panes();
    Box::new(interface)
}
