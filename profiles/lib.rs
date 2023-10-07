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
    use fractal_common::point_grid::Bounds;
    use fractal_common::prelude::{Cplx, Dist, EscapeState};
    use fractal_core::dynamics::julia::JuliaSet;
    use fractal_core::dynamics::orbit::{CycleDetectedOrbitFloyd, OrbitParams};
    use fractal_core::dynamics::symbolic::OrbitSchema;
    use fractal_core::dynamics::ParameterPlane;

    #[test]
    fn compute_biquadratic()
    {
        let plane = Biquadratic::default()
            .with_res_y(512)
            .with_max_iter(2048)
            .with_param((-0.3).into());
        plane.compute();
    }

    #[test]
    fn compute_per2()
    {
        let plane = QuadRatPer2::default().with_res_y(512).with_max_iter(2048);
        plane.compute();
    }

    #[test]
    fn compute_per2_julia()
    {
        let plane = QuadRatPer2::default().with_res_y(1024).with_max_iter(2048);
        let julia = JuliaSet::from(plane);
        let mut iter_plane = julia.compute();
        for _ in 0..9
        {
            julia.compute_into(&mut iter_plane);
        }
    }

    #[test]
    fn compute_per3()
    {
        let plane = QuadRatPer3::default().with_res_y(512).with_max_iter(2048);
        plane.compute();
    }

    #[test]
    fn compute_per4()
    {
        let plane = QuadRatPer4::default().with_res_y(512).with_max_iter(2048);
        plane.compute();
    }

    #[test]
    fn compute_preper21()
    {
        let plane = QuadRatPreper21::default()
            .with_res_y(512)
            .with_max_iter(2048);
        plane.compute();
    }

    #[test]
    fn compute_symmetry_locus()
    {
        let plane = QuadRatSymmetryLocus::default()
            .with_res_y(512)
            .with_max_iter(2048);
        plane.compute();
    }

    #[test]
    fn compute_rulkov()
    {
        let mut plane = Rulkov::default().with_res_y(256).with_max_iter(2048);
        let bounds = Bounds {
            min_x: -9.859092283464022,
            max_x: 11.712293384188932,
            min_y: -11.185367984659074,
            max_y: 10.011947411300476,
        };

        plane.point_grid_mut().change_bounds(bounds);
        plane.compute();
    }

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

    #[cfg(feature = "mpsolve")]
    #[test]
    fn mpsolve()
    {
        use crate::consts::*;
        use math_utils::math_utils::poly_solve::solve_polynomial;
        let poly = vec![-ONE, ZERO, ZERO, ONE];
        let roots = solve_polynomial(&poly);
        dbg!(roots);
    }

    #[test]
    fn chebyshev()
    {
        let chebyshev: Chebyshev<3> = Default::default();
        let c = Cplx::new(1.0, 0.0);
        let z = Cplx::new(10.0, 0.0);
        let (val, mul) = chebyshev.map_and_multiplier(z, c);
        dbg!(val, mul);
        assert!((val + 470449.).norm() < 1e-2);
        assert!((mul + 288090.).norm() < 1e-2);
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
        dbg!(result);
        assert!(matches!(result, EscapeState::Periodic { .. }));
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

            let error = approx.unwrap().dist_sqr(target);
            println!("Parameter error: {:.4e}", approx.unwrap().dist_sqr(target));
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
            let target = Cplx::new(1.30193773580484, 0.);
            let approx = dynam_plane.find_nearby_preperiodic_point(start, o);

            let error = approx.unwrap().dist_sqr(target);
            println!("Dynamical error: {:.4e}", approx.unwrap().dist_sqr(target));
            assert!(error < 1e-5);
        }
    }
}
