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

fn main() -> Result<(), Box<dyn std::error::Error>>
{
    gui::run_app()?;

    Ok(())
}

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
        let plane = Biquadratic::new_default(512, 2048, (-0.3).into());
        plane.compute();
    }

    #[test]
    fn compute_per2()
    {
        let plane = QuadRatPer2::new_default(512, 2048);
        plane.compute();
    }

    #[test]
    fn compute_per2_julia()
    {
        let plane = QuadRatPer2::new_default(512, 2048);
        let mut julia = JuliaSet::from(plane);
        for i in 0..10
        {
            let param = ComplexNum::new(0.1 * (i as f64), 0.);
            julia.set_param(param);
            julia.compute();
        }
    }

    #[test]
    fn compute_per3()
    {
        let plane = QuadRatPer3::new_default(512, 2048);
        plane.compute();
    }

    #[test]
    fn compute_per4()
    {
        let plane = QuadRatPer4::new_default(512, 2048);
        plane.compute();
    }

    #[test]
    fn compute_preper21()
    {
        let plane = QuadRatPreper21::new_default(512, 2048);
        plane.compute();
    }

    #[test]
    fn compute_symmetry_locus()
    {
        let plane = QuadRatSymmetryLocus::new_default(512, 2048);
        plane.compute();
    }

    #[test]
    fn compute_rulkov()
    {
        let mut plane = Rulkov::new_default(256, 2048);
        let bounds = Bounds {
            min_x: -9.859092283464022,
            max_x: 11.712293384188932,
            min_y: -11.185367984659074,
            max_y: 10.011947411300476,
        };

        plane.point_grid_mut().change_bounds(bounds);
        plane.compute();
    }
}
