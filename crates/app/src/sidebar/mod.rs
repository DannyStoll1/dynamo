pub mod menu;
use dynamo_common::prelude::*;
use dynamo_core::prelude::*;
use dynamo_gui::interface::{Interface, MainInterface, PanePair as _};
use dynamo_profiles::{
    BiquadraticMult, BiquadraticMultParam, BiquadraticMultSection, BurningShip, Chebyshev,
    CoshNewton, Cosine, CosineAdd, CubicMarked2Cycle, CubicPer1_0, CubicPer1_1, CubicPer1Lambda,
    CubicPer1LambdaModuli, CubicPer1LambdaParam, CubicPer2CritMarked, CubicPer2Lambda,
    CubicPer2LambdaParam, CubicPer3_0, EisensteinMandel, Exponential, GaussianMandel, Gudermannian,
    Mandelbrot, McMullenFamily, MinsikHanPhi, NewtonCubic, OddCubic, QuadRatPer1_1,
    QuadRatPer1Lambda, QuadRatPer1LambdaParam, QuadRatPer2, QuadRatPer2InfPuncture,
    QuadRatPer2Lambda, QuadRatPer2LambdaParam, QuadRatPer3, QuadRatPer4, QuadRatPer5,
    QuadRatPreper21, QuadRatPreper22, QuadRatSymmetryLocus, RealCubicImagCrit, RealCubicRealCrit,
    RiemannXi, RiemannXiNewton, Rulkov, Sailboat, SineWander, Tricorne, Unicorn, Unicritical,
};
use menu::{Menu, State};
use seq_macro::seq;

use crate::macros::{interface, interface_dyn, interface_mc, interface_mis};

