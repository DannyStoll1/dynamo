use crate::macros::profile_imports;
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
            _ => vec![],
        }
    }

    #[inline]
    fn default_julia_bounds(&self, _point: Cplx, _param: Cplx) -> Bounds
    {
        Bounds::centered_square(4.)
    }
}
