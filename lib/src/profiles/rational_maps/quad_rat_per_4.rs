use crate::macros::profile_imports;
use crate::math_utils::weierstrass_p;
profile_imports!();

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

    fn encode_escaping_point(&self, iters: Period, z: Cplx, c: Cplx) -> PointInfo<Self::Deriv>
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
    fn param_map(&self, c: Cplx) -> Cplx
    {
        let pole = 2.618_033_988_749_89;
        1. / c + pole
    }

    #[inline]
    fn start_point(&self, _point: Cplx, c: Cplx) -> Cplx
    {
        (c + c) * (c + c - 1.) / (c * (c + 1.) - 1.)
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
            ((4. * c2 - two_c) / z - (c2 + c_minus_1)) * u,
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
        let v = c - 1.;
        (2. - c) * c * (z - 2.) / (v * v * z * z)
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
            (4. * c2 - (c2 + v) * z - two_c) * u / z,
            (two_c - c2) * (z - 2.) * u / v,
        )
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
        let param_map: fn(Cplx) -> Cplx;
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

                    x / (x + 1.)
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
                param_map = |c| c;
                grid = self.point_grid.clone();
            }
        };
        CoveringMap::new(self, param_map, grid)
    }
}
