use crate::macros::{degree_impl, profile_imports};
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
    const DEFAULT_BOUNDS: Bounds = match N {
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

impl<const N: Period> DynamicalFamily for Tricorne<N>
{
    parameter_plane_impl!();
    default_name!();

    #[inline]
    fn map(&self, z: Self::Var, c: &Self::Param) -> Self::Var
    {
        z.powf(Self::N_FLOAT).conj() + c
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv)
    {
        let z_n_minus_1 = z.powf(Self::N_MINUS_1);
        (
            (z_n_minus_1 * z).conj() + c,
            Self::N_FLOAT * z_n_minus_1.conj(),
        )
    }

    #[inline]
    fn start_point(&self, _point: Cplx, _c: &Self::Param) -> Self::Var
    {
        ZERO
    }
}

impl<const N: Period> FamilyDefaults for Tricorne<N>
{
    default_bounds!();
}

impl<const N: Period> HasChild for Tricorne<N>
{
    type Child = JuliaSet<Self>;
}

impl<const N: Period> MarkedPoints for Tricorne<N>
{
    fn critical_points_child(&self, _param: &Self::Param) -> Vec<Self::Var>
    {
        vec![ZERO]
    }
}

impl<const N: Period> InfinityFirstReturnMap for Tricorne<N>
{
    degree_impl!(N as AngleNum);
}
impl<const N: Period> EscapeEncoding for Tricorne<N> {}
impl<const N: Period> ExternalRays for Tricorne<N> {}
