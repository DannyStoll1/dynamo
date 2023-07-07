use super::quad_rat_general::QuadRatGeneral;
use crate::macros::{horner_monic, profile_imports};
profile_imports!();

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct QuadRatPer2Lambda
{
    general_plane: QuadRatGeneral,
    multiplier: Cplx,
}

impl Default for QuadRatPer2Lambda
{
    fn default() -> Self
    {
        let general_plane = QuadRatGeneral::default();
        Self {
            general_plane,
            multiplier: ZERO,
        }
    }
}

impl ParameterPlane for QuadRatPer2Lambda
{
    type Var = Cplx;
    type Param = CplxPair;
    type Deriv = Cplx;
    type MetaParam = Cplx;
    type Child = JuliaSet<Self>;

    fn max_iter(&self) -> Period
    {
        self.general_plane.max_iter
    }

    fn max_iter_mut(&mut self) -> &mut Period
    {
        &mut self.general_plane.max_iter
    }

    fn set_max_iter(&mut self, new_max_iter: Period)
    {
        self.general_plane.max_iter = new_max_iter;
    }

    #[must_use]
    fn with_max_iter(mut self, max_iter: Period) -> Self
    {
        self.general_plane.max_iter = max_iter;
        self
    }

    fn point_grid(&self) -> &PointGrid
    {
        &self.general_plane.point_grid
    }

    fn point_grid_mut(&mut self) -> &mut PointGrid
    {
        &mut self.general_plane.point_grid
    }

    fn with_point_grid(mut self, point_grid: PointGrid) -> Self
    {
        self.general_plane.point_grid = point_grid;
        self
    }

    fn map(&self, z: Self::Var, c: Self::Param) -> Self::Var
    {
        self.general_plane.map(z, c)
    }

    fn param_map(&self, t: Cplx) -> Self::Param
    {
        let alpha = 0.25 * t;
        let u = self.multiplier * t - 4.;
        CplxPair {
            a: alpha * t * (self.multiplier + u),
            b: alpha * u,
        }
    }

    fn map_and_multiplier(&self, z: Self::Var, c: Self::Param) -> (Self::Var, Self::Deriv)
    {
        self.general_plane.map_and_multiplier(z, c)
    }

    fn dynamical_derivative(&self, z: Self::Var, c: Self::Param) -> Self::Deriv
    {
        self.general_plane.dynamical_derivative(z, c)
    }

    fn parameter_derivative(&self, _z: Self::Var, _c: Self::Param) -> Self::Deriv
    {
        ONE
    }

    fn start_point(&self, t: Cplx, c: Self::Param) -> Self::Var
    {
        self.general_plane.start_point(t, c)
    }

    fn critical_points_child(&self, c: Self::Param) -> Vec<Self::Var>
    {
        self.general_plane.critical_points_child(c)
    }

    fn cycles_child(&self, c: Self::Param, period: Period) -> Vec<Self::Var>
    {
        self.general_plane.cycles_child(c, period)
    }

    fn get_param(&self) -> <Self::MetaParam as ParamList>::Param
    {
        self.multiplier
    }

    fn set_param(&mut self, lambda: <Self::MetaParam as ParamList>::Param)
    {
        self.multiplier = lambda;
    }

    fn name(&self) -> String
    {
        "QuadRat Per(2, λ)".to_owned()
        // format!("QuadRat Per(2, λ); λ={:.4}", self.multiplier)
    }
}

#[derive(Clone, Debug)]
pub struct QuadRatPer2LambdaParam
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl QuadRatPer2LambdaParam
{
    const BASE_POINT: Cplx = Cplx::new(1e-4, 0.);

    fn base_param(lambda: Cplx) -> Cplx
    {
        -Self::BASE_POINT.inv() - 0.75 * lambda * Self::BASE_POINT
    }
}
impl Default for QuadRatPer2LambdaParam
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
        }
    }
}

impl ParameterPlane for QuadRatPer2LambdaParam
{
    type Var = Cplx;
    type Param = Cplx;
    type Deriv = Cplx;
    type MetaParam = NoParam;
    type Child = QuadRatPer2Lambda;
    basic_plane_impl!();
    basic_escape_encoding!(2., 1.);

    #[inline]
    fn map(&self, z: Self::Var, l: Self::Param) -> Self::Var
    {
        let a = 4. / l;
        1. + a / (z * z)
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, l: Self::Param) -> (Self::Var, Self::Deriv)
    {
        let a = 4. / l;
        let z2 = z * z;
        (1. + a / z2, -(a + a) / (z2 * z))
    }

    #[inline]
    fn dynamical_derivative(&self, z: Self::Var, l: Self::Param) -> Self::Deriv
    {
        let a = 4. / l;
        -(a + a) / (z * z * z)
    }

    #[inline]
    fn parameter_derivative(&self, z: Self::Var, _c: Self::Param) -> Self::Deriv
    {
        (z * z).inv()
    }

    #[inline]
    fn start_point(&self, _point: Cplx, _c: Self::Param) -> Self::Var
    {
        ONE
    }

    #[inline]
    fn param_map(&self, point: Cplx) -> Self::Param
    {
        point
    }

    fn critical_points_child(&self, _param: Self::Param) -> Vec<Self::Var>
    {
        vec![ZERO, ONE]
    }

    fn name(&self) -> String
    {
        "QuadRat Per(2, λ) λ-plane".to_owned()
    }

    fn default_selection(&self) -> Cplx
    {
        ONE
    }

    fn default_julia_bounds(&self, point: Cplx, _param: Self::Param) -> Bounds
    {
        let r = 4. / (point.norm() + 0.01);
        Bounds::centered_square(r)
    }
}

impl From<QuadRatPer2LambdaParam> for QuadRatPer2Lambda
{
    fn from(parent: QuadRatPer2LambdaParam) -> Self
    {
        let point = parent.default_selection();
        let param = parent.param_map(point);
        let point_grid = parent
            .point_grid()
            .new_with_same_height(parent.default_julia_bounds(point, param));
        let general_plane = QuadRatGeneral {
            point_grid,
            max_iter: parent.max_iter(),
        };
        Self {
            general_plane,
            multiplier: point,
        }
    }
}
