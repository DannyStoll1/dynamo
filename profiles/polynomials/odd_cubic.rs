use crate::macros::profile_imports;
use fractal_common::{horner, horner_monic, math_utils::weierstrass_p};
profile_imports!();

#[derive(Clone, Debug)]
pub struct OddCubic
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl OddCubic
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -1.6,
        max_x: 1.6,
        min_y: -1.3,
        max_y: 1.3,
    };
}
impl Default for OddCubic
{
    fractal_impl!();
}

impl ParameterPlane for OddCubic
{
    parameter_plane_impl!();
    default_name!();

    fn encode_escaping_point(
        &self,
        iters: Period,
        z: Cplx,
        _base_param: Cplx,
    ) -> PointInfo<Self::Deriv>
    {
        if z.is_nan()
        {
            return PointInfo::Escaping {
                potential: f64::from(iters) - 1.,
            };
        }

        let u = self.escape_radius().log2();
        let v = z.norm_sqr().log2();
        let residual = (v / u).log(3.);
        let potential = f64::from(iters) - (residual as IterCount);
        PointInfo::Escaping { potential }
    }

    #[inline]
    fn degree(&self) -> f64
    {
        3.0
    }

    #[inline]
    fn map(&self, z: Cplx, c: Cplx) -> Cplx
    {
        2. * (z * z * z / 3. - c * z)
    }

    #[inline]
    fn start_point(&self, _point: Cplx, param: Cplx) -> Cplx
    {
        param.powf(0.5)
    }

    #[inline]
    fn dynamical_derivative(&self, z: Cplx, c: Cplx) -> Cplx
    {
        2. * (z * z - c)
    }

    #[inline]
    fn parameter_derivative(&self, z: Cplx, _c: Cplx) -> Cplx
    {
        -(z + z)
    }

    #[inline]
    fn critical_points_child(&self, param: Cplx) -> ComplexVec
    {
        let sqrt_c = param.sqrt();
        vec![-sqrt_c, sqrt_c]
    }

    #[inline]
    fn cycles_child(&self, c: Self::Param, period: Period) -> Vec<Self::Var>
    {
        match period
        {
            1 =>
            {
                let r1 = (3. * c + 1.5).sqrt();
                vec![ZERO, r1, -r1]
            }
            2 =>
            {
                let r0 = (3. * c - 1.5).sqrt();
                let disc = (c * c - 1.).sqrt();
                let r2 = (1.5 * (c + disc)).sqrt();
                let r4 = (1.5 * (c - disc)).sqrt();
                vec![r0, -r0, r2, -r2, r4, -r4]
            }
            3 =>
            {
                let u = -(c + c);
                let coeffs = [
                    horner_monic!(u, 1., 1.),
                    horner_monic!(u, 1., 2., 2., 2., 1.),
                    horner!(u, 1., 3., 5., 4., 5., 3., 3.),
                    horner!(u, 1., 4., 6., 10., 12., 15., 3., 3.),
                    horner_monic!(u, 1., 4., 10., 19., 31., 16., 19., 1.),
                    horner!(u, 1., 5., 15., 34., 35., 51., 7., 8.),
                    horner!(u, 1., 6., 21., 40., 75., 21., 28.),
                    horner!(u, 1., 7., 25., 65., 35., 56.),
                    horner!(u, 1., 8., 33., 35., 70.),
                    horner!(u, 1., 9., 21., 56.),
                    horner!(u, 1., 7., 28.),
                    horner!(u, 1., 8.),
                    ONE,
                ];
                let squared_sols = solve_polynomial(&coeffs);

                squared_sols
                    .iter()
                    .flat_map(|w| {
                        let z = (1.5 * w).sqrt();
                        [z, -z]
                    })
                    .collect()
            }
            _ => vec![],
        }
    }

    #[inline]
    fn default_julia_bounds(&self, _point: Cplx, _param: Cplx) -> Bounds
    {
        Bounds::centered_square(2.2)
    }
}

impl HasDynamicalCovers for OddCubic
{
    fn marked_cycle_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> Cplx;
        let bounds: Bounds;

        match period
        {
            1 =>
            {
                param_map = |t| ONE_THIRD * t * t - 0.5;
                bounds = Bounds {
                    min_x: -2.5,
                    max_x: 2.5,
                    min_y: -2.5,
                    max_y: 2.5,
                };
            }
            2 =>
            {
                param_map = |t| ONE_THIRD * t * t + 1.;
                bounds = Bounds {
                    min_x: -2.5,
                    max_x: 2.5,
                    min_y: -2.5,
                    max_y: 2.5,
                };
            }
            _ =>
            {
                param_map = |t| t;
                bounds = self.point_grid.bounds.clone();
            }
        };
        let grid = self.point_grid.new_with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }

    fn dynatomic_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> Cplx;
        let bounds: Bounds;

        match period
        {
            1 =>
            {
                param_map = |t| ONE_THIRD * t * t - 0.5;
                bounds = Bounds {
                    min_x: -2.5,
                    max_x: 2.5,
                    min_y: -2.5,
                    max_y: 2.5,
                };
            }
            2 =>
            {
                param_map = |t| {
                    let t2 = t * t;
                    0.75 * t2 + ONE_THIRD / t2
                };
                bounds = Bounds {
                    min_x: -1.5,
                    max_x: 1.5,
                    min_y: -1.5,
                    max_y: 1.5,
                };
            }
            _ =>
            {
                param_map = |c| c;
                bounds = self.point_grid.bounds.clone();
            }
        };
        let grid = self.point_grid.new_with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }
    fn misiurewicz_curve(self, preperiod: Period, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> Cplx;
        let bounds: Bounds;

        match (preperiod, period)
        {
            (1, 1) =>
            {
                param_map = |t| {
                    let t2 = t * t;
                    0.75 * t2 + 0.5 + ONE_THIRD / t2
                };
                bounds = Bounds {
                    min_x: -2.5,
                    max_x: 2.5,
                    min_y: -2.5,
                    max_y: 2.5,
                };
            }
            (1, 2) =>
            {
                param_map = |t| {
                    let g2 = ONE_NINTH.into();
                    let g3 = ZERO;
                    let (mut x, mut y) = weierstrass_p(g2, g3, t, 0.01);

                    x *= 3.;
                    y *= 6.;

                    x = x.inv();
                    y *= x;

                    let y2 = y * y;
                    let x2 = x * x;
                    let x4 = x2 * x2;

                    let u0 = 3. / y;
                    let u2 = (x2 * x + 3. * y2).inv();
                    let u3 = x4 * u0 * u2 - x * u0;
                    let u4 = 3. / y2;
                    let u5 = x4 * x * u2 * u4 - x2 * u4;

                    let u3_2 = u3 * u3;
                    let u5_2 = u5 * u5;
                    let v = u3_2 * u3_2 / (u5 * u5_2) + 3. * u3_2 / u5_2;

                    v.inv()
                };
                bounds = Bounds {
                    min_x: -2.5,
                    max_x: 2.5,
                    min_y: -2.5,
                    max_y: 2.5,
                };
            }
            (_, _) =>
            {
                param_map = |c| c;
                bounds = self.point_grid.bounds.clone();
            }
        };
        let grid = self.point_grid.new_with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }
}
