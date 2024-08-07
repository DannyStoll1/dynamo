use crate::macros::{degree_impl, horner, profile_imports};
use dynamo_common::math_utils::weierstrass_p;
profile_imports!();

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct QuadRatPreper21
{
    point_grid: PointGrid,
    compute_mode: ComputeMode,
    max_iter: IterCount,
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

impl DynamicalFamily for QuadRatPreper21
{
    parameter_plane_impl!();
    default_name!();

    #[inline]
    fn map(&self, z: Cplx, c: &Cplx) -> Cplx
    {
        c * (z + 2. + z.inv())
    }

    #[inline]
    fn map_and_multiplier(&self, z: Cplx, c: &Cplx) -> (Cplx, Cplx)
    {
        let u = z.inv();
        (c * (z + 2. + u), c * (1. - u.powi(2)))
    }

    #[inline]
    fn gradient(&self, z: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        let u = z.inv();
        let v = z + 2. + u;
        (c * v, c * (1. - u.powi(2)), v)
    }

    fn start_point(&self, _point: Cplx, _c: &Cplx) -> Cplx
    {
        (1.).into()
    }
}

impl FamilyDefaults for QuadRatPreper21
{
    default_bounds!();
}

impl HasJulia for QuadRatPreper21
{
    fn default_bounds_child(&self, _point: Cplx, _param: &Cplx) -> Bounds
    {
        Bounds::centered_square(4.)
    }
}

impl MarkedPoints for QuadRatPreper21
{
    #[inline]
    fn critical_points_child(&self, _param: &Cplx) -> ComplexVec
    {
        vec![(-1.).into(), (1.).into()]
    }

    fn cycles_child(&self, c: &Self::Param, period: Period) -> Vec<Self::Var>
    {
        match period {
            1 => {
                let u = c / (c - 1.);
                solve_quadratic(u, u + u).to_vec()
            }
            2 => solve_quadratic(c / (c + 1.), TWO).to_vec(),
            3 => {
                if c.norm_sqr() < 1e-20 {
                    return vec![];
                }

                let c2 = c.powi(2);
                let coeffs = [
                    c2 * c2,
                    c2 * horner!(c, 2., 4., 6.),
                    horner!(c, 1., 3., 14., 17., 15.),
                    horner!(c, 2., 10., 28., 28., 20.),
                    horner!(c, 1., 8., 23., 22., 15.),
                    c * horner!(c, 2., 8., 8., 6.),
                    c2 * (1. + c + c2),
                ];
                solve_polynomial(coeffs)
            }
            4 => {
                if c.norm_sqr() < 1e-20 {
                    return vec![];
                }

                let c2 = c.powi(2);
                let c3 = c * c2;
                let c5 = c2 * c3;
                let coeffs0 = [
                    c5,
                    c2 * horner!(c, 1., 3., 4., 8.),
                    c * horner!(c, 4., 13., 25., 24., 28.),
                    horner!(c, 2., 22., 50., 75., 60., 56.),
                    horner!(c, 6., 42., 82., 111., 80., 70.),
                    horner!(c, 5., 33., 65., 89., 60., 56.),
                    horner!(c, 1., 11., 25., 39., 24., 28.),
                    c * horner!(c, 1., 4., 9., 4., 8.),
                    c3 + c5,
                ];
                let coeffs1 = [
                    c3,
                    horner!(c, 1., 1., 2., 4.),
                    horner!(c, 1., 3., 4., 6.),
                    c * horner!(c, 1., 2., 4.),
                    c3,
                ];

                let mut sol0 = solve_polynomial(coeffs0);
                let sol1 = solve_polynomial(coeffs1);

                sol0.extend(sol1);
                sol0
            }
            _ => vec![],
        }
    }
}

impl InfinityFirstReturnMap for QuadRatPreper21
{
    degree_impl!(1, 1);
}

impl EscapeEncoding for QuadRatPreper21
{
    fn encode_escaping_point(
        &self,
        iters: IterCount,
        z: Cplx,
        base_param: &Cplx,
    ) -> PointInfo<Self::Deriv>
    {
        if z.is_nan() {
            return PointInfo::Escaping {
                potential: (iters - 1) as IterCountSmooth,
                phase: None,
            };
        }

        let expansion_rate = base_param.norm_sqr();
        let u = self.escape_radius().log(expansion_rate);
        let v = z.norm_sqr().log(expansion_rate);
        let residual = u - v;
        let potential = (iters as IterCountSmooth) + (residual as IterCountSmooth);
        PointInfo::Escaping {
            potential,
            phase: None,
        }
    }
}

impl HasDynamicalCovers for QuadRatPreper21
{
    fn marked_cycle_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> (Cplx, Cplx);
        let bounds: Bounds;

        match period {
            3 => {
                param_map = |c| {
                    let u = c + 4.;
                    (0.25 * u.powi(2), 0.5 * u)
                };
                bounds = Bounds {
                    min_x: -6.2,
                    max_x: -1.8,
                    min_y: -2.2,
                    max_y: 2.2,
                };
            }
            4 => {
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

                    // derivative unimplemented
                    (y / x, ONE)
                };
                bounds = Bounds {
                    min_x: -16.,
                    max_x: 16.,
                    min_y: -12.,
                    max_y: 12.,
                };
            }
            _ => return CoveringMap::from(self),
        }
        CoveringMap::new(self, param_map).with_orig_bounds(bounds)
    }
}

impl ExternalRays for QuadRatPreper21 {}
