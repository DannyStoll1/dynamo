use crate::macros::*;
profile_imports!();

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NewtonCubic
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl NewtonCubic
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -2.5,
        max_x: 2.5,
        min_y: -2.5,
        max_y: 2.5,
    };
}
impl Default for NewtonCubic
{
    fractal_impl!();
}

impl ParameterPlane for NewtonCubic
{
    parameter_plane_impl!();
    default_name!();
    default_bounds!();

    // f(z) = z^3 + cz - 1
    // f'(z) = 3z^2 + c
    #[inline]
    fn map(&self, z: Self::Var, c: Self::Param) -> Self::Var
    {
        let z2 = z * z;
        (2. * z * z2 + 1.) / (3. * z2 + c)
    }

    fn dynamical_derivative(&self, z: Self::Var, c: Self::Param) -> Self::Deriv
    {
        let z2 = z * z;
        let df = 3. * z2 + c;
        6. * (z * (z2 + c) - 1.) * z / (df * df)
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, c: Self::Param) -> (Self::Var, Self::Deriv)
    {
        let z2 = z * z;
        let f = z * (z2 + c) - 1.;
        let df = 3. * z2 + c;
        let u = f / df;
        (z - u, 6. * z * u / df)
    }

    #[inline]
    fn start_point(&self, _point: Cplx, _c: Self::Param) -> Self::Var
    {
        ZERO
    }

    fn parameter_derivative(&self, z: Self::Var, c: Self::Param) -> Self::Deriv
    {
        let z2 = z * z;
        let df = 3. * z2 + c;
        (-1. - 2. * z * z2) / (df * df)
    }

    fn critical_points_child(&self, c: Self::Param) -> Vec<Self::Var>
    {
        let [r0, r1, r2] = solve_cubic(-ONE, c, ZERO);
        vec![r0, r1, r2, ZERO]
    }

    fn cycles_child(&self, c: Self::Param, period: Period) -> Vec<Self::Var>
    {
        match period
        {
            1 => solve_cubic(-ONE, c, ZERO).to_vec(),
            _ => vec![],
        }
    }

    fn get_marked_points(&self, c: Self::Param) -> Vec<(Cplx, PointClassId)>
    {
        solve_cubic(-ONE, c, ZERO)
            .into_iter()
            .enumerate()
            .map(|(i, z)| (z, PointClassId::from(i)))
            .collect()
    }
}
