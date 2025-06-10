use super::julia::JuliaSet;
use super::{
    DynamicalFamily, EscapeEncoding, ExternalRays, FamilyDefaults, HasChild, HasJulia,
    InfinityFirstReturnMap, MarkedPoints,
};
use crate::orbit::EscapeResult;
use dynamo_color::{Coloring, IncoloringAlgorithm};
use dynamo_common::prelude::*;
use dynamo_common::symbolic_dynamics::OrbitSchema;
use num_traits::One;

#[derive(Clone)]
pub struct CoveringMap<C>
where
    C: DynamicalFamily,
{
    base_curve: C,
    covering_map_d: fn(Cplx) -> (C::Param, C::Deriv),
    point_grid: PointGrid,
    orig_bounds: Bounds,
    multiplier_map: fn(Cplx) -> (Cplx, Cplx),
    marked_points: Vec<Cplx>,
}

impl<C> CoveringMap<C>
where
    C: DynamicalFamily,
{
    #[must_use]
    pub fn new(base_curve: C, covering_map_d: fn(Cplx) -> (C::Param, C::Deriv)) -> Self
    {
        let point_grid = base_curve.point_grid().clone();
        let orig_bounds = point_grid.bounds.clone();
        Self {
            base_curve,
            covering_map_d,
            point_grid,
            orig_bounds,
            multiplier_map: |t| (t, ONE),
            marked_points: Vec::new(),
        }
    }
    #[must_use]
    pub fn with_orig_bounds(mut self, bounds: Bounds) -> Self
    {
        self.orig_bounds = bounds.clone();
        self.with_bounds(bounds)
    }
    #[must_use]
    pub fn with_multiplier_map(mut self, multiplier_map: fn(Cplx) -> (Cplx, Cplx)) -> Self
    {
        self.multiplier_map = multiplier_map;
        self
    }
    #[must_use]
    pub fn with_marked_points(mut self, marked_points: Vec<Cplx>) -> Self
    {
        self.marked_points = marked_points;
        self
    }
}

impl<C> From<C> for CoveringMap<C>
where
    C: DynamicalFamily,
    C::Param: From<Cplx>,
{
    fn from(base_curve: C) -> Self
    {
        let covering_map_d = |t: Cplx| (t.into(), C::Deriv::one());
        Self::new(base_curve, covering_map_d)
    }
}