type InterfaceFactory = fn() -> Box<dyn Interface>;
type MenuEntry = (&'static str, InterfaceFactory);
type MarkedPointMenu<const C: usize, const P: usize, const M: usize> =
    (&'static str, InterfaceFactory, MarkedPointSections<C, P, M>);

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

fn polynomials_menu() -> State
{
    State::submenu()
        .with_submenu("Quadratic Family", quadratic_family_menu)
        .with_submenu("Cubic Family", cubic_family_menu)
        .with_submenu("Unicritical Maps\nz -> c*(1+z/d)^d", unicritical_menu)
        .with_submenu(
            "Chebyshev family\nz -> (-1)^k * c * T_2k(z/2)",
            chebyshev_menu,
        )
        .with_submenu("Biquadratic Maps", biquadratic_menu)
}

fn rational_maps_menu() -> State
{
    State::submenu()
        .with_submenu("QuadRat Per(2)", quad_rat_per_2_menu)
        .with_submenu("QuadRat Per(3)", quad_rat_per_3_menu)
        .with_submenu("QuadRat Per(4)", quad_rat_per_4_menu)
        .with_fractal_button("QuadRat Per(5)", interface!(QuadRatPer5))
        .with_submenu("QuadRat Preper(2, 1)", quad_rat_preper_21_menu)
        .with_fractal_button("QuadRat Preper(2, 2)", interface!(QuadRatPreper22))
        .with_submenu("QuadRat Per(1, λ)", quad_rat_per_1_lambda_menu)
        .with_submenu("QuadRat Per(2, λ)", quad_rat_per_2_lambda_menu)
        .with_fractal_button("QuadRat Symmetry Locus", interface!(QuadRatSymmetryLocus))
        .with_fractal_button("Newton Cubic", interface!(NewtonCubic))
        .with_submenu("McMullen Family\nz -> z^m + 1/(c*z^n)", mcmullen_menu)
        .with_submenu("Minsik Han Φ\nz -> az/(z^d+d-1)", minsik_han_menu)
}

fn buttons_menu(entries: &[MenuEntry]) -> State
{
    entries
        .iter()
        .fold(State::submenu(), |state, (name, create)| {
            state.with_fractal_button(name, *create)
        })
}

fn with_button_submenu<const N: usize>(
    state: State,
    title: &'static str,
    entries: [MenuEntry; N],
) -> State
{
    if entries.is_empty() {
        state
    } else {
        state.with_submenu(title, move || buttons_menu(&entries))
    }
}

fn add_marked_point_sections<const C: usize, const P: usize, const M: usize>(
    state: State,
    cycles: [MenuEntry; C],
    periodic: [MenuEntry; P],
    preperiodic: [MenuEntry; M],
) -> State
{
    let state = with_button_submenu(state, "Marked Cycle", cycles);
    let state = with_button_submenu(state, "Marked Periodic Point", periodic);
    with_button_submenu(state, "Marked Preperiodic Point", preperiodic)
}

fn marked_points_menu<const C: usize, const P: usize, const M: usize>(
    base_title: &'static str,
    base_curve: InterfaceFactory,
    point_sections: MarkedPointSections<C, P, M>,
) -> State
{
    add_marked_point_sections(
        State::submenu().with_fractal_button(base_title, base_curve),
        point_sections.cycles,
        point_sections.periodic,
        point_sections.preperiodic,
    )
}

#[derive(Clone, Copy)]
struct MarkedPointSections<const C: usize, const P: usize, const M: usize>
{
    cycles:      [MenuEntry; C],
    periodic:    [MenuEntry; P],
    preperiodic: [MenuEntry; M],
}

fn with_marked_point_submenu<const C: usize, const P: usize, const M: usize>(
    state: State,
    title: &'static str,
    menu: MarkedPointMenu<C, P, M>,
) -> State
{
    state.with_submenu(title, move || marked_points_menu(menu.0, menu.1, menu.2))
}

fn quadratic_family_menu() -> State
{
    marked_points_menu(
        "Base Curve",
        interface!(Mandelbrot),
        MarkedPointSections {
            cycles:      [
                ("Period 1", interface_mc!(Mandelbrot, 1)),
                ("Period 3", interface_mc!(Mandelbrot, 3)),
                ("Period 4", interface_mc!(Mandelbrot, 4)),
            ],
            periodic:    [
                ("Period 1", interface_mc!(Mandelbrot, 1)),
                ("Period 2", interface_dyn!(Mandelbrot, 2)),
                ("Period 3", interface_dyn!(Mandelbrot, 3)),
            ],
            preperiodic: [
                ("Preperiod 2, Period 1", interface_mis!(Mandelbrot, 2, 1)),
                ("Preperiod 2, Period 2", interface_mis!(Mandelbrot, 2, 2)),
            ],
        },
    )
}

fn cubic_family_menu() -> State
{
    let cubic_family = State::submenu()
        .with_submenu("Real Slices", || {
            buttons_menu(&[
                ("Real critical point", interface!(RealCubicRealCrit)),
                ("Imag critical point", interface!(RealCubicImagCrit)),
            ])
        })
        .with_fractal_button("Per(3)", interface!(CubicPer3_0))
        .with_submenu("Cubic Per(1, λ)", cubic_per_1_lambda_menu)
        .with_submenu("Per(2, λ)", cubic_per_2_lambda_menu)
        .with_submenu("2-cycle 0 <-> 1", cubic_marked_2_cycle_menu);

    with_cubic_marked_point_menus(cubic_family)
}

fn with_cubic_marked_point_menus(state: State) -> State
{
    let state = with_cubic_base_menus(state);
    with_cubic_per_menus(state)
}

fn with_cubic_base_menus(state: State) -> State
{
    with_marked_point_submenu(
        state,
        "Odd Cubics",
        (
            "Base curve",
            interface!(OddCubic),
            MarkedPointSections {
                cycles:      [
                    ("Period 1", interface_mc!(OddCubic, 1)),
                    ("Period 2", interface_mc!(OddCubic, 2)),
                ],
                periodic:    [
                    ("Period 1", interface_dyn!(OddCubic, 1)),
                    ("Period 2", interface_dyn!(OddCubic, 2)),
                ],
                preperiodic: [
                    ("Preperiod 1, Period 1", interface_mis!(OddCubic, 1, 1)),
                    ("Preperiod 1, Period 2", interface_mis!(OddCubic, 1, 2)),
                ],
            },
        ),
    )
}

fn with_cubic_per_menus(state: State) -> State
{
    let state = with_marked_point_submenu(
        state,
        "Cubic Per(1)",
        (
            "Base Curve",
            interface!(CubicPer1_0),
            MarkedPointSections {
                cycles:      [
                    ("Period 1", interface_mc!(CubicPer1_0, 1)),
                    ("Period 2", interface_mc!(CubicPer1_0, 2)),
                ],
                periodic:    [
                    ("Period 1", interface_dyn!(CubicPer1_0, 1)),
                    ("Period 2", interface_dyn!(CubicPer1_0, 2)),
                ],
                preperiodic: [("Preperiod 1, Period 1", interface_mis!(CubicPer1_0, 1, 1))],
            },
        ),
    );
    let state = with_marked_point_submenu(
        state,
        "Cubic Per(2)",
        (
            "Base curve",
            interface!(CubicPer2CritMarked),
            MarkedPointSections {
                cycles:      [
                    ("Period 1", interface_mc!(CubicPer2CritMarked, 1)),
                    ("Period 2", interface_mc!(CubicPer2CritMarked, 2)),
                ],
                periodic:    [],
                preperiodic: [],
            },
        ),
    );
    with_marked_point_submenu(
        state,
        "Cubic Per(1, 1)",
        (
            "Base Curve",
            interface!(CubicPer1_1),
            MarkedPointSections {
                cycles:      [("Period 2", interface_mc!(CubicPer1_1, 2))],
                periodic:    [("Period 2", interface_dyn!(CubicPer1_1, 2))],
                preperiodic: [("Preperiod 1, Period 1", interface_mis!(CubicPer1_1, 1, 1))],
            },
        ),
    )
}

fn cubic_per_1_lambda_menu() -> State
{
    buttons_menu(&[
        ("λ-plane", interface!(CubicPer1LambdaParam, CubicPer1Lambda)),
        (
            "λ=0.3",
            interface!(CubicPer1Lambda, with_param, Cplx::from(0.3)),
        ),
        (
            "λ=0.3 moduli",
            interface!(CubicPer1LambdaModuli, with_param, Cplx::from(0.3)),
        ),
        (
            "λ=0.2+0.7i moduli",
            interface!(CubicPer1LambdaModuli, with_param, Cplx::new(0.2, 0.7)),
        ),
        (
            "λ=0.99 moduli",
            interface!(CubicPer1LambdaModuli, with_param, Cplx::from(0.99)),
        ),
        (
            "λ=0.99i",
            interface!(CubicPer1Lambda, with_param, Cplx::new(0., 0.99)),
        ),
    ])
}

fn cubic_per_2_lambda_menu() -> State
{
    buttons_menu(&[
        ("λ-plane", interface!(CubicPer2LambdaParam, CubicPer2Lambda)),
        (
            "λ=0.3",
            interface!(CubicPer2Lambda, with_param, Cplx::from(0.3)),
        ),
        (
            "λ=0.99i",
            interface!(CubicPer2Lambda, with_param, Cplx::new(0., 0.99)),
        ),
    ])
}

fn cubic_marked_2_cycle_menu() -> State
{
    marked_points_menu(
        "Base curve",
        interface!(CubicMarked2Cycle),
        MarkedPointSections {
            cycles:      [("Period 1", interface_mc!(CubicMarked2Cycle, 1))],
            periodic:    [("Period 2", interface_dyn!(CubicMarked2Cycle, 2))],
            preperiodic: [
                (
                    "Preperiod 1, Period 1",
                    interface_mis!(CubicMarked2Cycle, 1, 1),
                ),
                (
                    "Preperiod 1, Period 2",
                    interface_mis!(CubicMarked2Cycle, 1, 2),
                ),
            ],
        },
    )
}

fn unicritical_menu() -> State
{
    let mut submenu = State::submenu();
    submenu.add_submenu("Degree 3", || {
        marked_points_menu(
            "Base curve",
            interface!(Unicritical<3>),
            MarkedPointSections {
                cycles:      [
                    ("Period 1", interface_mc!(Unicritical<3>, 1)),
                    ("Period 2", interface_mc!(Unicritical<3>, 2)),
                    ("Period 3", interface_mc!(Unicritical<3>, 3)),
                ],
                periodic:    [
                    ("Period 1", interface_mc!(Unicritical<3>, 1)),
                    ("Period 2", interface_dyn!(Unicritical<3>, 2)),
                ],
                preperiodic: [],
            },
        )
    });
    seq!(D in 4..=8 {
        submenu.add_fractal_button(&format!("Degree {}", D), interface!(Unicritical<D>));
    });
    submenu
}

fn chebyshev_menu() -> State
{
    let mut submenu = State::submenu();
    seq!(D in 1..=5 {
        submenu.add_fractal_button(&format!("Degree {}", D + D), interface!(Chebyshev<D>));
    });
    submenu
}

fn quad_rat_per_1_lambda_menu() -> State
{
    buttons_menu(&[
        (
            "λ-plane",
            interface!(QuadRatPer1LambdaParam, QuadRatPer1Lambda),
        ),
        ("λ=1", interface!(QuadRatPer1_1)),
        ("λ=-1", interface!(QuadRatPer1Lambda, with_param, -ONE)),
        ("λ=ω", interface!(QuadRatPer1Lambda, with_param, OMEGA)),
        (
            "λ=i",
            interface!(QuadRatPer1Lambda, with_param, Cplx::new(0., 1.)),
        ),
        (
            "λ=exp(φτi)",
            interface!(
                QuadRatPer1Lambda,
                with_param,
                Cplx::new(-0.737_368_878_078_320, 0.675_490_294_261_524)
            ),
        ),
    ])
}

fn quad_rat_per_2_lambda_menu() -> State
{
    buttons_menu(&[
        (
            "λ-plane",
            interface!(QuadRatPer2LambdaParam, QuadRatPer2Lambda),
        ),
        ("λ=1", interface!(QuadRatPer2Lambda, with_param, ONE)),
        (
            "λ=i",
            interface!(QuadRatPer2Lambda, with_param, Cplx::new(0., 1.)),
        ),
        (
            "λ=-3",
            interface!(QuadRatPer2Lambda, with_param, Cplx::from(-3.)),
        ),
        (
            "λ=-27",
            interface!(QuadRatPer2Lambda, with_param, Cplx::from(-27.)),
        ),
    ])
}

fn mcmullen_menu() -> State
{
    let mut submenu = State::submenu();
    seq!(N in 2..=8 {
        submenu.add_fractal_button(&format!("m=2, n={}", N), interface!(McMullenFamily<2, N>));
    });
    seq!(M in 2..=8 {
        submenu.add_fractal_button(&format!("m={m}, n={m}", m = M), interface!(McMullenFamily<M, M>));
    });
    submenu
}

fn minsik_han_menu() -> State
{
    let mut submenu = State::submenu();
    seq!(D in 2..=8 {
        submenu.add_fractal_button(&format!("Degree {d}", d = D), interface!(MinsikHanPhi<D>));
    });
    submenu
}

fn biquadratic_menu() -> State
{
    buttons_menu(&[
        ("λ-plane", interface!(BiquadraticMultParam, BiquadraticMult)),
        (
            "λ=0.3",
            interface!(BiquadraticMult, with_param, Cplx::from(0.3)),
        ),
        (
            "λ=0.2+0.7j",
            interface!(BiquadraticMult, with_param, Cplx::new(0.2, 0.7)),
        ),
        (
            "λ=0.99i",
            interface!(BiquadraticMult, with_param, Cplx::new(0., 0.99)),
        ),
        ("Section (b=1): λ-plane", interface!(BiquadraticMultSection)),
    ])
}

fn quad_rat_per_2_menu() -> State
{
    let state = State::submenu()
        .with_fractal_button("Moduli space", interface!(QuadRatPer2))
        .with_fractal_button("3-fold cover", interface!(QuadRatPer2InfPuncture));
    let state = with_button_submenu(
        state,
        "Marked Cycle",
        [
            ("Period 1", interface_mc!(QuadRatPer2, 1)),
            ("Period 4", interface_mc!(QuadRatPer2, 4)),
            ("Period 5", interface_mc!(QuadRatPer2, 5)),
        ],
    );
    let state = with_button_submenu(
        state,
        "Marked Periodic Point",
        [
            ("Period 1", interface_mc!(QuadRatPer2, 1)),
            ("Period 3", interface_dyn!(QuadRatPer2, 3)),
            ("Period 4", interface_dyn!(QuadRatPer2, 4)),
        ],
    );
    with_button_submenu(
        state,
        "Marked Preperiodic Point",
        [
            ("Preperiod 1, Period 1", interface_mis!(QuadRatPer2, 1, 1)),
            ("Preperiod 2, Period 1", interface_mis!(QuadRatPer2, 2, 1)),
            ("Preperiod 2, Period 2", interface_mis!(QuadRatPer2, 2, 2)),
        ],
    )
}

fn quad_rat_per_3_menu() -> State
{
    with_button_submenu(
        State::submenu().with_fractal_button("Base Curve", interface!(QuadRatPer3)),
        "Marked Cycle curves",
        [
            ("Period 1", interface_mc!(QuadRatPer3, 1)),
            ("Period 4", interface_mc!(QuadRatPer3, 4)),
        ],
    )
}

fn quad_rat_per_4_menu() -> State
{
    with_button_submenu(
        State::submenu().with_fractal_button("Base Curve", interface!(QuadRatPer4)),
        "Marked Cycle curves",
        [("Period 3", interface_mc!(QuadRatPer4, 3))],
    )
}

fn quad_rat_preper_21_menu() -> State
{
    with_button_submenu(
        State::submenu().with_fractal_button("Base Curve", interface!(QuadRatPreper21)),
        "Marked Cycle",
        [
            ("Period 3", interface_mc!(QuadRatPreper21, 3)),
            ("Period 4", interface_mc!(QuadRatPreper21, 4)),
        ],
    )
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
