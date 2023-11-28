use crate::macros::{degree_impl, profile_imports};
profile_imports!();

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct McMullenFamily<const M: i32, const N: i32>
{
    point_grid: PointGrid,
    compute_mode: ComputeMode,
    max_iter: IterCount,
}

impl<const M: i32, const N: i32> McMullenFamily<M, N>
{
    const M_FLOAT: Real = M as Real;
    const N_FLOAT: Real = N as Real;
    const M_MINUS_1: i32 = M - 1;
    const M_PLUS_N_INV: Real = 1. / (Self::M_FLOAT + Self::N_FLOAT);
    const DEFAULT_BOUNDS: Bounds = Bounds::centered_square(80. / (Self::M_FLOAT - 1.8));
}

impl<const M: i32, const N: i32> Default for McMullenFamily<M, N>
{
    fractal_impl!();
}

impl<const M: i32, const N: i32> DynamicalFamily for McMullenFamily<M, N>
{
    parameter_plane_impl!();

    fn start_point(&self, _point: Cplx, c: &Self::Param) -> Self::Var
    {
        let z0 = Self::N_FLOAT / (c * Self::M_FLOAT);
        z0.powf(Self::M_PLUS_N_INV)
    }

    fn map(&self, z: Self::Var, c: &Self::Param) -> Self::Var
    {
        z.powi(M) + (c * z.powi(N)).inv()
    }

    fn map_and_multiplier(&self, z: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv)
    {
        let zm1 = z.powi(Self::M_MINUS_1);
        let czn_inv = (c * z.powi(N)).inv();
        (
            zm1 * z + czn_inv,
            Self::M_FLOAT * zm1 - Self::N_FLOAT * czn_inv / z,
        )
    }

    fn gradient(&self, z: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        let zm1 = z.powi(Self::M_MINUS_1);
        let czn_inv = (c * z.powi(N)).inv();
        (
            zm1 * z + czn_inv,
            Self::M_FLOAT * zm1 - Self::N_FLOAT * czn_inv / z,
            -czn_inv / c,
        )
    }

    fn name(&self) -> String
    {
        format!("McMullen Family ({M}, {N})")
    }
}

impl<const M: i32, const N: i32> FamilyDefaults for McMullenFamily<M, N>
{
    default_bounds!();

    fn default_selection(&self) -> Cplx
    {
        ONE
    }
}

impl<const M: i32, const N: i32> HasJulia for McMullenFamily<M, N>
{
    fn default_bounds_child(&self, _point: Cplx, _param: &Self::Param) -> Bounds
    {
        Bounds {
            min_x: -1.15,
            max_x: 1.15,
            min_y: -1.15,
            max_y: 1.15,
        }
    }
}

impl<const M: i32, const N: i32> MarkedPoints for McMullenFamily<M, N>
{
    fn cycles_child(&self, c: &Self::Param, period: Period) -> Vec<Self::Var>
    {
        match period {
            1 => {
                let mut coeffs = vec![ZERO; (M + N + 1).try_into().unwrap_or(3)];
                coeffs[usize::try_from(M + N).unwrap_or(2)] = *c;
                coeffs[usize::try_from(N + 1).unwrap_or(1)] = -c;
                coeffs[0] = ONE;
                solve_polynomial(coeffs)
            }
            _ => vec![],
        }
    }

    fn critical_points_child(&self, c: &Self::Param) -> Vec<Self::Var>
    {
        let w0 = Self::N_FLOAT / (c * Self::M_FLOAT);
        let z0 = w0.powf(Self::M_PLUS_N_INV);
        (0..(M + N))
            .map(|k| (TAUI * f64::from(k) * Self::M_PLUS_N_INV).exp() * z0)
            .collect()
    }
}

impl<const M: i32, const N: i32> InfinityFirstReturnMap for McMullenFamily<M, N>
{
    degree_impl!(M as AngleNum);
}

impl<const M: i32, const N: i32> EscapeEncoding for McMullenFamily<M, N> {}
impl<const M: i32, const N: i32> ExternalRays for McMullenFamily<M, N> {}
