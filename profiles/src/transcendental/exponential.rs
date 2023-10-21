use std::f64::consts::PI;

use crate::macros::{degree_impl_transcendental, profile_imports};
use dynamo_common::math_utils::slog;
profile_imports!();

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Exponential
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl Exponential
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -7.,
        max_x: 7.,
        min_y: -7.,
        max_y: 7.,
    };
}
impl Default for Exponential
{
    fractal_impl!();
}

impl ParameterPlane for Exponential
{
    parameter_plane_impl!();
    default_name!();
    default_bounds!();

    #[inline]
    fn map(&self, z: Cplx, lambda: Cplx) -> Cplx
    {
        z.exp() * lambda
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, lambda: Self::Param) -> (Self::Var, Self::Deriv)
    {
        let u = z.exp() * lambda;
        (u, u)
    }

    #[inline]
    fn dynamical_derivative(&self, z: Cplx, _lambda: Cplx) -> Cplx
    {
        z.exp()
    }

    #[inline]
    fn parameter_derivative(&self, _z: Cplx, _lambda: Cplx) -> Cplx
    {
        ONE
    }

    #[inline]
    fn gradient(&self, z: Cplx, lambda: Cplx) -> (Cplx, Cplx, Cplx)
    {
        let u = z.exp();
        (u + lambda, u, ONE)
    }

    #[inline]
    fn param_map(&self, lambda: Cplx) -> Cplx
    {
        lambda
    }

    #[inline]
    fn start_point(&self, _point: Cplx, _c: Self::Param) -> Self::Var
    {
        ZERO
    }

    #[inline]
    fn critical_points_child(&self, _param: Self::Param) -> Vec<Self::Var>
    {
        vec![ZERO]
    }

    fn default_julia_bounds(&self, _point: Cplx, lambda: Self::Param) -> Bounds
    {
        Bounds::square(5., lambda)
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CosineAdd
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl CosineAdd
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -7.,
        max_x: 7.,
        min_y: -7.,
        max_y: 7.,
    };
}

impl Default for CosineAdd
{
    fractal_impl!();
}

impl ParameterPlane for CosineAdd
{
    parameter_plane_impl!();
    default_name!();
    default_bounds!();

    #[inline]
    fn map(&self, z: Cplx, c: Cplx) -> Cplx
    {
        z.cos() + c
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, c: Self::Param) -> (Self::Var, Self::Deriv)
    {
        (z.cos() + c, -z.sin())
    }

    #[inline]
    fn dynamical_derivative(&self, z: Cplx, _c: Cplx) -> Cplx
    {
        -z.sin()
    }

    #[inline]
    fn parameter_derivative(&self, _z: Cplx, _c: Cplx) -> Cplx
    {
        ONE
    }

    #[inline]
    fn param_map(&self, c: Cplx) -> Cplx
    {
        c
    }

    #[inline]
    fn start_point(&self, _point: Cplx, _c: Self::Param) -> Self::Var
    {
        ZERO
    }

    fn default_julia_bounds(&self, _point: Cplx, _c: Self::Param) -> Bounds
    {
        Bounds::centered_square(5.5)
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Cosine
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl Cosine
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -7.,
        max_x: 7.,
        min_y: -7.,
        max_y: 7.,
    };
}
impl Default for Cosine
{
    fractal_impl!();
}

impl ParameterPlane for Cosine
{
    parameter_plane_impl!();
    default_name!();
    default_bounds!();

    #[inline]
    fn map(&self, z: Cplx, lambda: Cplx) -> Cplx
    {
        z.cos() * lambda
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, lambda: Self::Param) -> (Self::Var, Self::Deriv)
    {
        (z.cos() * lambda, -z.sin() * lambda)
    }

    #[inline]
    fn dynamical_derivative(&self, z: Cplx, lambda: Cplx) -> Cplx
    {
        -z.sin() * lambda
    }

    #[inline]
    fn parameter_derivative(&self, z: Cplx, _lambda: Cplx) -> Cplx
    {
        z.cos()
    }

    #[inline]
    fn param_map(&self, lambda: Cplx) -> Cplx
    {
        lambda
    }

    #[inline]
    fn start_point(&self, _point: Cplx, _c: Self::Param) -> Self::Var
    {
        ZERO
    }

    fn default_julia_bounds(&self, _point: Cplx, _param: Self::Param) -> Bounds
    {
        Bounds::centered_square(5.5)
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SineWander
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl SineWander
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -7.,
        max_x: 7.,
        min_y: -7.,
        max_y: 7.,
    };
}

impl Default for SineWander
{
    fractal_impl!();
}

impl ParameterPlane for SineWander
{
    parameter_plane_impl!();
    default_name!();
    default_bounds!();

    #[inline]
    fn map(&self, z: Cplx, c: Cplx) -> Cplx
    {
        z.sin() + z + TAU * c
    }

    #[inline]
    fn dynamical_derivative(&self, z: Cplx, _c: Cplx) -> Cplx
    {
        z.cos() + 1.
    }

    #[inline]
    fn parameter_derivative(&self, _z: Cplx, _c: Cplx) -> Cplx
    {
        TAU.into()
    }

    #[inline]
    fn start_point(&self, _point: Cplx, _c: Self::Param) -> Self::Var
    {
        PI.into()
    }

    fn default_selection(&self) -> Cplx
    {
        ONE
    }

    fn default_julia_bounds(&self, _point: Cplx, _param: Self::Param) -> Bounds
    {
        Bounds::centered_square(5.5)
    }
}

degree_impl_transcendental!(Exponential);
degree_impl_transcendental!(Cosine);
degree_impl_transcendental!(CosineAdd);
degree_impl_transcendental!(SineWander);
