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

impl Default for CubicPer1Lambda
{
    fn default() -> Self
    {
        let bounds = Self::DEFAULT_BOUNDS;
        let max_iter = 1024;
        Self::with_res_y(1024, max_iter, bounds, ZERO)
    }
}

impl ParameterPlane for CubicPer1Lambda
{
    parameter_plane_impl!(ComplexNum, ComplexNum, ComplexNum, ComplexNum);
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

    fn critical_points_child(&self, c: Self::Param) -> Vec<Self::Var>
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

    fn get_local_param(&self) -> Self::Param
    {
        self.lambda
    }

    fn set_meta_param(&mut self, value: Self::Param)
    {
        self.lambda = value
    }

    fn set_param(&mut self, value: <Self::MetaParam as ParamList>::Param)
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
    fractal_impl!(-2.2, 4.2, -2.5, 2.5);

    const BASE_POINT: ComplexNum = ComplexNum::new(1e-4, 0.);

    fn base_param(lambda: ComplexNum) -> ComplexNum
    {
        -Self::BASE_POINT.inv() - 0.75 * lambda * Self::BASE_POINT
    }
}

impl ParameterPlane for CubicPer1LambdaParam
{
    parameter_plane_impl!(CubicPer1Lambda);
    basic_escape_encoding!(3., 1.);

    #[inline]
    fn map(&self, z: Self::Var, a: Self::Param) -> Self::Var
    {
        let c = Self::base_param(a);
        z * horner_monic!(z, a, c)
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, a: Self::Param) -> (Self::Var, Self::Deriv)
    {
        let c = Self::base_param(a);
        let z2 = z * z;
        let u = z2 + c * z + a;
        (z * u, u + z * (c + z + z))
    }

    #[inline]
    fn dynamical_derivative(&self, z: Self::Var, a: Self::Param) -> Self::Deriv
    {
        let c = Self::base_param(a);
        let z2 = z * z;
        let u = z2 + c * z + a;
        u + z * (c + z + z)
    }

    #[inline]
    fn parameter_derivative(&self, z: Self::Var, _c: Self::Param) -> Self::Deriv
    {
        z
    }

    #[inline]
    fn start_point(&self, _point: ComplexNum, c: Self::Param) -> Self::Var
    {
        0.5 * c * Self::BASE_POINT
    }

    #[inline]
    fn param_map(&self, point: ComplexNum) -> Self::Param
    {
        point
    }

    fn critical_points_child(&self, _param: Self::Param) -> Vec<Self::Var>
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

#[derive(Clone, Debug)]
pub struct CubicPer1_1
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl CubicPer1_1
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -2.5,
        max_x: 2.5,
        min_y: -2.2,
        max_y: 2.2,
    };
    fractal_impl!();
}

impl Default for CubicPer1_1
{
    fn default() -> Self
    {
        let bounds = Self::DEFAULT_BOUNDS;
        let max_iter = 3164;
        Self::with_res_y(1024, max_iter, bounds)
    }
}

impl ParameterPlane for CubicPer1_1
{
    parameter_plane_impl!();
    default_name!();

    fn periodicity_tolerance(&self) -> RealNum
    {
        1e-6
    }
    fn min_iter(&self) -> Period
    {
        self.max_iter() / 3
    }
    fn encode_escaping_point(
        &self,
        iters: Period,
        z: ComplexNum,
        _base_param: ComplexNum,
    ) -> PointInfo<Self::Deriv>
    {
        if z.is_nan()
        {
            return PointInfo::Escaping {
                potential: f64::from(iters) - 1.,
            };
        }

        let u = self.escape_radius().log2();
        let v = z.norm_sqr().log2();
        let residual = (v / u).log(3.);
        let potential = f64::from(iters) - (residual as IterCount);
        PointInfo::Escaping { potential }
    }

    #[inline]
    fn map(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        z * (z * (z + c) + 1.)
    }

    #[inline]
    fn start_point(&self, _point: ComplexNum, param: ComplexNum) -> ComplexNum
    {
        let mut u = (param * param - 3.).sqrt();
        if param.re < 0.
        {
            u = -u
        }
        -(param + u) / 3.
    }

    #[inline]
    fn dynamical_derivative(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        z * (2. * c + 3. * z) + 1.
    }

    #[inline]
    fn parameter_derivative(&self, z: ComplexNum, _c: ComplexNum) -> ComplexNum
    {
        z * z
    }

    #[inline]
    fn critical_points_child(&self, param: ComplexNum) -> ComplexVec
    {
        let u = (param * param - 3.).sqrt();
        vec![-(param + u) / 3., (u - param) / 3.]
    }

    #[inline]
    fn default_julia_bounds(&self, _point: ComplexNum, _param: ComplexNum) -> Bounds
    {
        Bounds::centered_square(2.2)
    }
}
