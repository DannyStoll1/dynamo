use crate::macros::profile_imports;
use crate::math_utils::{weierstrass_p, solve_quadratic};
profile_imports!();

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct QuadRatPreper21
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl QuadRatPreper21
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -1.1,
        max_x: 1.1,
        min_y: -1.1,
        max_y: 1.1,
    };
}
impl Default for QuadRatPreper21
{
    fractal_impl!();
}

impl ParameterPlane for QuadRatPreper21
{
    parameter_plane_impl!();
    default_name!();

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
                potential: f64::from(iters) - 2.,
            };
        }

        let expansion_rate = base_param.norm_sqr();
        let u = self.escape_radius().log(expansion_rate);
        let v = z.norm_sqr().log(expansion_rate);
        let residual = u - v;
        let potential = IterCount::from(iters) + (residual as IterCount);
        PointInfo::Escaping { potential }
    }

    #[inline]
    fn map(&self, z: Cplx, c: Cplx) -> Cplx
    {
        c * (z + 2. + 1. / z)
    }

    #[inline]
    fn map_and_multiplier(&self, z: Cplx, c: Cplx) -> (Cplx, Cplx)
    {
        let u = z.inv();
        (c * (z + 2. + u), c * (1. - u * u))
    }

    fn start_point(&self, _point: Cplx, _c: Cplx) -> Cplx
    {
        (1.).into()
    }

    #[inline]
    fn dynamical_derivative(&self, z: Cplx, c: Cplx) -> Cplx
    {
        c * (1. - 1. / (z * z))
    }

    #[inline]
    fn parameter_derivative(&self, z: Cplx, _c: Cplx) -> Cplx
    {
        z + 1. / z + 2.
    }

    #[inline]
    fn critical_points_child(&self, _param: Cplx) -> ComplexVec
    {
        vec![(-1.).into(), (1.).into()]
    }

    fn cycles_child(&self, c: Self::Param, period: Period) -> Vec<Self::Var> {
        match period {
            1 => {
                let u = c/(c-1.);
                solve_quadratic(u, u+u).to_vec()
            },
            2 => solve_quadratic(c/(c+1.), TWO).to_vec(),
            _ => vec![],
        }
    }

    #[inline]
    fn default_julia_bounds(&self, _point: Cplx, _param: Cplx) -> Bounds
    {
        Bounds::centered_square(4.)
    }
}

impl HasDynamicalCovers for QuadRatPreper21
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
                    let u = c + 4.;
                    u * u / 4.
                };
                bounds = Bounds {
                    min_x: -6.2,
                    max_x: -1.8,
                    min_y: -2.2,
                    max_y: 2.2,
                };
                grid = self.point_grid.clone().with_same_height(bounds);
            }
            4 =>
            {
                param_map = |c| {
                    let g2 = Cplx::new(-1. / 96., 0.);
                    let g3 = Cplx::new(-13. / 55296., 0.);
                    let (p, dp) = weierstrass_p(g2, g3, c, 0.01);

                    let x = p + 1.0 / 24.0;
                    let root_neg2_over_16 = Cplx::new(0., 0.088_388_347_648_318_4);
                    let mut y = dp * root_neg2_over_16;
                    // e4 = 8*x^3 - x^2 + 256*y^2 + x/16 - 1.0/1024.0

                    y += (1. - 32. * x) / 512.;
                    // e3 = 8*x^3 + 32*x*y + 256*y^2 - y

                    y /= x;
                    // e2 = 256*x*y^2 + 8*x^2 + 32*x*y - y

                    y / x
                };
                bounds = Bounds {
                    min_x: -16.,
                    max_x: 16.,
                    min_y: -12.,
                    max_y: 12.,
                };
                grid = self.point_grid.clone().with_same_height(bounds);
            }
            _ =>
            {
                param_map = |c| c;
                grid = self.point_grid.clone();
            }
        }

        CoveringMap::new(self, param_map, grid)
    }
}
