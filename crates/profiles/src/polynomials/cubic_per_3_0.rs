use dynamo_common::horner_monic;

use crate::macros::{degree_impl, horner, profile_imports};
profile_imports!();

// Cubic polynomials with a critical 3-cycle 0 -2-> 1 -> a+b+1 -> 0
#[derive(Clone, Debug, PartialEq)]
pub struct CubicPer3_0
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl CubicPer3_0
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -7.5,
        max_x: 2.5,
        min_y: -5.,
        max_y: 5.,
    };
}
impl Default for CubicPer3_0
{
    fractal_impl!();
}

impl DynamicalFamily for CubicPer3_0
{
    type Var = Cplx;
    type Param = CplxPair;
    type MetaParam = NoParam;
    type Deriv = Cplx;
    type Child = JuliaSet<Self>;
    basic_plane_impl!();
    default_name!();
    default_bounds!();

    #[inline]
    fn map_and_multiplier(
        &self,
        z: Self::Var,
        CplxPair { a, b }: &Self::Param,
    ) -> (Self::Var, Self::Deriv)
    {
        let z2 = z.powi(2);
        let az = a * z;
        (1. + z2 * (b + az), z * (3. * az + 2. * b))
    }

    #[inline]
    fn map(&self, z: Self::Var, CplxPair { a, b }: &Self::Param) -> Self::Var
    {
        horner!(z, 1., 0., b, a)
    }

    fn param_map(&self, t: Cplx) -> Self::Param
    {
        let u = t + 1.;
        let u2_inv = u.powi(-2);
        let a = 1. - t - 3. / u + u2_inv;
        let b = t.powi(3) * u2_inv + u / t;
        CplxPair { a, b }
    }
    #[inline]
    fn param_map_d(&self, point: Cplx) -> (Self::Param, Self::Deriv)
    {
        (self.param_map(point), ZERO)
    }
    fn start_point(&self, _point: Cplx, CplxPair { a, b }: &Self::Param) -> Self::Var
    {
        -TWO_THIRDS * b / a
    }
    fn start_point_d(
        &self,
        t: Cplx,
        CplxPair { a, b }: &Self::Param,
    ) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        let dz_dt = -TWO_THIRDS * horner!(t, 1., 4., 6., 8., 7., 2.)
            / horner_monic!(t, 1., 4., 6., 6., 5., 2.);
        (-TWO_THIRDS * b / a, dz_dt, ZERO)
    }
    fn default_selection(&self) -> Cplx
    {
        // ComplexNum::new(-3.34447065821736, 0.) // center of a capture component; c1 -2> c0=0 -2> 1 -> a+b+1 -> 0
        Cplx::new(-0.521_257_806_222_939, 0.) // center of a period 1 component; c1 -2> c1
    }
    fn default_julia_bounds(&self, point: Cplx, c: &Self::Param) -> Bounds
    {
        let crit = self.start_point(point, c);
        // let center = (crit + 2. + c.a + c.b) * ONE_THIRD;
        // Centroid of free critical point and marked critical orbit
        let center = (crit + 2. + c.a + c.b) * 0.25;
        let radius = crit.norm() + (c.a + c.b + 1.).norm() + 0.5;
        Bounds::square(radius, center)
    }
}

impl MarkedPoints for CubicPer3_0
{
    #[inline]
    fn critical_points_child(&self, CplxPair { a, b }: &Self::Param) -> Vec<Self::Var>
    {
        vec![ZERO, -TWO_THIRDS * b / a]
    }
    fn critical_points(&self) -> Vec<Self::Var>
    {
        let [r0, r1, r2] = solve_cubic(ONE, 2.0.into(), ONE);
        vec![ZERO, (-1.).into(), r0, r1, r2]
    }
    fn cycles_child(&self, Self::Param { a, b }: &Self::Param, period: Period) -> Vec<Self::Var>
    {
        match period {
            1 => {
                let ainv = a.inv();
                solve_cubic(ainv, -ainv, b * ainv).to_vec()
            }
            2 => {
                let a2 = a.powi(2);
                let coeffs = [
                    a + b + 1.,
                    a + b,
                    a + b * (b + 2. * a),
                    2. * a * (a + b),
                    a * (a + b.powi(2)),
                    2. * a2 * b,
                    a2 * a,
                ];
                solve_polynomial(coeffs)
            }
            _ => vec![],
        }
    }
}

degree_impl!(CubicPer3_0, 3);
