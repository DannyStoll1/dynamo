#![allow(unused_imports)]

pub mod macros;

pub mod polynomials;
pub use polynomials::*;

pub mod rational_maps;
pub use rational_maps::*;

pub mod transcendental;
pub use transcendental::*;

pub mod non_analytic;
pub use non_analytic::*;

#[cfg(test)]
mod tests
{
    use crate::*;
    use dynamo_common::coloring::palette::ColorPalette;
    use dynamo_common::coloring::Coloring;
    use dynamo_common::prelude::*;
    use dynamo_core::prelude::*;

    #[test]
    fn test_horner()
    {
        use crate::macros::horner;
        let x = 2;
        let res = horner!(x, 2, -3, 5);
        assert_eq!(res, 16);
    }

    #[test]
    fn test_horner_monic()
    {
        use crate::macros::horner_monic;
        let x = 2;
        let res = horner_monic!(x, 2, -3, 5);
        assert_eq!(res, 24);
    }

    // #[cfg(feature = "mpsolve")]
    // #[test]
    // fn mpsolve()
    // {
    //     use crate::consts::*;
    //     use math_utils::math_utils::poly_solve::solve_polynomial;
    //     let poly = vec![-ONE, ZERO, ZERO, ONE];
    //     let roots = solve_polynomial(poly);
    //     dbg!(roots);
    // }

    #[test]
    fn chebyshev()
    {
        let chebyshev: Chebyshev<3> = Default::default();
        let c = Cplx::new(1.0, 0.0);
        let z = Cplx::new(10.0, 0.0);
        let (val, mul) = chebyshev.map_and_multiplier(z, c);
        dbg!(val, mul);
        assert!((val + 470_449.).norm() < 1e-2);
        assert!((mul + 288_090.).norm() < 1e-2);
    }

    // #[test]
    // fn erf()
    // {
    //     use math_utils::math_utils::erf::erf;
    //     let s = 0.3;
    //     let val = erf(s);
    //     let val_true = 0.328626759459127;
    //     let err = (val - val_true).abs();
    //     dbg!(err);
    //     assert!(err < 1e-11);
    // }

    // Test the result of an orbit
    // Failed cycle detection is often the result of
    // conflicting `map` and `map_and_multiplier` implementations.
    #[test]
    fn orbit()
    {
        let plane: Tricorne<4> = Default::default();
        let param = Cplx::new(0.3, 0.1);
        let start = Cplx::from(0.0);

        let orbit_params = OrbitParams {
            max_iter: 256,
            min_iter: 1,
            periodicity_tolerance: 1e-12,
            escape_radius: 1e6,
        };

        let mut orbit = CycleDetectedOrbitFloyd::new(
            |z, c| plane.map(z, c),
            |z, c| plane.map_and_multiplier(z, c),
            |z, c| plane.early_bailout(z, c),
            start,
            param,
            &orbit_params,
        );

        let result = orbit.run_until_complete();
        dbg!(&result);
        assert!(matches!(result, EscapeResult::Periodic { .. }));
    }

    #[test]
    fn find_nearby_preperiodic()
    {
        let param_plane = Mandelbrot::default();

        // Parameter plane
        {
            let o = OrbitSchema {
                period: 2,
                preperiod: 2,
            };
            let start = Cplx::new(0.2, 1.2);
            let target = Cplx::new(0., 1.);
            let approx = param_plane.find_nearby_preperiodic_point(start, o);

            let approx = approx.expect("Failed to converge");
            let error = approx.dist_sqr(target);
            println!("Parameter error: {:.4e}", approx.dist_sqr(target));
            assert!(error < 1e-5);
        }

        // Dynamical plane
        {
            let c = Cplx::new(-1.75, 0.);
            let dynam_plane = JuliaSet::from(param_plane).with_param(c);
            let o = OrbitSchema {
                preperiod: 0,
                period: 3,
            };
            let start = Cplx::new(1.5, 0.1);
            let target = Cplx::new(1.301_937_735_804_84, 0.);
            let approx = dynam_plane.find_nearby_preperiodic_point(start, o);

            let approx = approx.expect("Failed to converge");

            let error = approx.dist_sqr(target);
            println!("Dynamical error: {:.4e}", approx.dist_sqr(target));
            assert!(error < 1e-5);
        }
    }

    #[test]
    fn debug_find_point()
    {
        let param_plane = QuadRatPer4::default();
        {
            let o = OrbitSchema {
                period: 3,
                preperiod: 0,
            };
            let start = Cplx::new(0.2, 1.2);
            let target = Cplx::new(0., 1.);
            let approx = param_plane.find_nearby_preperiodic_point(start, o);

            let approx = approx.expect("Failed to converge");

            let error = approx.dist_sqr(target);
            println!("Parameter error: {:.4e}", approx.dist_sqr(target));
            assert!(error < 1e-5);
        }
    }

    #[test]
    fn equipotential()
    {
        let param_plane = Mandelbrot::default();
        {
            let t = Cplx::new(-1.4, 0.5);
            param_plane.equipotential(t);
        }
    }

    #[test]
    fn per10_debug()
    {
        let param_plane = CubicPer1_0::default().marked_cycle_curve(1);
        dbg!(param_plane.point_grid());
        let julia = JuliaSet::from(param_plane);
        dbg!(julia.point_grid());
    }

    #[test]
    fn ext_ray()
    {
        // let param_plane = Mandelbrot::default();
        let param_plane: Chebyshev<2> = Default::default();
        let angle = RationalAngle::new(1, 3);
        let ray = param_plane.external_ray(angle);
        dbg!(ray);
        // let target = Cplx::new(-0.125, 0.649519052838329);
        // assert!((ray.last().unwrap() - target).norm_sqr() < 1e-4);
    }

    #[test]
    fn escaping_period()
    {
        let plane = QuadRatPer3::default();
        assert_eq!(plane.escaping_period(), 3);
    }

    #[test]
    fn escape_coeff()
    {
        let plane = QuadRatPer4::default();
        let c = Cplx::new(4.2, 0.0);
        let q = plane.escape_coeff(c);
        assert!((q - 0.119_960_462_401_084).norm_sqr() < 1e-12);
    }

    #[test]
    fn internal_potential_coloring()
    {
        let plane = Mandelbrot::default();
        let c0 = Cplx::from(-1.040_624);
        // let c1 = Cplx::from(-0.140_626);

        let res0 = plane.get_orbit_info(c0).result;
        // let res1 = plane.get_orbit_info(c1).result;
        dbg!(&res0);

        let calg = plane.preperiod_smooth_coloring();
        let palette = ColorPalette::default();
        let mut coloring = Coloring::new(calg, palette);

        coloring
            .load_palette("../palettes/test_palette.toml")
            .expect("Failed to load palette");

        let col0 = coloring.map_color32(&res0);
        // let col1 = coloring.map_color32(res1);

        dbg!(col0);
        // dbg!(col1);
    }
}
