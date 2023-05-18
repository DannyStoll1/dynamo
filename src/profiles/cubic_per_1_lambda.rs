use crate::macros::*;
profile_imports!();

#[derive(Clone, Debug)]
pub struct CubicPer1Lambda
{
    point_grid: PointGrid,
    max_iter: Period,
    lambda: ComplexNum,
}

impl CubicPer1Lambda
{
    const DEFAULT_BOUNDS: Bounds = Bounds::centered_square(2.5);

    #[must_use]
    pub const fn new(
        res_x: usize,
        res_y: usize,
        max_iter: Period,
        bounds: Bounds,
        param: ComplexNum,
    ) -> Self
    {
        let point_grid = PointGrid::new(res_x, res_y, bounds);

        Self {
            point_grid,
            max_iter,
            lambda: param,
        }
    }

    #[must_use]
    pub const fn with_res_x(
        res_x: usize,
        max_iter: Period,
        bounds: Bounds,
        param: ComplexNum,
    ) -> Self
    {
        let point_grid = PointGrid::with_res_x(res_x, bounds);

        Self {
            point_grid,
            max_iter,
            lambda: param,
        }
    }

    #[must_use]
    pub const fn with_res_y(
        res_y: usize,
        max_iter: Period,
        bounds: Bounds,
        param: ComplexNum,
    ) -> Self
    {
        let point_grid = PointGrid::with_res_y(res_y, bounds);

        Self {
            point_grid,
            max_iter,
            lambda: param,
        }
    }

    #[must_use]
    pub const fn new_default(res_y: usize, max_iter: Period) -> Self
    {
        let bounds = Self::DEFAULT_BOUNDS;
        Self::with_res_y(res_y, max_iter, bounds, ZERO)
    }
}

impl ParameterPlane for CubicPer1Lambda
{
    parameter_plane_impl!();
    basic_escape_encoding!(3., 1.);

    #[inline]
    fn map(&self, z: Self::Var, c: Self::Param) -> Self::Var
    {
        z * horner_monic!(z, self.lambda, c)
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, c: Self::Param) -> (Self::Var, Self::Deriv)
    {
        let z2 = z * z;
        let u = z2 + c * z + self.lambda;
        (z * u, u + z * (c + z + z))
    }

    #[inline]
    fn dynamical_derivative(&self, z: Self::Var, c: Self::Param) -> Self::Deriv
    {
        let z2 = z * z;
        let u = z2 + c * z + self.lambda;
        u + z * (c + z + z)
    }

    #[inline]
    fn parameter_derivative(&self, z: Self::Var, _c: Self::Param) -> Self::Deriv
    {
        z
    }

    #[inline]
    fn start_point(&self, m: ComplexNum, _c: Self::Param) -> Self::Var
    {
        0.5 * self.lambda * m
    }

    fn critical_points(&self, c: Self::Param) -> Vec<Self::Var>
    {
        let disc = (self.lambda * self.lambda - 3. * c).sqrt();
        vec![
            -ONE_THIRD * (self.lambda + disc),
            -ONE_THIRD * (self.lambda - disc),
        ]
    }

    fn get_param(&self) -> Self::Param
    {
        self.lambda
    }

    fn set_param(&mut self, value: Self::Param)
    {
        self.lambda = value
    }

    fn param_map(&self, m: ComplexNum) -> Self::Param
    {
        -m.inv() - 0.75 * self.lambda * m
    }

    fn name(&self) -> String
    {
        format!("Cubic Per(1, {})", self.lambda)
    }
}

#[derive(Clone, Debug)]
pub struct CubicPer1LambdaParam
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl CubicPer1LambdaParam
{
    fractal_impl!(-2.5, 2.5, -2.5, 2.5);
}

impl ParameterPlane for CubicPer1LambdaParam
{
    parameter_plane_impl!();
    basic_escape_encoding!(3., 1.);

    #[inline]
    fn map(&self, z: Self::Var, a: Self::Param) -> Self::Var
    {
        let z2 = z * z;
        z * (z2 + a)
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, a: Self::Param) -> (Self::Var, Self::Deriv)
    {
        let z2 = z * z;
        let u = z2 + a;
        (z * u, u + z * (a + z + z))
    }

    #[inline]
    fn dynamical_derivative(&self, z: Self::Var, a: Self::Param) -> Self::Deriv
    {
        let z2 = z * z;
        let u = z2 + a;
        u + z * (a + z + z)
    }

    #[inline]
    fn parameter_derivative(&self, z: Self::Var, _c: Self::Param) -> Self::Deriv
    {
        z
    }

    #[inline]
    fn start_point(&self, _point: ComplexNum, c: Self::Param) -> Self::Var
    {
        -ONE_THIRD * (c + c)
    }

    fn critical_points(&self, _param: Self::Param) -> Vec<Self::Var>
    {
        vec![ZERO, ONE]
    }

    fn name(&self) -> String
    {
        "Cubic Per(1, lambda) lambda-plane".to_owned()
    }

    fn default_selection(&self) -> ComplexNum
    {
        ZERO
    }

    fn default_julia_bounds(&self, point: ComplexNum, _param: Self::Param) -> Bounds
    {
        let r = 3.5 / (point.norm() + 0.01);
        Bounds::centered_square(r)
    }
}

impl From<CubicPer1LambdaParam> for CubicPer1Lambda
{
    fn from(parent: CubicPer1LambdaParam) -> Self
    {
        let point = parent.default_selection();
        let param = parent.param_map(point);
        let point_grid = parent
            .point_grid()
            .with_same_height(parent.default_julia_bounds(point, param));
        Self {
            point_grid,
            max_iter: parent.max_iter(),
            lambda: param,
        }
    }
}
