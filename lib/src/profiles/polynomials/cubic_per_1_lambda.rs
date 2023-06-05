use crate::macros::*;
use crate::types::variables::PlaneID;
profile_imports!();

#[derive(Clone, Debug)]
pub struct CubicPer1Lambda
{
    point_grid: PointGrid,
    max_iter: Period,
    multiplier: ComplexNum,
    starting_crit: PlaneID,
}

impl CubicPer1Lambda
{
    const DEFAULT_BOUNDS: Bounds = Bounds::centered_square(6.5);
}

impl Default for CubicPer1Lambda
{
    fn default() -> Self
    {
        let bounds = Self::DEFAULT_BOUNDS;
        let point_grid = PointGrid::new_by_res_y(1024, bounds);
        Self {
            point_grid,
            max_iter: 1024,
            multiplier: ZERO,
            starting_crit: PlaneID::ZPlane,
        }
    }
}

impl ParameterPlane for CubicPer1Lambda
{
    parameter_plane_impl!(ComplexNum, ComplexNum, ComplexNum, ComplexNum);
    basic_escape_encoding!(3., 1.);

    #[inline]
    fn map(&self, z: Self::Var, c: Self::Param) -> Self::Var
    {
        z * horner_monic!(z, self.multiplier, c)
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, c: Self::Param) -> (Self::Var, Self::Deriv)
    {
        let z2 = z * z;
        let u = z2 + c * z + self.multiplier;
        (z * u, u + z * (c + z + z))
    }

    #[inline]
    fn dynamical_derivative(&self, z: Self::Var, c: Self::Param) -> Self::Deriv
    {
        let z2 = z * z;
        let u = z2 + c * z + self.multiplier;
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
        match self.starting_crit
        {
            PlaneID::ZPlane => 0.5 * self.multiplier * m,
            PlaneID::WPlane => TWO_THIRDS / m,
        }
    }

    fn critical_points(&self) -> Vec<Self::Var>
    {
        let l = self.multiplier;
        let d0 = l.sqrt();
        let d1 = (l - 2.).sqrt();
        let u = 2. / l;
        let r0 = u * d0;
        let r1 = u * d1;
        vec![r0, -r0, r1, -r1, u, -u]
    }

    fn critical_points_child(&self, c: Self::Param) -> Vec<Self::Var>
    {
        let disc = (c * c - 3. * self.multiplier).sqrt();
        vec![-ONE_THIRD * (c + disc), -ONE_THIRD * (c - disc)]
    }

    fn cycles_child(&self, c: Self::Param, period: Period) -> Vec<Self::Var>
    {
        match period
        {
            1 =>
            {
                let disc = (c * c + 4. * (4. - self.multiplier)).sqrt();
                vec![ZERO, -0.5 * (c + disc), 0.5 * (disc - c)]
            }
            _ => vec![],
        }
    }

    fn get_meta_params(&self) -> Self::Param
    {
        self.multiplier
    }

    fn get_param(&self) -> Self::Param
    {
        self.multiplier
    }

    fn set_meta_param(&mut self, value: Self::Param)
    {
        self.multiplier = value
    }

    fn set_param(&mut self, value: <Self::MetaParam as ParamList>::Param)
    {
        self.multiplier = value
    }

    fn param_map(&self, m: ComplexNum) -> Self::Param
    {
        -m.inv() - 0.75 * self.multiplier * m
    }

    fn name(&self) -> String
    {
        format!("Cubic Per(1, {}) {}", self.multiplier, self.starting_crit)
    }

    fn default_bounds(&self) -> Bounds
    {
        let r = 4. / (self.multiplier.norm() + 0.01);
        Bounds::centered_square(r)
    }

    fn cycle_active_plane(&mut self)
    {
        self.starting_crit = self.starting_crit.swap();
    }
}

#[derive(Clone, Debug)]
pub struct CubicPer1LambdaParam
{
    point_grid: PointGrid,
    max_iter: Period,
    starting_crit: PlaneID,
}

impl CubicPer1LambdaParam
{
    const BASE_POINT: ComplexNum = ComplexNum::new(1e-4, 0.);

    fn base_param(lambda: ComplexNum) -> ComplexNum
    {
        -Self::BASE_POINT.inv() - 0.75 * lambda * Self::BASE_POINT
    }
}
impl Default for CubicPer1LambdaParam
{
    fn default() -> Self
    {
        let bounds = Bounds {
            min_x: -2.2,
            max_x: 4.2,
            min_y: -2.5,
            max_y: 2.5,
        };
        let point_grid = PointGrid::new_by_res_y(1024, bounds);
        Self {
            point_grid,
            max_iter: 1024,
            starting_crit: PlaneID::ZPlane,
        }
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
        match self.starting_crit
        {
            PlaneID::ZPlane => 0.5 * c * Self::BASE_POINT,
            PlaneID::WPlane => TWO_THIRDS / Self::BASE_POINT,
        }
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
        let r = 4. / (point.norm() + 0.01);
        Bounds::centered_square(r)
    }

    fn cycle_active_plane(&mut self)
    {
        self.starting_crit = self.starting_crit.swap();
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
            .new_with_same_height(parent.default_julia_bounds(point, param));
        Self {
            point_grid,
            max_iter: parent.max_iter(),
            multiplier: param,
            starting_crit: parent.starting_crit,
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
}

impl Default for CubicPer1_1
{
    fractal_impl!();
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

    fn cycles_child(&self, c: Self::Param, period: Period) -> Vec<Self::Var>
    {
        match period
        {
            1 =>
            {
                vec![ZERO, -c]
            }
            _ => vec![],
        }
    }

    #[inline]
    fn default_julia_bounds(&self, _point: ComplexNum, _param: ComplexNum) -> Bounds
    {
        Bounds::centered_square(2.2)
    }
}
