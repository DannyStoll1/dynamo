use crate::macros::{degree_impl, horner, profile_imports};
profile_imports!();

// Quadratic rational maps of the form z -> c(z+1/z)
#[derive(Clone, Debug)]
pub struct QuadRatSymmetryLocus
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl QuadRatSymmetryLocus
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -1.1,
        max_x: 1.1,
        min_y: -1.1,
        max_y: 1.1,
    };
}
impl Default for QuadRatSymmetryLocus
{
    fractal_impl!();
}

impl ParameterPlane for QuadRatSymmetryLocus
{
    parameter_plane_impl!();
    default_name!();
    default_bounds!();

    #[inline]
    fn map(&self, z: Cplx, c: Cplx) -> Cplx
    {
        c * (z + 1. / z)
    }

    #[inline]
    fn map_and_multiplier(&self, z: Cplx, c: Cplx) -> (Cplx, Cplx)
    {
        let u = z.inv();
        (c * (z + u), c * (1. - u * u))
    }

    fn start_point(&self, _point: Cplx, _c: Self::Param) -> Self::Var
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
        z + 1. / z
    }

    #[inline]
    fn critical_points_child(&self, _param: Cplx) -> ComplexVec
    {
        vec![(-1.).into(), (1.).into()]
    }

    fn cycles_child(&self, c: Self::Param, period: Period) -> Vec<Self::Var>
    {
        match period
        {
            1 =>
            {
                let disc = (c / (1. - c)).sqrt();
                vec![disc, -disc]
            }
            2 =>
            {
                let disc = (-c / (1. + c)).sqrt();
                vec![disc, -disc]
            }
            3 =>
            {
                if c.norm_sqr() < 1e-20
                {
                    return vec![];
                }
                let c2 = c * c;
                let u = 1. + c + c2;
                let v = (c2 * u).inv();
                let a2 = v * horner!(c, 1., -1., 2., 1., 3.);
                let a4 = v * (1. + c2 * (3. + 2. * c + 3. * c2));
                let squared_sols = solve_cubic(c2 / u, a2, a4);
                squared_sols
                    .iter()
                    .flat_map(|z| {
                        let sqrt_z = z.sqrt();
                        [sqrt_z, -sqrt_z]
                    })
                    .collect()
            }
            4 =>
            {
                if c.norm_sqr() < 1e-20
                {
                    return vec![];
                }
                let c2 = c * c;
                let c4 = c2 * c2;
                let c_neg6 = (c2 * c4).inv();
                let u = c2 * horner!(c2, 1., 2., 4.);
                let v = horner!(c2, 1., 3., 4., 6.);

                let mut squared_sols =
                    solve_quartic(ONE, (u + 1.) * c_neg6, v * c_neg6, u * c_neg6).to_vec();
                squared_sols.extend(solve_quadratic(c2 / (c2 + 1.), TWO));
                squared_sols
                    .iter()
                    .flat_map(|z| {
                        let sqrt_z = z.sqrt();
                        [sqrt_z, -sqrt_z]
                    })
                    .collect()
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

impl InfinityFirstReturnMap for QuadRatSymmetryLocus
{
    degree_impl!(1, 1);
}

impl EscapeEncoding for QuadRatSymmetryLocus
{
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
}

impl ExternalRays for QuadRatSymmetryLocus {}
