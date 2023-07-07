use crate::macros::profile_imports;
profile_imports!();

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Tricorne
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl Default for Tricorne
{
    fractal_impl!(-2.4, 1.5, -2.2, 2.2);
}

impl ParameterPlane for Tricorne
{
    parameter_plane_impl!();
    default_name!();

    #[inline]
    fn map(&self, z: Self::Var, c: Self::Param) -> Self::Var
    {
        (z * z).conj() + c
    }

    #[inline]
    fn start_point(&self, _point: Cplx, c: Self::Param) -> Self::Var
    {
        ZERO
    }

    #[inline]
    fn dynamical_derivative(&self, z: Self::Var, _c: Self::Param) -> Self::Deriv
    {
        (z + z).conj()
    }

    #[inline]
    fn parameter_derivative(&self, _z: Self::Var, _c: Self::Param) -> Self::Deriv
    {
        ONE
    }

    fn critical_points_child(&self, _param: Self::Param) -> Vec<Self::Var>
    {
        vec![ZERO]
    }
}
