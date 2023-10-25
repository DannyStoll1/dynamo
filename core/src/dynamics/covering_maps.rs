use super::julia::JuliaSet;
use super::orbit::EscapeResult;
use super::{EscapeEncoding, ExternalRays, InfinityFirstReturnMap, ParameterPlane};
use dynamo_common::prelude::*;
use dynamo_common::symbolic_dynamics::OrbitSchema;
use num_traits::One;

#[derive(Clone)]
pub struct CoveringMap<C>
where
    C: ParameterPlane + Clone,
{
    base_curve: C,
    covering_map_d: fn(Cplx) -> (C::Param, C::Deriv),
    point_grid: PointGrid,
    orig_bounds: Bounds,
}

impl<C> CoveringMap<C>
where
    C: ParameterPlane + Clone,
{
    #[must_use]
    pub fn new(
        base_curve: C,
        covering_map_d: fn(Cplx) -> (C::Param, C::Deriv),
        point_grid: PointGrid,
    ) -> Self
    {
        let orig_bounds = point_grid.bounds.clone();
        Self {
            base_curve,
            covering_map_d,
            point_grid,
            orig_bounds,
        }
    }
}

impl<C> ParameterPlane for CoveringMap<C>
where
    C: ParameterPlane + Clone,
{
    type Var = C::Var;
    type Param = C::Param;
    type MetaParam = C::MetaParam;
    type Deriv = C::Deriv;
    type Child = JuliaSet<Self>;

    fn point_grid(&self) -> &PointGrid
    {
        &self.point_grid
    }

    fn point_grid_mut(&mut self) -> &mut PointGrid
    {
        &mut self.point_grid
    }

    fn with_point_grid(mut self, point_grid: PointGrid) -> Self
    {
        self.point_grid = point_grid;
        self
    }

    fn early_bailout(
        &self,
        start: Self::Var,
        param: Self::Param,
    ) -> Option<EscapeResult<C::Var, C::Deriv>>
    {
        self.base_curve.early_bailout(start, param)
    }

    #[inline]
    fn min_iter(&self) -> Period
    {
        self.base_curve.min_iter()
    }

    #[inline]
    fn max_iter(&self) -> Period
    {
        self.base_curve.max_iter()
    }

    #[inline]
    fn max_iter_mut(&mut self) -> &mut Period
    {
        self.base_curve.max_iter_mut()
    }

    #[inline]
    fn set_max_iter(&mut self, new_max_iter: Period)
    {
        self.base_curve.set_max_iter(new_max_iter);
    }

    fn with_max_iter(mut self, max_iter: Period) -> Self
    {
        self.set_max_iter(max_iter);
        self
    }

    fn param_map(&self, t: Cplx) -> C::Param
    {
        (self.covering_map_d)(t).0
    }

    fn param_map_d(&self, t: Cplx) -> (C::Param, C::Deriv)
    {
        (self.covering_map_d)(t)
    }

    #[inline]
    fn start_point(&self, t: Cplx, c: C::Param) -> C::Var
    {
        self.base_curve.start_point(t, c)
    }

    #[inline]
    fn start_point_d(&self, t: Cplx, c: Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        self.base_curve.start_point_d(t, c)
    }

    #[inline]
    fn map(&self, z: Self::Var, c: Self::Param) -> Self::Var
    {
        self.base_curve.map(z, c)
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, c: Self::Param) -> (Self::Var, Self::Deriv)
    {
        self.base_curve.map_and_multiplier(z, c)
    }

    #[inline]
    fn gradient(&self, z: Self::Var, c: Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        self.base_curve.gradient(z, c)
    }

    #[inline]
    fn get_meta_params(&self) -> Self::MetaParam
    {
        self.base_curve.get_meta_params()
    }

    #[inline]
    fn set_meta_param(&mut self, value: Self::MetaParam)
    {
        self.base_curve.set_meta_param(value);
    }

    #[inline]
    fn name(&self) -> String
    {
        format!("Cover over {}", self.base_curve.name())
    }

    #[inline]
    fn default_bounds(&self) -> Bounds
    {
        self.orig_bounds.clone()
    }

    #[inline]
    fn plane_type(&self) -> super::PlaneType
    {
        self.base_curve.plane_type()
    }

    fn description(&self) -> String
    {
        format!(
            "A dynamical cover over {}. Description of base curve: \n{}",
            self.base_curve.name(),
            self.base_curve.description()
        )
    }

    #[inline]
    fn default_coloring(&self) -> dynamo_common::coloring::Coloring
    {
        self.base_curve.default_coloring()
    }

    #[inline]
    fn critical_points_child(&self, param: C::Param) -> Vec<Self::Var>
    {
        self.base_curve.critical_points_child(param)
    }

    #[inline]
    fn cycles_child(&self, param: C::Param, period: Period) -> Vec<Self::Var>
    {
        self.base_curve.cycles_child(param, period)
    }

    #[inline]
    fn precycles_child(&self, c: Self::Param, orbit_schema: OrbitSchema) -> Vec<Self::Var>
    {
        self.base_curve.precycles_child(c, orbit_schema)
    }

    #[inline]
    fn default_julia_bounds(&self, point: Cplx, param: C::Param) -> Bounds
    {
        self.base_curve.default_julia_bounds(point, param)
    }

    #[inline]
    fn periodicity_tolerance(&self) -> Real
    {
        self.base_curve.periodicity_tolerance()
    }

    #[inline]
    fn set_param(&mut self, value: <Self::MetaParam as ParamList>::Param)
    {
        self.base_curve.set_param(value);
    }

    #[inline]
    fn get_param(&self) -> <Self::MetaParam as ParamList>::Param
    {
        self.base_curve.get_param()
    }

    #[inline]
    fn cycle_active_plane(&mut self)
    {
        self.base_curve.cycle_active_plane();
    }
}

