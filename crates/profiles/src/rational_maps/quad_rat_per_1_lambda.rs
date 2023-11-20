use super::quad_rat_general::QuadRatGeneral;
use crate::macros::{degree_impl, has_child_impl, horner_monic, profile_imports};
profile_imports!();

// Maps of the form f_t(z) = (z^2+a_t)/(z^2+b_t),
// with a fixed point at z0 = -t/2 of multiplier lambda
// Critical points are 0 and infinity.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct QuadRatPer1Lambda
{
    general_plane: QuadRatGeneral,
    multiplier: Cplx,
    tolerance: Real,
}

impl Default for QuadRatPer1Lambda
{
    fn default() -> Self
    {
        let general_plane = QuadRatGeneral::default();
        let tolerance = Self::compute_tolerance(ZERO);
        Self {
            general_plane,
            multiplier: ZERO,
            tolerance,
        }
    }
}
impl QuadRatPer1Lambda
{
    fn compute_tolerance(multiplier: Cplx) -> Real
    {
        let err = multiplier.norm() - 1.;
        if err > 1e-3 {
            return 1e-12;
        }
        let err2 = -1e8 * err * err;
        err2.exp2().mul_add(1e-8, 1e-12)
    }
}

impl DynamicalFamily for QuadRatPer1Lambda
{
    type Var = Cplx;
    type Param = CplxPair;
    type Deriv = Cplx;
    type MetaParam = Cplx;
    default_name!();

    #[inline]
    fn max_iter(&self) -> Period
    {
        self.general_plane.max_iter
    }

    #[inline]
    fn max_iter_mut(&mut self) -> &mut Period
    {
        &mut self.general_plane.max_iter
    }

    #[inline]
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

    fn param_map(&self, t: Cplx) -> Self::Param
    {
        let alpha = 0.25 * t / self.multiplier;
        CplxPair {
            a: alpha * t * (t - self.multiplier + 2.),
            b: -alpha * (4. + (self.multiplier + 2.) * t),
        }
    }

    #[inline]
    fn map(&self, z: Self::Var, c: &Self::Param) -> Self::Var
    {
        self.general_plane.map(z, c)
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
    fn start_point_d(&self, point: Cplx, c: &Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        self.general_plane.start_point_d(point, c)
    }

    #[inline]
    fn get_param(&self) -> <Self::MetaParam as ParamList>::Param
    {
        self.multiplier
    }

    #[inline]
    fn periodicity_tolerance(&self) -> Real
    {
        self.tolerance
    }

    #[inline]
    fn set_param(&mut self, lambda: <Self::MetaParam as ParamList>::Param)
    {
        self.multiplier = lambda;
        self.tolerance = Self::compute_tolerance(lambda);
    }
}

impl FamilyDefaults for QuadRatPer1Lambda
{
    fn default_bounds(&self) -> Bounds
    {
        Bounds::centered_square(4.0)
    }
}
has_child_impl!(QuadRatPer1Lambda);

impl MarkedPoints for QuadRatPer1Lambda
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

// impl HasDynamicalCovers for QuadRatPer1Lambda
// {
//     fn marked_cycle_curve(self, period: Period) -> CoveringMap<Self>
//     {
//         let param_map: fn(Cplx) -> Cplx;
//         let bounds: Bounds;
//
//         match period
//         {
//             1 =>
//             {
//                 param_map = |t| {
//                     let u = -2. * self.multiplier;
//                     u * (t + self.multiplier - 2.) / horner_monic!(t, u, self.multiplier)
//                 };
//                 bounds = Bounds {
//                     min_x: -2.5,
//                     max_x: 2.5,
//                     min_y: -2.5,
//                     max_y: 2.5,
//                 };
//             }
//             _ =>
//             {
//                 param_map = |t| t;
//                 bounds = self.point_grid().bounds.clone();
//             }
//         };
//         let grid = self.point_grid().new_with_same_height(bounds);
//         CoveringMap::new(self, param_map, grid)
//     }
//
//     fn dynatomic_curve(self, period: Period) -> CoveringMap<Self>
//     {
//         let param_map: fn(Cplx) -> Cplx;
//         let bounds: Bounds;
//
//         match period
//         {
//             1 => return self.marked_cycle_curve(1),
//             2 =>
//             {
//                 param_map = |t| todo!();
//                 bounds = Bounds {
//                     min_x: -2.5,
//                     max_x: 2.5,
//                     min_y: -2.5,
//                     max_y: 2.5,
//                 };
//             }
//             _ =>
//             {
//                 param_map = |t| t;
//                 bounds = self.point_grid().bounds.clone();
//             }
//         };
//         let grid = self.point_grid().new_with_same_height(bounds);
//         CoveringMap::new(self, param_map, grid)
//     }
//     fn misiurewicz_curve(self, preperiod: Period, period: Period) -> CoveringMap<Self>
//     {
//         let param_map: fn(Cplx) -> Cplx;
//         let bounds: Bounds;
//
//         match (preperiod, period)
//         {
//             (2, 1) =>
//             {
//                 param_map = |t| todo!();
//                 bounds = Bounds {
//                     min_x: -2.5,
//                     max_x: 2.5,
//                     min_y: -2.5,
//                     max_y: 2.5,
//                 };
//             }
//             (_, _) =>
//             {
//                 param_map = |t| t;
//                 bounds = self.point_grid().bounds.clone();
//             }
//         };
//         let grid = self.point_grid().new_with_same_height(bounds);
//         CoveringMap::new(self, param_map, grid)
//     }
// }

#[derive(Clone, Debug)]
pub struct QuadRatPer1LambdaParam
{
    point_grid: PointGrid,
    compute_mode: ComputeMode,
    max_iter: Period,
}

impl QuadRatPer1LambdaParam
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -2.2,
        max_x: 4.2,
        min_y: -2.5,
        max_y: 2.5,
    };
}