impl<C> DynamicalFamily for CoveringMap<C>
where
    C: DynamicalFamily,
{
    type Var = C::Var;
    type Param = C::Param;
    type MetaParam = C::MetaParam;
    type Deriv = C::Deriv;

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

    fn compute_mode(&self) -> super::ComputeMode
    {
        self.base_curve.compute_mode()
    }

    fn compute_mode_mut(&mut self) -> &mut super::ComputeMode
    {
        self.base_curve.compute_mode_mut()
    }

    fn set_compute_mode(&mut self, compute_mode: super::ComputeMode)
    {
        self.base_curve.set_compute_mode(compute_mode);
    }

    fn early_bailout(&self, start: Self::Var, param: &Self::Param) -> Option<PointInfo<C::Deriv>>
    {
        self.base_curve.early_bailout(start, param)
    }

    #[inline]
    fn min_iter(&self) -> IterCount
    {
        self.base_curve.min_iter()
    }

    #[inline]
    fn max_iter(&self) -> IterCount
    {
        self.base_curve.max_iter()
    }

    #[inline]
    fn max_iter_mut(&mut self) -> &mut IterCount
    {
        self.base_curve.max_iter_mut()
    }

    #[inline]
    fn set_max_iter(&mut self, new_max_iter: IterCount)
    {
        self.base_curve.set_max_iter(new_max_iter);
    }

    fn with_max_iter(mut self, max_iter: IterCount) -> Self
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
    fn start_point(&self, t: Cplx, c: &C::Param) -> C::Var
    {
        self.base_curve.start_point(t, c)
    }

    #[inline]
    fn start_point_d(&self, t: Cplx, c: &Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        self.base_curve.start_point_d(t, c)
    }

    #[inline]
    fn map(&self, z: Self::Var, c: &Self::Param) -> Self::Var
    {
        self.base_curve.map(z, c)
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv)
    {
        self.base_curve.map_and_multiplier(z, c)
    }

    #[inline]
    fn gradient(&self, z: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
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

    #[inline]
    fn auxiliary_value(&self, t: Cplx) -> Option<(Cplx, Cplx)>
    {
        Some((self.multiplier_map)(t))
    }
}

impl<C> FamilyDefaults for CoveringMap<C>
where
    C: FamilyDefaults,
{
    #[inline]
    fn default_bounds(&self) -> Bounds
    {
        self.orig_bounds.clone()
    }

    #[inline]
    fn default_coloring(&self) -> Coloring
    {
        self.base_curve.default_coloring()
    }
}

impl<C: HasJulia> HasJulia for CoveringMap<C>
{
    #[inline]
    fn default_bounds_child(&self, t: Cplx, c: &Self::Param) -> Bounds
    {
        self.base_curve.default_bounds_child(t, c)
    }

    #[inline]
    fn default_max_iter_child(&self) -> IterCount
    {
        self.base_curve.default_max_iter_child()
    }

    #[inline]
    fn default_coloring_child(&self) -> Coloring
    {
        self.base_curve.default_coloring_child()
    }
}

impl<C> MarkedPoints for CoveringMap<C>
where
    C: MarkedPoints,
{
    #[inline]
    fn critical_points_child(&self, param: &C::Param) -> Vec<Self::Var>
    {
        self.base_curve.critical_points_child(param)
    }

    #[inline]
    fn cycles_child(&self, param: &C::Param, period: Period) -> Vec<Self::Var>
    {
        self.base_curve.cycles_child(param, period)
    }

    #[inline]
    fn precycles_child(&self, c: &C::Param, orbit_schema: OrbitSchema) -> Vec<Self::Var>
    {
        self.base_curve.precycles_child(c, orbit_schema)
    }

    #[inline]
    fn other_marked_points(&self) -> Vec<Cplx>
    {
        self.marked_points.clone()
    }
}

impl<C> InfinityFirstReturnMap for CoveringMap<C>
where
    C: DynamicalFamily + InfinityFirstReturnMap,
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
    fn escape_coeff_d(&self, c: &Self::Param) -> (Cplx, Cplx)
    {
        self.base_curve.escape_coeff_d(c)
    }
}

impl<C: EscapeEncoding> EscapeEncoding for CoveringMap<C>
{
    #[inline]
    fn encode_escape_result(
        &self,
        state: EscapeResult<C::Var, C::Deriv>,
        start: C::Var,
        base_param: &C::Param,
    ) -> PointInfo<C::Deriv>
    {
        self.base_curve
            .encode_escape_result(state, start, base_param)
    }
}

pub trait HasDynamicalCovers: super::DynamicalFamily + Sized
{
    fn marked_cycle_curve(self, _period: Period) -> CoveringMap<Self>
    {
        let param_map_d = |_| (Self::Param::default(), Self::Deriv::one());
        let bounds = self.point_grid().bounds.clone();

        warn!("Marked cycle has not been implemented; falling back to base curve!");
        CoveringMap::new(self, param_map_d).with_orig_bounds(bounds)
    }
    fn dynatomic_curve(self, _period: Period) -> CoveringMap<Self>
    {
        let param_map_d = |_| (Self::Param::default(), Self::Deriv::one());
        let bounds = self.point_grid().bounds.clone();

        warn!("Dynatomic curve has not been implemented; falling back to base curve!");
        CoveringMap::new(self, param_map_d).with_orig_bounds(bounds)
    }
    fn misiurewicz_curve(self, _preperiod: Period, _period: Period) -> CoveringMap<Self>
    {
        let param_map_d = |_| (Self::Param::default(), Self::Deriv::one());
        let bounds = self.point_grid().bounds.clone();

        warn!("Misiurewicz curve has not been implemented; falling back to base curve!");
        CoveringMap::new(self, param_map_d).with_orig_bounds(bounds)
    }
}

impl<C: ExternalRays> ExternalRays for CoveringMap<C> {}
