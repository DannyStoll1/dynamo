use crate::macros::profile_imports;
profile_imports!();

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Tricorne<const N: Period>
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl<const N: Period> Tricorne<N>
{
    const N_FLOAT: Real = N as Real;
    const N_MINUS_1: Real = (N - 1) as Real;
    const DEFAULT_BOUNDS: Bounds = match N
    {
        2 => Bounds {
            min_x: -2.4,
            max_x: 1.5,
            min_y: -2.2,
            max_y: 2.2,
        },
        _ => Bounds::centered_square(1.4),
    };
}

impl<const N: Period> Default for Tricorne<N>
{
    fractal_impl!();
}

impl<const N: Period> ParameterPlane for Tricorne<N>
{
    parameter_plane_impl!();
    basic_escape_encoding!(Self::N_FLOAT);
    default_name!();
    default_bounds!();

    #[inline]
    fn degree_real(&self) -> f64
    {
        Self::N_FLOAT
    }

    #[inline]
    fn map(&self, z: Self::Var, c: Self::Param) -> Self::Var
    {
        z.powf(Self::N_FLOAT).conj() + c
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, c: Self::Param) -> (Self::Var, Self::Deriv)
    {
        let z_n_minus_1 = z.powf(Self::N_MINUS_1);
        (
            (z_n_minus_1 * z).conj() + c,
            Self::N_FLOAT * z_n_minus_1.conj(),
        )
    }

    #[inline]
    fn start_point(&self, _point: Cplx, _c: Self::Param) -> Self::Var
    {
        ZERO
    }

    #[inline]
    fn dynamical_derivative(&self, z: Self::Var, _c: Self::Param) -> Self::Deriv
    {
        Self::N_FLOAT * z.powf(Self::N_MINUS_1)
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
