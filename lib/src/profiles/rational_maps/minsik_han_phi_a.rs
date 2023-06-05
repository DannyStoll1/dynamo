use crate::{macros::*, math_utils::solve_quadratic};
profile_imports!();

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MinsikHanPhi
{
    point_grid: PointGrid,
    max_iter: Period,
    d: i32,
    d_float: RealNum,
    d_minus_1: RealNum,
}

impl MinsikHanPhi
{
    const DEFAULT_BOUNDS: Bounds = Bounds::centered_square(12.);

    #[must_use]
    pub const fn new(point_grid: PointGrid, max_iter: Period, d: i32) -> Self
    {
        let d_float = d as RealNum;
        Self {
            point_grid,
            max_iter,
            d,
            d_float,
            d_minus_1: d_float - 1.,
        }
    }
}

impl Default for MinsikHanPhi
{
    fn default() -> Self
    {
        let bounds = Self::DEFAULT_BOUNDS;
        let point_grid = PointGrid::new_by_res_y(1024, bounds);
        Self::new(point_grid, 1024, 7)
    }
}

impl ParameterPlane for MinsikHanPhi
{
    type MetaParam = i32;
    type Param = ComplexNum;
    type Var = ComplexNum;
    type Deriv = ComplexNum;
    type Child = JuliaSet<Self>;
    basic_plane_impl!();
    default_name!();

    fn map(&self, z: Self::Var, a: Self::Param) -> Self::Var
    {
        let u = z.powi(self.d) + self.d_minus_1;
        a * z / u
    }

    fn map_and_multiplier(&self, z: Self::Var, a: Self::Param) -> (Self::Var, Self::Deriv)
    {
        let u = z.powi(self.d) + self.d_minus_1;
        (
            a * z / u,
            -a * (self.d_minus_1) * (u - self.d_float) / (u * u),
        )
    }

    fn dynamical_derivative(&self, z: Self::Var, a: Self::Param) -> Self::Deriv
    {
        let u = z.powi(self.d) + self.d_minus_1;
        -a * (self.d_minus_1) * (u - self.d_minus_1 - 1.) / (u * u)
    }

    fn parameter_derivative(&self, z: Self::Var, _a: Self::Param) -> Self::Deriv
    {
        z / (z.powi(self.d) + self.d_minus_1)
    }

    fn start_point(&self, _point: ComplexNum, _a: Self::Param) -> Self::Var
    {
        ONE
    }

    fn critical_points_child(&self, _param: Self::Param) -> Vec<Self::Var>
    {
        (0..self.d)
            .map(|k| (TAUI * (k as RealNum) / self.d_float).exp())
            .collect()
    }
    fn default_selection(&self) -> ComplexNum
    {
        ComplexNum::new(self.d_float, 0.0)
        // ComplexNum::new(8.03871259641341, 4.08815358590093)
    }
    fn set_param(&mut self, d: <Self::MetaParam as ParamList>::Param) {
        self.d = d;
        self.d_float = d as RealNum;
        self.d_minus_1 = self.d_float - 1.;
    }
    fn get_meta_params(&self) -> Self::MetaParam {
        self.d
    }
    fn get_param(&self) -> <Self::MetaParam as ParamList>::Param {
        self.d
    }
}
