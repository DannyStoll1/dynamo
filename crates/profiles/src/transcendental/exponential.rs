use std::f64::consts::PI;

use dynamo_common::math_utils::slog;

use crate::macros::{degree_impl_transcendental, has_child_impl, profile_imports};
profile_imports!();

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Exponential
{
    point_grid:   PointGrid,
    compute_mode: ComputeMode,
    max_iter:     IterCount,
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

impl DynamicalFamily for Exponential
{
    parameter_plane_impl!();
    default_name!();

    #[inline]
    fn map(&self, z: Cplx, lambda: &Cplx) -> Cplx
    {
        z.exp() * lambda
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, lambda: &Self::Param) -> (Self::Var, Self::Deriv)
    {
        let u = z.exp() * lambda;
        (u, u)
    }

    #[inline]
    fn extra_stop_condition(
        &self,
        z: Self::Var,
        _c: &Self::Param,
        iter: IterCount,
    ) -> Option<EscapeResult<Self::Var, Self::Deriv>>
    {
        if z.re > 250. {
            Some(EscapeResult::Escaped {
                iters:       iter,
                final_value: z,
            })
        } else if z.re < -50. {
            None
        } else if z.im.abs() > 1e15 {
            Some(EscapeResult::Unknown)
        } else {
            None
        }
    }

    #[inline]
    fn gradient(&self, z: Cplx, lambda: &Cplx) -> (Cplx, Cplx, Cplx)
    {
        let u = z.exp();
        let v = lambda * u;
        (v, v, u)
    }

    #[inline]
    fn start_point(&self, _point: Cplx, _c: &Self::Param) -> Self::Var
    {
        ZERO
    }
}

impl FamilyDefaults for Exponential
{
    default_bounds!();
}

impl HasJulia for Exponential
{
    fn default_bounds_child(&self, _point: Cplx, lambda: &Self::Param) -> Bounds
    {
        Bounds::square(5., *lambda)
    }
}

impl MarkedPoints for Exponential
{
    #[inline]
    fn critical_points_child(&self, _param: &Self::Param) -> Vec<Self::Var>
    {
        vec![ZERO]
    }
}

degree_impl_transcendental!(Exponential);
