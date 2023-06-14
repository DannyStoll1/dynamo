use crate::math_utils::solve_cubic;
use crate::types::CplxPair;
use crate::{macros::*, math_utils::solve_quadratic};
profile_imports!();

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct QuadRatPreper22
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl Default for QuadRatPreper22
{
    fractal_impl!(-2.0, 3.0, -2.5, 2.5);
}

impl ParameterPlane for QuadRatPreper22
{
    type Var = Cplx;
    type Param = CplxPair;
    type Deriv = Cplx;
    type MetaParam = NoParam;
    type Child = JuliaSet<Self>;
    basic_plane_impl!();
    default_name!();

    fn map(&self, z: Self::Var, CplxPair { a, b }: Self::Param) -> Self::Var
    {
        a / z * (1. - z) / (b + z)
    }

    fn dynamical_derivative(&self, z: Self::Var, CplxPair { a, b }: Self::Param) -> Self::Deriv
    {
        let u = z * (b + z);
        -a * (b + z * (2. - z)) / (u * u)
    }

    fn map_and_multiplier(
        &self,
        z: Self::Var,
        CplxPair { a, b }: Self::Param,
    ) -> (Self::Var, Self::Deriv)
    {
        let u = z * (b + z);
        (a * (1. - z) / u, -a * (b + z * (2. - z)) / (u * u))
    }

    fn param_map(&self, t: Cplx) -> Self::Param
    {
        let t = t.inv();
        let t2 = t * t;
        CplxPair {
            a: -t2,
            b: t2 + t + t,
        }
    }

    fn parameter_derivative(&self, _z: Self::Var, _c: Self::Param) -> Self::Deriv
    {
        ONE
    }

    fn start_point(&self, _t: Cplx, CplxPair { a, b }: Self::Param) -> Self::Var
    {
        1. + (b + 1.).sqrt() * (b + a + 2.).re.signum()
    }

    fn encode_escaping_point(
        &self,
        iters: Period,
        z: Cplx,
        CplxPair { a: _, b }: Self::Param,
    ) -> PointInfo<Self::Deriv>
    {
        if z.is_nan()
        {
            return PointInfo::Escaping {
                potential: f64::from(iters) - 2.,
            };
        }

        let expansion_rate = b.norm_sqr();
        let u = self.escape_radius().log(expansion_rate);
        let v = z.norm_sqr().log(expansion_rate);
        let residual = u - v;
        let potential = IterCount::from(iters) + 2.*(residual as IterCount);
        PointInfo::Escaping { potential }
    }

    fn critical_points_child(&self, CplxPair { a, b }: Self::Param) -> Vec<Self::Var>
    {
        let disc = (b + 1.).sqrt();
        vec![1. + disc, 1. - disc]
    }

    #[inline]
    fn cycles_child(&self, CplxPair { a, b }: Self::Param, period: Period) -> Vec<Self::Var> {
        match period {
            1 => {
                solve_cubic(-a, a, b).to_vec()
            }
            2 => vec![ZERO],
            _ => vec![]
        }
    }
}
