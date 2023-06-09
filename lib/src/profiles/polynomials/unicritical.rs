use crate::{
    macros::*,
    math_utils::{roots_of_unity, solve_quadratic},
};
profile_imports!();

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Unicritical<const D: i32>
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl<const D: i32> Unicritical<D>
{
    const D_FLOAT: Real = D as Real;
    const CRIT: Cplx = Cplx::new(-Self::D_FLOAT, 0.0);
    const DEFAULT_BOUNDS: Bounds =
        Bounds::square(-Self::D_FLOAT * 1.2, Cplx::new(-Self::D_FLOAT + 1.0, 0.0));
}

impl<const D: i32> Default for Unicritical<D>
{
    fractal_impl!();
}

impl<const D: i32> ParameterPlane for Unicritical<D>
{
    parameter_plane_impl!();
    basic_escape_encoding!(Self::D_FLOAT, 1.);

    fn map(&self, z: Self::Var, c: Self::Param) -> Self::Var
    {
        c * (1. + z / Self::D_FLOAT).powi(D)
    }

    fn dynamical_derivative(&self, z: Self::Var, c: Self::Param) -> Self::Deriv
    {
        c * (1. + z / Self::D_FLOAT).powi(D - 1)
    }

    fn parameter_derivative(&self, _z: Self::Var, _c: Self::Param) -> Self::Deriv
    {
        ONE
    }

    fn start_point(&self, _point: Cplx, _c: Self::Param) -> Self::Var
    {
        ZERO
    }

    fn critical_points_child(&self, _c: Self::Param) -> Vec<Self::Var>
    {
        vec![Self::CRIT]
    }

    fn default_julia_bounds(&self, _point: Cplx, _c: Self::Param) -> Bounds
    {
        Bounds::square(Self::D_FLOAT * 1.618, Self::CRIT)
    }

    fn default_selection(&self) -> Cplx
    {
        let zeta = (TAUI / Self::D_FLOAT).exp();
        (zeta - 1.) * Self::D_FLOAT
    }

    fn name(&self) -> String
    {
        format!("Unicritical({})", D)
    }
}