impl<C> InfinityFirstReturnMap for CoveringMap<C>
where
    C: ParameterPlane + InfinityFirstReturnMap + Clone,
{
    #[inline]
    fn degree_real(&self) -> f64
    {
        self.base_curve.degree_real()
    }

    #[inline]
    fn degree(&self) -> AngleNum
    {
        self.base_curve.degree()
    }

    #[inline]
    fn escaping_period(&self) -> Period
    {
        self.base_curve.escaping_period()
    }

    #[inline]
    fn escaping_phase(&self) -> Period
    {
        self.base_curve.escaping_phase()
    }

    #[inline]
    fn escape_coeff_d(&self, c: Self::Param) -> (Cplx, Cplx)
    {
        self.base_curve.escape_coeff_d(c)
    }
}

impl<C: EscapeEncoding + Clone> EscapeEncoding for CoveringMap<C>
{
    #[inline]
    fn encode_escape_result(
        &self,
        state: EscapeResult<C::Var, C::Deriv>,
        base_param: C::Param,
    ) -> PointInfo<C::Deriv>
    {
        self.base_curve.encode_escape_result(state, base_param)
    }
}

pub trait HasDynamicalCovers: super::ParameterPlane + Clone
{
    fn marked_cycle_curve(self, _period: Period) -> CoveringMap<Self>
    {
        let param_map_d = |t| (Self::Param::from(t), Self::Deriv::one());
        let bounds = self.point_grid().clone();

        println!("Marked cycle has not been implemented; falling back to base curve!");
        CoveringMap::new(self, param_map_d, bounds)
    }
    fn dynatomic_curve(self, _period: Period) -> CoveringMap<Self>
    {
        let param_map_d = |t| (Self::Param::from(t), Self::Deriv::one());
        let bounds = self.point_grid().clone();

        println!("Dynatomic curve has not been implemented; falling back to base curve!");
        CoveringMap::new(self, param_map_d, bounds)
    }
    fn misiurewicz_curve(self, _preperiod: Period, _period: Period) -> CoveringMap<Self>
    {
        let param_map = |t| (Self::Param::from(t), Self::Deriv::one());
        let bounds = self.point_grid().clone();

        println!("Misiurewicz curve has not been implemented; falling back to base curve!");
        CoveringMap::new(self, param_map, bounds)
    }
}

impl<C: ExternalRays + Clone> ExternalRays for CoveringMap<C> {}
