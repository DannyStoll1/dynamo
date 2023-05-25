#![allow(dead_code)]
#![allow(unused_imports)]
#![feature(const_fn_floating_point_arithmetic)]

pub mod coloring;
pub mod dynamics;
pub mod gui;
pub mod iter_plane;
pub mod macros;
pub mod math_utils;
pub mod point_grid;
pub mod profiles;
pub mod types;

#[cfg(test)]
mod tests
{
    use crate::dynamics::julia::JuliaSet;
    use crate::dynamics::ParameterPlane;
    use crate::point_grid::Bounds;
    use crate::profiles::*;
    use crate::types::ComplexNum;

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
    fn gui_speedtest()
    {
        let height = 1024;
        use crate::gui::pane::{MainInterface, PanePair};
        let parameter_plane = QuadRatPer2::default()
            .with_res_y(height)
            .with_max_iter(2048);

        let dynamical_plane = JuliaSet::from(parameter_plane.clone());

        let mut pane_pair = Box::new(MainInterface::new(parameter_plane, dynamical_plane, height));
        for _ in 0..10
        {
            pane_pair.child_mut().recompute();
        }
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
}
