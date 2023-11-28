use super::quad_rat_general::QuadRatGeneral;
use crate::macros::{degree_impl, has_child_impl, horner_monic, profile_imports};
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

impl DynamicalFamily for QuadRatPer2Lambda
{
    type Var = Cplx;
    type Param = CplxPair;
    type Deriv = Cplx;
    type MetaParam = Cplx;

    fn max_iter(&self) -> IterCount
    {
        self.general_plane.max_iter
    }

    fn max_iter_mut(&mut self) -> &mut IterCount
    {
        &mut self.general_plane.max_iter
    }

    fn set_max_iter(&mut self, new_max_iter: IterCount)
    {
        self.general_plane.max_iter = new_max_iter;
    }

    #[must_use]
    fn with_max_iter(mut self, max_iter: IterCount) -> Self
    {
        self.general_plane.max_iter = max_iter;
        self
    }

    #[inline]
    fn point_grid(&self) -> &PointGrid
    {
        &self.general_plane.point_grid
    }

    #[inline]
    fn point_grid_mut(&mut self) -> &mut PointGrid
    {
        &mut self.general_plane.point_grid
    }

    #[inline]
    fn with_point_grid(mut self, point_grid: PointGrid) -> Self
    {
        self.general_plane.point_grid = point_grid;
        self
    }

    #[inline]
    fn compute_mode(&self) -> ComputeMode
    {
        self.general_plane.compute_mode()
    }

    #[inline]
    fn compute_mode_mut(&mut self) -> &mut ComputeMode
    {
        self.general_plane.compute_mode_mut()
    }

    #[inline]
    fn map(&self, z: Self::Var, c: &Self::Param) -> Self::Var
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

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv)
    {
        self.general_plane.map_and_multiplier(z, c)
    }

    #[inline]
    fn start_point(&self, t: Cplx, c: &Self::Param) -> Self::Var
    {
        self.general_plane.start_point(t, c)
    }

    #[inline]
    fn get_param(&self) -> <Self::MetaParam as ParamList>::Param
    {
        self.multiplier
    }
    #[inline]
    fn set_param(&mut self, lambda: <Self::MetaParam as ParamList>::Param)
    {
        self.multiplier = lambda;
    }

    #[inline]
    fn name(&self) -> String
    {
        "QuadRat Per(2, λ)".to_owned()
        // format!("QuadRat Per(2, λ); λ={:.4}", self.multiplier)
    }
}

impl FamilyDefaults for QuadRatPer2Lambda
{
    fn default_bounds(&self) -> Bounds
    {
        let r = 4. / (self.multiplier.norm() + 0.01);
        Bounds::centered_square(r)
    }
}

has_child_impl!(QuadRatPer2Lambda);

impl MarkedPoints for QuadRatPer2Lambda
{
    #[inline]
    fn critical_points_child(&self, c: &Self::Param) -> Vec<Self::Var>
    {
        self.general_plane.critical_points_child(c)
    }

    #[inline]
    fn cycles_child(&self, c: &Self::Param, period: Period) -> Vec<Self::Var>
    {
        self.general_plane.cycles_child(c, period)
    }
}

#[derive(Clone, Debug)]
pub struct QuadRatPer2LambdaParam
{
    point_grid: PointGrid,
    compute_mode: ComputeMode,
    max_iter: IterCount,
}

impl QuadRatPer2LambdaParam
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -2.2,
        max_x: 4.2,
        min_y: -2.5,
        max_y: 2.5,
    };
}

impl Default for QuadRatPer2LambdaParam
{
    fn default() -> Self
    {
        let point_grid = PointGrid::new_by_res_y(1024, Self::DEFAULT_BOUNDS);
        Self {
            point_grid,
            compute_mode: ComputeMode::default(),
            max_iter: 1024,
        }
    }
}

impl DynamicalFamily for QuadRatPer2LambdaParam
{
    type Var = Cplx;
    type Param = Cplx;
    type Deriv = Cplx;
    type MetaParam = NoParam;
    basic_plane_impl!();

    #[inline]
    fn map(&self, z: Self::Var, l: &Self::Param) -> Self::Var
    {
        let a = 4. / l;
        1. + a / z.powi(2)
    }

    #[allow(clippy::suspicious_operation_groupings)]
    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, l: &Self::Param) -> (Self::Var, Self::Deriv)
    {
        let a = 4. / l;
        let u = a * z.powi(-2);
        (1. + u, -2. * u / z)
    }

    fn gradient(&self, z: Self::Var, l: &Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        let a = 4. / l;
        let u = a * z.powi(-2);
        (1. + u, -2. * u / z, -u / l)
    }

    #[inline]
    fn start_point(&self, _point: Cplx, _c: &Self::Param) -> Self::Var
    {
        ONE
    }

    #[inline]
    fn param_map(&self, point: Cplx) -> Self::Param
    {
        point
    }

    #[inline]
    fn param_map_d(&self, point: Cplx) -> (Self::Param, Self::Deriv)
    {
        (point, ONE)
    }

    #[inline]
    fn name(&self) -> String
    {
        "QuadRat Per(2, λ) λ-plane".to_owned()
    }
}

impl FamilyDefaults for QuadRatPer2LambdaParam
{
    default_bounds!();

    fn default_selection(&self) -> Cplx
    {
        ONE
    }
}

impl HasChild<QuadRatPer2Lambda> for QuadRatPer2LambdaParam
{
    fn to_child_param(
        param: Self::Param,
    ) -> <<QuadRatPer2Lambda as DynamicalFamily>::MetaParam as ParamList>::Param
    {
        param
    }
}

impl MarkedPoints for QuadRatPer2LambdaParam
{
    fn critical_points_child(&self, _param: &Self::Param) -> Vec<Self::Var>
    {
        vec![ZERO, ONE]
    }
}

impl From<QuadRatPer2LambdaParam> for QuadRatPer2Lambda
{
    fn from(parent: QuadRatPer2LambdaParam) -> Self
    {
        let point = parent.default_selection();
        let point_grid = parent.point_grid().clone();
        let general_plane = QuadRatGeneral {
            point_grid,
            compute_mode: ComputeMode::default(),
            max_iter: parent.max_iter(),
        };
        Self {
            general_plane,
            multiplier: point,
        }
        .with_default_bounds()
    }
}

degree_impl!(QuadRatPer2Lambda, 1, 1);
degree_impl!(QuadRatPer2LambdaParam, 1, 1);
