use crate::macros::profile_imports;
profile_imports!();

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MinsikHanPhi<const D: i32>
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl<const D: i32> MinsikHanPhi<D>
{
    const D_FLOAT: Real = D as Real;
    const D_MINUS_1: Real = Self::D_FLOAT - 1.;
    const DEFAULT_BOUNDS: Bounds = Bounds::centered_square(12.);

    #[must_use]
    pub const fn new(point_grid: PointGrid, max_iter: Period) -> Self
    {
        Self {
            point_grid,
            max_iter,
        }
    }
}

impl<const D: i32> Default for MinsikHanPhi<D>
{
    fractal_impl!();
}

impl<const D: i32> ParameterPlane for MinsikHanPhi<D>
{
    parameter_plane_impl!();

    fn map(&self, z: Self::Var, a: Self::Param) -> Self::Var
    {
        let u = z.powi(D) + Self::D_MINUS_1;
        a * z / u
    }

    fn map_and_multiplier(&self, z: Self::Var, a: Self::Param) -> (Self::Var, Self::Deriv)
    {
        let u = z.powi(D) + Self::D_MINUS_1;
        (
            a * z / u,
            -a * (Self::D_MINUS_1) * (u - Self::D_FLOAT) / (u * u),
        )
    }

    fn dynamical_derivative(&self, z: Self::Var, a: Self::Param) -> Self::Deriv
    {
        let u = z.powi(D) + Self::D_MINUS_1;
        -a * (Self::D_MINUS_1) * (u - Self::D_MINUS_1 - 1.) / (u * u)
    }

    fn parameter_derivative(&self, z: Self::Var, _a: Self::Param) -> Self::Deriv
    {
        z / (z.powi(D) + Self::D_MINUS_1)
    }

    fn start_point(&self, _point: Cplx, _a: Self::Param) -> Self::Var
    {
        ONE
    }

    fn critical_points_child(&self, _param: Self::Param) -> Vec<Self::Var>
    {
        (0..D)
            .map(|k| (TAUI * f64::from(k) / Self::D_FLOAT).exp())
            .collect()
    }
    fn default_selection(&self) -> Cplx
    {
        Cplx::new(Self::D_FLOAT, 0.0)
        // ComplexNum::new(8.03871259641341, 4.08815358590093)
    }

    fn name(&self) -> String
    {
        format!("Minsik Han Family, degree {D}")
    }
}
