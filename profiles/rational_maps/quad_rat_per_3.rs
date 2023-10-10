use crate::macros::{horner, horner_monic, profile_imports};
use fractal_common::math_utils::weierstrass_p;
profile_imports!();

// Quadratic rational maps with a critical 3-cycle: -c => ∞ -> 1 -> -c
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

    fn start_point_d(&self, _point: Cplx, _c: Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv) {
        (ZERO, ZERO, ZERO)
    }

    fn encode_escaping_point(
        &self,
        iters: Period,
        z: Cplx,
        base_param: Cplx,
    ) -> PointInfo<Self::Var, Self::Deriv>
    {
        if z.is_nan()
        {
            return PointInfo::Escaping {
                potential: f64::from(iters) - 3.,
            };
        }

        let u = self.escape_radius().log2();
        let v = z.norm_sqr().log2();
        let delta = ((base_param - 1.) / (4. * base_param)).norm().log2();
        let residual = ((u + delta) / (v + delta)).log2();
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

        let f = u * (v - z2 - c2 * c);
        let df_dz = 2. * (1. - c) * v * v * z * u2;
        let df_dc = v * u2 * (r - c * (r + 2. * (ONE - z * z)));
        (f, df_dz, df_dc)
    }

    #[inline]
    fn degree(&self) -> Real
    {
        2.0
    }

    #[inline]
    fn escaping_period(&self) -> Period
    {
        3
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
            3 =>
            {
                let c2 = c * c;
                let u = (c - 1.).inv();
                let a0 = u * (1. + c + c2 + c2 * c2);
                let a1 = u * (1. + c * (1. - 2. * c2));
                let a2 = -u * (2. + c + c2);
                let [r0, r1, r2] = solve_cubic(a0, a1, a2);
                vec![ONE, -c, r0, r1, r2]
            }
            4 =>
            {
                let c2 = c * c;
                let coeffs = [
                    horner_monic!(c, -1., -4., -7., -2., 13., 29., 29., 15., -3., -8., -6., 0., 0.),
                    horner_monic!(c, -1., -4., -7., -2., 12., 22., 14., -2., -9., -6., -2., 0.),
                    horner!(c, 5., 14., 11., -34., -93., -115., -61., -3., 30., 14., 6., -6.),
                    horner!(c, 5., 14., 13., -20., -60., -60., -12., 28., 26., 6., -4.),
                    horner!(c, -7., -3., 37., 125., 157., 109., -1., -31., -27., 9.),
                    horner!(c, -8., -10., 16., 66., 72., 18., -30., -26., -2.),
                    -horner_monic!(c, 1., 33., 85., 121., 63., 1., -33.),
                    horner!(c, 2., -12., -38., -40., -4., 20., 8.),
                    horner!(c, 9., 35., 44., 21., -14., -7.),
                    horner!(c, 4., 12., 9., -4., -5.),
                    c * (5. * c2 - 7.) - 6.,
                    c2 - 1.,
                    1. - c,
                ];
                solve_polynomial(coeffs)
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
        let param_map: fn(Cplx) -> (Cplx, Cplx);
        let bounds: Bounds;

        match period
        {
            1 =>
            {
                param_map = |t| {
                    let pole = 1.324_717_957_244_75;
                    let u = 1. / t + pole;
                    let u2 = u * u;
                    let u3 = u2 * u;

                    let du = -u2.inv();

                    let num = u3 - u + 1.;
                    let dnum = 3. * u2 - 1.;
                    let den = u3 - u2 - u2 + 3. * u - 1.;
                    let dden = dnum - 4. * u + 4.;

                    (num / den, du*(den * dnum - num * dden) / (den * den))
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

                    // TODO: derivative
                    (s0 * s1 + s1 + (t + 4.), ONE)
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
                param_map = |t| (t, ONE);
                bounds = self.point_grid.bounds.clone();
            }
        };
        let grid = self.point_grid.clone().with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }
}
