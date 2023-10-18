use crate::macros::{horner, horner_monic, profile_imports};
use fractal_common::math_utils::weierstrass_p;
profile_imports!();

// Quadratic rational maps with a critical 4-cycle: 0 => ∞ -> 1 -> c -> 0
#[derive(Clone, Debug)]
pub struct QuadRatPer4
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl QuadRatPer4
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -1.,
        max_x: 0.2,
        min_y: -0.5,
        max_y: 0.5,
    };
}
impl Default for QuadRatPer4
{
    fractal_impl!();
}

impl ParameterPlane for QuadRatPer4
{
    parameter_plane_impl!();
    default_name!();
    default_bounds!();

    fn description(&self) -> String
    {
        "The moduli space of quadratic rational maps with a critical 4-cycle, \
            parameterized as $f_c(z) = (z-c)(z(c-1)-2c+1)/(z^2(c-1))$. \
            In these coordinates, 0 -> ∞ -> 1 -> c is the critical 4-cycle. \
            The plane is colored according to the \
            activity of the free critical point 0."
            .to_owned()
    }

    fn encode_escaping_point(
        &self,
        iters: Period,
        z: Cplx,
        c: Cplx,
    ) -> PointInfo<Self::Var, Self::Deriv>
    {
        {
            if z.is_nan()
            {
                return PointInfo::Escaping {
                    potential: f64::from(iters) - 4.,
                };
            }

            let u = self.escape_radius().log2();
            let v = z.norm_sqr().log2();
            let c2 = c * c;
            let two_c = c + c;
            let c12 = c2 - two_c + 1.; // (c-1)^2

            let d0 = c2 + c - 1.; // c^2 + c - 1
            let d1 = d0 - two_c - two_c + 2.; // c^2 - 3c + 1
            let d2 = c2 + c2 + d1; // 3c^2 - 3c + 1

            // (2*a - 1) * (a - 1)^5 * a^5 * (a^2 - 3*a + 1)^-2 * (3a^2 - 3a + 1)^-2 * (a^2 + a - 1)^-2
            let q_numer = (two_c - 1.) * c2 * c2 * c * c12 * c12 * (c - 1.);
            let q_denom = d0 * d0 + d1 * d1 + d2 * d2;
            let q = (q_numer / q_denom).norm().log2();
            let residual = ((u + q) / (v + q)).log2();
            let potential = (residual as IterCount).mul_add(4., f64::from(iters));
            PointInfo::Escaping { potential }
        }
    }

    #[inline]
    fn param_map(&self, t: Cplx) -> Cplx
    {
        let pole = 2.618_033_988_749_89;
        1. / t + pole
    }

    #[inline]
    fn param_map_d(&self, t: Cplx) -> (Cplx, Cplx)
    {
        let pole = 2.618_033_988_749_89;
        let u = t.inv();
        (u + pole, -u * u)
    }

    #[inline]
    fn start_point(&self, _point: Cplx, c: Cplx) -> Cplx
    {
        (c + c) * (c + c - 1.) / (c * (c + 1.) - 1.)
    }

    #[inline]
    fn start_point_d(&self, _point: Cplx, c: Cplx) -> (Cplx, Cplx, Cplx)
    {
        let c2 = c * c;
        let denom = (c2 + c - 1.).inv();
        (
            2. * (2. * c2 - c) * denom,
            ZERO,
            (6. * c2 - 8. * c + 2.) * denom * denom,
        )
    }

    #[inline]
    fn map(&self, z: Cplx, c: Cplx) -> Cplx
    {
        (c * z - c - c - z + 1.) * (z - c) / (z * z * (c - 1.))
    }

    #[inline]
    fn map_and_multiplier(&self, z: Cplx, c: Cplx) -> (Cplx, Cplx)
    {
        let c2 = c * c;
        let c_minus_1 = c - 1.;
        let u = (c_minus_1 * z * z).inv();
        let two_c = c + c;

        (
            (z - c) * (c * z - z - two_c + 1.) * u,
            (c2 + c_minus_1 - (4. * c2 - two_c) / z) * u,
        )
    }

    #[inline]
    fn dynamical_derivative(&self, z: Cplx, c: Cplx) -> Cplx
    {
        let c2 = c * c;
        (4. * c2 - (c2 + c - 1.) * z - c - c) / ((c - 1.) * z * z * z)
    }

    #[inline]
    fn parameter_derivative(&self, z: Cplx, c: Cplx) -> Cplx
    {
        let v = (c - 1.) * z;
        (1. + (2. - c) * c * (z - 2.)) / (v * v)
    }

    #[inline]
    fn gradient(&self, z: Self::Var, c: Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        let v = c - 1.;
        let c2 = c * c;
        let z2 = z * z;
        let u = (v * z2).inv();
        let two_c = c + c;
        (
            (z - c) * (c * z - z - two_c + 1.) * u,
            (c2 + v - (4. * c2 - two_c) / z) * u,
            (1. + (two_c - c2) * (z - 2.)) * u / v,
        )
    }

    #[inline]
    fn degree_real(&self) -> Real
    {
        2.0
    }

    #[inline]
    fn degree(&self) -> AngleNum
    {
        2
    }

    #[inline]
    fn escaping_period(&self) -> Period
    {
        4
    }

    #[inline]
    fn critical_points_child(&self, c: Self::Param) -> Vec<Self::Var>
    {
        let c2 = c * c;
        vec![ZERO, 2. * (2. * c2 - c) / (c2 + c - 1.)]
    }

