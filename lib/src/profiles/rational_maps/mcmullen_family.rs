use crate::{macros::*, math_utils::solve_quadratic};
profile_imports!();

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct McMullenFamily
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl Default for McMullenFamily
{
    fractal_impl!(-3200., 4200., -3500., 3500.);
}

impl ParameterPlane for McMullenFamily
{
    parameter_plane_impl!();
    default_name!();

    fn start_point(&self, _point: ComplexNum, c: Self::Param) -> Self::Var {
        (1.5/c).powf(0.2)
    }

    fn map(&self, z: Self::Var, c: Self::Param) -> Self::Var
    {
        let z2 = z * z;
        z2 + (c * z * z2).inv()
    }

    fn map_and_multiplier(&self, z: Self::Var, c: Self::Param) -> (Self::Var, Self::Deriv)
    {
        let z2 = z * z;
        let denom = c * z2;
        (
            z2 + (denom * z).inv(),
            z + z - (ONE_THIRD * denom * z2).inv(),
        )
    }

    fn dynamical_derivative(&self, z: Self::Var, c: Self::Param) -> Self::Deriv
    {
        let z2 = z * z;
        z + z - 3. / (c * z2 * z2)
    }

    fn parameter_derivative(&self, z: Self::Var, c: Self::Param) -> Self::Deriv
    {
        -(c*c*z.powi(3)).inv()
    }

    fn critical_points_child(&self, c: Self::Param) -> Vec<Self::Var>
    {
        let z0 = (1.5/c).powf(0.2);
        vec![z0, z0*ZETA_5_1, z0*ZETA_5_2, z0*ZETA_5_1.conj(), z0*ZETA_5_2.conj()]
    }

    fn default_julia_bounds(&self, _point: ComplexNum, _param: Self::Param) -> Bounds {
        Bounds { min_x: -1.15, max_x: 1.15, min_y: -1.15, max_y: 1.15 }
    }

    fn default_selection(&self) -> ComplexNum {
        ONE
    }
}