impl Default for QuadRatPer1LambdaParam
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

impl DynamicalFamily for QuadRatPer1LambdaParam
{
    parameter_plane_impl!();

    #[inline]
    fn map(&self, z: Self::Var, l: &Self::Param) -> Self::Var
    {
        let a = -4. * l / horner_monic!(l, 8., 12., 6.);
        1. + a / (z * z)
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, l: &Self::Param) -> (Self::Var, Self::Deriv)
    {
        let a = -4. * l / horner_monic!(l, 8., 12., 6.);
        let z2 = z * z;
        (1. + a / z2, -2. * a / (z2 * z))
    }

    #[inline]
    fn gradient(&self, z: Self::Var, l: &Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        let a = -4. * l / horner_monic!(l, 8., 12., 6.);
        let u = (l + 2.).powi(-4);
        let z_i2 = z.powi(-2);
        (1. + a * z_i2, -2. * a * z_i2 / z, 8. * (l - 1.) * u * z_i2)
    }

    #[inline]
    fn start_point(&self, _point: Cplx, _c: &Self::Param) -> Self::Var
    {
        ONE
    }

    #[inline]
    fn start_point_d(&self, _point: Cplx, _c: &Self::Param)
        -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        (ONE, ZERO, ZERO)
    }

    fn name(&self) -> String
    {
        "QuadRat Per(1, λ) λ-plane".to_owned()
    }
}

impl FamilyDefaults for QuadRatPer1LambdaParam
{
    default_bounds!();

    fn default_selection(&self) -> Cplx
    {
        ONE
    }
}

impl HasChild<QuadRatPer1Lambda> for QuadRatPer1LambdaParam
{
    fn to_child_param(
        param: Self::Param,
    ) -> <<QuadRatPer1Lambda as DynamicalFamily>::MetaParam as ParamList>::Param
    {
        param
    }
}

impl MarkedPoints for QuadRatPer1LambdaParam
{
    fn critical_points_child(&self, _param: &Self::Param) -> Vec<Self::Var>
    {
        vec![ZERO, ONE]
    }
}

impl From<QuadRatPer1LambdaParam> for QuadRatPer1Lambda
{
    fn from(parent: QuadRatPer1LambdaParam) -> Self
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
            tolerance: Self::compute_tolerance(point),
        }
        .with_default_bounds()
    }
}

// Maps of the form f_t(z) = (z^2+a_t)/(z^2+b_t),
// with a fixed point at z0 = -t/2 of multiplier lambda
// Critical points are 0 and infinity.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct QuadRatPer1_1
{
    general_plane: QuadRatGeneral,
}

impl Default for QuadRatPer1_1
{
    fn default() -> Self
    {
        let general_plane = QuadRatGeneral::default();
        Self { general_plane }
    }
}

impl DynamicalFamily for QuadRatPer1_1
{
    type Var = Cplx;
    type Param = CplxPair;
    type Deriv = Cplx;
    type MetaParam = NoParam;
    default_name!();

    #[inline]
    fn max_iter(&self) -> Period
    {
        self.general_plane.max_iter
    }

    #[inline]
    fn max_iter_mut(&mut self) -> &mut Period
    {
        &mut self.general_plane.max_iter
    }

    #[inline]
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
        let t = t.inv() - 2.;
        let alpha = 0.25 * t;
        CplxPair {
            a: alpha * t * (t + 1.),
            b: -alpha * (4. + 3. * t),
        }
    }

    #[inline]
    fn periodicity_tolerance(&self) -> Real
    {
        1e-6
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
}

default_bounds_impl!(QuadRatPer1_1, Bounds::centered_square(3.));
has_child_impl!(QuadRatPer1_1);

impl MarkedPoints for QuadRatPer1_1
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

degree_impl!(QuadRatPer1Lambda, 1, 1);
degree_impl!(QuadRatPer1LambdaParam, 1, 1);
degree_impl!(QuadRatPer1_1, 1, 1);
