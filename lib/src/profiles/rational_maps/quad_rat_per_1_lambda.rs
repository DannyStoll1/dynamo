use crate::math_utils::solve_cubic;
use crate::types::CplxPair;
use crate::{macros::*, math_utils::solve_quadratic};
profile_imports!();

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct QuadRatPer1Lambda
{
    point_grid: PointGrid,
    max_iter: Period,
    multiplier: Cplx,
}

impl Default for QuadRatPer1Lambda
{
    fn default() -> Self
    {
        let bounds = Bounds::centered_square(3.);
        let point_grid = PointGrid::new_by_res_y(1024, bounds);
        Self {
            point_grid,
            max_iter: 1024,
            multiplier: ZERO,
        }
    }
}

impl ParameterPlane for QuadRatPer1Lambda
{
    type Var = Cplx;
    type Param = CplxPair;
    type Deriv = Cplx;
    type MetaParam = Cplx;
    type Child = JuliaSet<Self>;
    basic_plane_impl!();
    default_name!();

    fn map(&self, z: Self::Var, CplxPair { a, b }: Self::Param) -> Self::Var
    {
        let z2 = z * z;
        (z2 + a) / (z2 + b)
    }

    fn param_map(&self, t: Cplx) -> Self::Param
    {
        let alpha = 0.25 * t / self.multiplier;
        CplxPair {
            a: alpha * t * (t - self.multiplier + 2.),
            b: -alpha * (4. + (self.multiplier + 2.) * t),
        }
    }

    fn map_and_multiplier(
        &self,
        z: Self::Var,
        CplxPair { a, b }: Self::Param,
    ) -> (Self::Var, Self::Deriv)
    {
        let z2 = z * z;
        let denom = (z2 + b).inv();
        ((z2 + a) * denom, (z + z) * (b - a) * denom * denom)
    }

    fn dynamical_derivative(&self, z: Self::Var, CplxPair { a, b }: Self::Param) -> Self::Deriv
    {
        let denom = z * z + b;
        2. * z * (b - a) / (denom * denom)
    }

    fn parameter_derivative(&self, _z: Self::Var, _c: Self::Param) -> Self::Deriv
    {
        ONE
    }

    fn start_point(&self, _point: Cplx, _c: Self::Param) -> Self::Var
    {
        ONE
    }

    fn critical_points_child(&self, _c: Self::Param) -> Vec<Self::Var>
    {
        vec![ZERO]
    }

    fn cycles_child(&self, CplxPair { a, b }: Self::Param, period: Period) -> Vec<Self::Var>
    {
        match period
        {
            1 => solve_cubic(-a, b, -ONE).to_vec(),
            2 =>
            {
                let denom = (b + 1.).inv();
                solve_quadratic((a + b * b) * denom, (b - a) * denom).to_vec()
            }
            _ => vec![],
        }
    }

    fn get_param(&self) -> <Self::MetaParam as ParamList>::Param
    {
        self.multiplier
    }

    fn set_param(&mut self, lambda: <Self::MetaParam as ParamList>::Param)
    {
        self.multiplier = lambda
    }
}

#[derive(Clone, Debug)]
pub struct QuadRatPer1LambdaParam
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl QuadRatPer1LambdaParam
{
    const BASE_POINT: Cplx = Cplx::new(1e-4, 0.);

    fn base_param(lambda: Cplx) -> Cplx
    {
        -Self::BASE_POINT.inv() - 0.75 * lambda * Self::BASE_POINT
    }
}
impl Default for QuadRatPer1LambdaParam
{
    fn default() -> Self
    {
        let bounds = Bounds {
            min_x: -2.2,
            max_x: 4.2,
            min_y: -2.5,
            max_y: 2.5,
        };
        let point_grid = PointGrid::new_by_res_y(1024, bounds);
        Self {
            point_grid,
            max_iter: 1024,
        }
    }
}

impl ParameterPlane for QuadRatPer1LambdaParam
{
    type Var = Cplx;
    type Param = Cplx;
    type Deriv = Cplx;
    type MetaParam = NoParam;
    type Child = QuadRatPer1Lambda;
    basic_plane_impl!();
    basic_escape_encoding!(2., 1.);

    #[inline]
    fn map(&self, z: Self::Var, l: Self::Param) -> Self::Var
    {
        let a = -4.*l/horner_monic!(l, 8., 12., 6.);
        1. + a / (z * z)
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, l: Self::Param) -> (Self::Var, Self::Deriv)
    {
        let a = -4.*l/horner_monic!(l, 8., 12., 6.);
        let z2 = z * z;
        (1. + a / z2, -(a + a) / (z2 * z))
    }

    #[inline]
    fn dynamical_derivative(&self, z: Self::Var, l: Self::Param) -> Self::Deriv
    {
        let a = -4.*l/horner_monic!(l, 8., 12., 6.);
        -(a + a) / (z * z * z)
    }

    #[inline]
    fn parameter_derivative(&self, z: Self::Var, _c: Self::Param) -> Self::Deriv
    {
        (z * z).inv()
    }

    #[inline]
    fn start_point(&self, _point: Cplx, _c: Self::Param) -> Self::Var
    {
        ONE
    }


    fn critical_points_child(&self, _param: Self::Param) -> Vec<Self::Var>
    {
        vec![ZERO, ONE]
    }

    fn name(&self) -> String
    {
        "QuadRat Per(1, λ) λ-plane".to_owned()
    }

    fn default_selection(&self) -> Cplx
    {
        ONE
    }

    fn default_julia_bounds(&self, _point: Cplx, _param: Self::Param) -> Bounds
    {
        let r = 4.;
        Bounds::centered_square(r)
    }
}

impl From<QuadRatPer1LambdaParam> for QuadRatPer1Lambda
{
    fn from(parent: QuadRatPer1LambdaParam) -> Self
    {
        let point = parent.default_selection();
        let param = parent.param_map(point);
        let point_grid = parent
            .point_grid()
            .new_with_same_height(parent.default_julia_bounds(point, param));
        Self {
            point_grid,
            max_iter: parent.max_iter(),
            multiplier: point,
        }
    }
}
