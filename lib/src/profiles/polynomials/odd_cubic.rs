use crate::macros::*;
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
        z: ComplexNum,
        _base_param: ComplexNum,
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
    fn map(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        2. * (z * z * z / 3. - c * z)
    }

    #[inline]
    fn start_point(&self, _point: ComplexNum, param: ComplexNum) -> ComplexNum
    {
        param.powf(0.5)
    }

    #[inline]
    fn dynamical_derivative(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        2. * (z * z - c)
    }

    #[inline]
    fn parameter_derivative(&self, z: ComplexNum, _c: ComplexNum) -> ComplexNum
    {
        -(z + z)
    }

    #[inline]
    fn critical_points_child(&self, param: ComplexNum) -> ComplexVec
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
            _ => vec![],
        }
    }

    #[inline]
    fn default_julia_bounds(&self, _point: ComplexNum, _param: ComplexNum) -> Bounds
    {
        Bounds::centered_square(2.2)
    }
}
