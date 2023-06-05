use crate::macros::*;
use crate::math_utils::weierstrass_p;
profile_imports!();

#[derive(Clone, Debug)]
pub struct QuadRatPer3
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl QuadRatPer3
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -2.5,
        max_x: 3.2,
        min_y: -2.5,
        max_y: 2.5,
    };
}
impl Default for QuadRatPer3
{
    fractal_impl!();
}

impl ParameterPlane for QuadRatPer3
{
    parameter_plane_impl!();
    default_name!();

    fn start_point(&self, _point: Cplx, _c: Cplx) -> Cplx
    {
        0.0.into()
    }

    fn encode_escaping_point(
        &self,
        iters: Period,
        z: Cplx,
        base_param: Cplx,
    ) -> PointInfo<Self::Deriv>
    {
        if z.is_nan()
        {
            return PointInfo::Escaping {
                potential: f64::from(iters) - 3.,
            };
        }

        let u = self.escape_radius().log2();
        let v = z.norm_sqr().log2();
        let q = ((base_param - 1.) / (4. * base_param)).norm().log2();
        let residual = ((u + q) / (v + q)).log2();
        let potential = (residual as IterCount).mul_add(3., f64::from(iters));
        PointInfo::Escaping { potential }
    }

    #[inline]
    fn map_and_multiplier(&self, z: Cplx, c: Cplx) -> (Cplx, Cplx)
    {
        let z2 = z * z;
        let c2 = c * c;
        let u = (z2 - c2).inv();
        let v = c + 1.;
        ((z2 + c2 * c - v) * u, 2.0 * (1. - c) * v * v * z * u * u)
    }

    #[inline]
    fn map(&self, z: Cplx, c: Cplx) -> Cplx
    {
        let z2 = z * z;
        let c2 = c * c;
        (z2 + c2 * c - c - 1.) / (z2 - c2)
    }

    #[inline]
    fn dynamical_derivative(&self, z: Cplx, c: Cplx) -> Cplx
    {
        let u = 1. / (c * c - z * z);
        let v = c + 1.;
        2.0 * (1. - c) * v * v * z * u * u
    }

    #[inline]
    fn parameter_derivative(&self, z: Cplx, c: Cplx) -> Cplx
    {
        let r = c * c - z * z;
        let u2 = 1. / (r * r);
        (c + 1.) * u2 * (r - c * (r + 2.0 * (ONE - z * z)))
    }

    #[inline]
    fn gradient(&self, z: Self::Var, c: Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        let z2 = z * z;
        let c2 = c * c;
        let r = c2 - z2;
        let u = r.inv();
        let u2 = u * u;
        let v = c + 1.;

        let f = u * (z2 + c2 * c - v);
        let df_dz = 2. * (1. - c) * v * v * z * u2;
        let df_dc = v * u2 * (r - c * (r + 2. * (ONE - z * z)));
        (f, df_dz, df_dc)
    }

    #[inline]
    fn critical_points_child(&self, _param: Cplx) -> ComplexVec
    {
        vec![(0.).into()]
    }

    fn cycles_child(&self, c: Cplx, period: Period) -> ComplexVec
    {
        match period
        {
            1 =>
            {
                let x0 = c * c;
                let x1 = c * x0;
                let x2 = 3. * x0 + 1.;
                let u = 27. * (c - x1) - 9. * x0 + 25.;
                let x3 = (0.5 * (u + (-4. * x2 * x2 * x2 + u * u).sqrt())).powf(ONE_THIRD);
                let x4 = x3 / 3.;
                let x5 = x2 / (3. * x3);
                let r1 = -x4 * OMEGA_BAR - x5 * OMEGA + ONE_THIRD;
                let r2 = -x4 * OMEGA - x5 * OMEGA_BAR + ONE_THIRD;
                vec![-x4 - x5 + ONE_THIRD, r1, r2]
            }
            2 =>
            {
                let disc = (c * (5. * c + 6.) + 5.).sqrt();
                vec![-0.5 * (c - disc + 1.), -0.5 * (c + disc + 1.)]
            }
            _ => vec![],
        }
    }

    #[inline]
    fn default_julia_bounds(&self, _point: Cplx, _param: Cplx) -> Bounds
    {
        Bounds::centered_square(4.)
    }
}
impl HasDynamicalCovers for QuadRatPer3
{
    fn marked_cycle_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> Cplx;
        let bounds: Bounds;

        match period
        {
            1 =>
            {
                param_map = |c| {
                    let pole = 1.324_717_957_244_75;
                    let c = 1. / c + pole;
                    let c2 = c * c;
                    let c3 = c2 * c;
                    (c3 - c + 1.) / (c3 - c2 - c2 + c + c + c - 1.)
                };
                bounds = Bounds {
                    min_x: -5.75,
                    max_x: 5.08,
                    min_y: -5.32,
                    max_y: 5.32,
                };
            }
            4 =>
            {
                param_map = |c| {
                    let t = (13.0 as Real).sqrt();
                    let g2 = Cplx::new(-8.0 / 3.0, 0.);
                    let g3 = Cplx::new(1.0 / 27.0, 0.);

                    let (p, dp) = weierstrass_p(g2, g3, c, 0.01);
                    let x = p - 1. / 3.;
                    let y = (dp + 1.) / x - t - 1.;

                    let u = x / 2.;
                    let v = y / 4.;
                    let xx = -(t + 1.) * u + (t + 3.) * v + (t + 4.);
                    let yy = u - v - (t + 1.) / 4.;
                    let zz = -x + 2. * v + (t + 3.) / 2.;

                    let s0 = xx / zz;
                    let s1 = zz / yy;

                    s0 * s1 + s1 + (t + 4.)
                    // let l = s0^2*s1 + s0*s1 + (2*t)*s0 + (t - 1);
                };
                bounds = Bounds {
                    min_x: -3.9,
                    max_x: 3.9,
                    min_y: -2.6,
                    max_y: 2.6,
                };
            }
            _ =>
            {
                param_map = |c| c;
                bounds = self.point_grid.bounds.clone();
            }
        };
        let grid = self.point_grid.clone().with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }
}