    #[inline]
    fn angle_map_large_param(&self, angle: RationalAngle) -> RationalAngle
    {
        angle + RationalAngle::ONE_HALF
    }

    fn cycles_child(&self, c: Cplx, period: Period) -> ComplexVec
    {
        match period
        {
            1 =>
            {
                let x0 = c - 1.;
                let x1 = x0.inv();
                let x2 = c * c;
                let x3 = x1 * (x0 + x2);
                let x4 = x1 * (c - (x2 + x2));
                let x5 = 1. - 3. * x3;
                let s = -4. * x5.powf(3.);
                let t = 9. * x3 + 27. * x4 - 2.;
                let u = (s + t * t).sqrt();
                let x6 = (0.5 * (t + u)).powf(ONE_THIRD);
                let x7 = x6 / 3.;
                let x8 = x5 / (3. * x6);
                let r1 = -x7 * OMEGA_BAR - x8 * OMEGA + ONE_THIRD;
                let r2 = -x7 * OMEGA - x8 * OMEGA_BAR + ONE_THIRD;
                vec![-x7 - x8 + ONE_THIRD, r1, r2]
            }
            2 =>
            {
                let c2 = c * c;
                let x0 = c2 * 3.;
                let denom = 0.5 / (c - 1.);
                let disc = (x0 * x0 - c * (8. * c2 - 6. * c + 4.) + 1.).sqrt();
                vec![denom * (x0 + disc - 1.), denom * (x0 - disc - 1.)]
            }
            3 =>
            {
                let c2 = c * c;
                let coeffs = [
                    c2 * c * horner!(c, 1., -7., 18., -20., 8.),
                    c2 * horner!(c, -4., 25., -54., 41., 4., -12.),
                    c * horner!(c, 5., -24., 26., 33., -72., 23., 10.),
                    horner!(c, -2., 2., 29., -83., 71., -4., -10., -5.),
                    horner_monic!(c, 4., -17., 19., 11., -36., 23., -4.),
                    horner!(c, -2., 9., -16., 14., -4., -3., 2.),
                    c * horner_monic!(c, 1., -4., 6., -4.),
                ];
                solve_polynomial(coeffs)
            }
            4 =>
            {
                let c2 = c * c;
                let c3 = c * c2;
                let c4 = c2 * c2;
                let coeffs = [
                    c3 * c4 * horner!(c, -1., 12., -61., 170., -280., 272., -144., 32.),
                    c4 * horner!(
                        c, 1., -15., 103., -419., 1089., -1817., 1835., -896., -72., 272., -80.
                    ),
                    c3 * horner!(
                        c, -4., 57., -360., 1300., -2868., 3747., -2293., -527., 1686., -732.,
                        -104., 96.
                    ),
                    c2 * horner!(
                        c, 6., -79., 445., -1345., 2127., -841., -3011., 5721., -3916., 382., 726.,
                        -144., -72.
                    ),
                    c * horner!(
                        c, -4., 45., -191., 261., 737., -3856., 7348., -6869., 2028., 1633.,
                        -1223., -90., 151., 34.
                    ),
                    horner!(
                        c, 1., -6., -21., 322., -1375., 2999., -3272., 469., 3191., -3641., 1294.,
                        192., -105., -41., -9.
                    ),
                    horner_monic!(
                        c, -2., 24., -117., 264., -90., -1028., 2817., -3546., 2169., -238., -392.,
                        115., 26., -3.
                    ),
                    horner!(
                        c, 1., -14., 87., -312., 701., -987., 774., -121., -362., 329., -98., 1.,
                        -1., 2.
                    ),
                    c2 * c3 * horner_monic!(c, -1., 7., -21., 35., -35., 21., -7.),
                ];
                let mut rs = solve_polynomial(coeffs);
                rs.extend([ONE, c, ZERO]);
                rs
            }
            _ => vec![],
        }
    }

    #[inline]
    fn default_julia_bounds(&self, _point: Cplx, _param: Cplx) -> Bounds
    {
        Bounds::square(4., (2.).into())
    }
}

impl HasDynamicalCovers for QuadRatPer4
{
    fn marked_cycle_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> (Cplx, Cplx);
        let grid: PointGrid;
        let bounds: Bounds;

        match period
        {
            3 =>
            {
                param_map = |c| {
                    // cbrt(12)
                    let alpha = Cplx::new(2.289_428_485_106_66, 0.);
                    let g2 = alpha;
                    let g3 = Cplx::new(-19. / 12., 0.);

                    let (p, _dp) = weierstrass_p(g2, g3, c, 0.01);
                    let x = (alpha * p + 1.) / 3.;
                    // let y = (dp - 1.5) / x;

                    // TODO: derivative
                    (x / (x + 1.), ONE)
                    // let xx = x + 1.;
                    // let yy = y - 3. * x - 3.;
                    //
                    // let x0 = yy / x;
                    // let _s1 = x0 * xx / x;

                    // x / xx
                };
                bounds = Bounds {
                    min_x: -3.6,
                    max_x: 3.6,
                    min_y: -2.4,
                    max_y: 2.4,
                };
                grid = self.point_grid.clone().with_same_height(bounds);
            }
            _ =>
            {
                param_map = |t| (t, ONE);
                grid = self.point_grid.clone();
            }
        };
        CoveringMap::new(self, param_map, grid)
    }
}
