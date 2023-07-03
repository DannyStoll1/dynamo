use crate::{
    macros::{basic_escape_encoding, profile_imports},
    math_utils::solve_quadratic,
};
profile_imports!();

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct McMullenFamily<const M: i32, const N: i32>
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl<const M: i32, const N: i32> McMullenFamily<M, N>
{
    const M_FLOAT: Real = M as Real;
    const N_FLOAT: Real = N as Real;
    const M_MINUS_1: i32 = M - 1;
    const N_MINUS_1: i32 = N - 1;
    const M_PLUS_N_INV: Real = 1. / (Self::M_FLOAT + Self::N_FLOAT);
    const DEFAULT_BOUNDS: Bounds = Bounds::centered_square(80. / (Self::M_FLOAT - 1.8));
}

impl<const M: i32, const N: i32> Default for McMullenFamily<M, N>
{
    fractal_impl!();
}

impl<const M: i32, const N: i32> ParameterPlane for McMullenFamily<M, N>
{
    parameter_plane_impl!();
    basic_escape_encoding!(Self::M_FLOAT, 1.);

    fn start_point(&self, _point: Cplx, c: Self::Param) -> Self::Var
    {
        let z0 = Self::N_FLOAT / (c * Self::M_FLOAT);
        z0.powf(Self::M_PLUS_N_INV)
    }

    fn map(&self, z: Self::Var, c: Self::Param) -> Self::Var
    {
        z.powi(M) + (c * z.powi(N)).inv()
        // let z2 = z * z;
        // z2 + (c * z * z2).inv()
    }

    fn map_and_multiplier(&self, z: Self::Var, c: Self::Param) -> (Self::Var, Self::Deriv)
    {
        let zm1 = z.powi(Self::M_MINUS_1);
        let czn_inv = (c * z.powi(N)).inv();
        (
            zm1 * z + czn_inv,
            Self::M_FLOAT * zm1 - Self::N_FLOAT * czn_inv / z,
        )
    }

    fn dynamical_derivative(&self, z: Self::Var, c: Self::Param) -> Self::Deriv
    {
        Self::M_FLOAT * z.powi(Self::M_MINUS_1) - Self::N_FLOAT / (c * z.powi(N + 1))
    }

    fn parameter_derivative(&self, z: Self::Var, c: Self::Param) -> Self::Deriv
    {
        -(c * c * z.powi(N)).inv()
    }

    fn critical_points_child(&self, c: Self::Param) -> Vec<Self::Var>
    {
        let w0 = Self::N_FLOAT / (c * Self::M_FLOAT);
        let z0 = w0.powf(Self::M_PLUS_N_INV);
        (0..(M + N))
            .map(|k| (TAUI * f64::from(k) * Self::M_PLUS_N_INV).exp() * z0)
            .collect()
    }

    fn default_julia_bounds(&self, _point: Cplx, _param: Self::Param) -> Bounds
    {
        Bounds {
            min_x: -1.15,
            max_x: 1.15,
            min_y: -1.15,
            max_y: 1.15,
        }
    }

    fn default_selection(&self) -> Cplx
    {
        ONE
    }

    fn name(&self) -> String
    {
        format!("McMullen Family ({M}, {N})")
    }
}
