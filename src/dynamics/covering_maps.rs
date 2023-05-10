use super::ParameterPlane;
use crate::point_grid::{Bounds, PointGrid};
use crate::types::{ComplexNum, EscapeState, Period, PointInfo};

#[derive(Clone)]
pub struct CoveringMap<C>
where
    C: ParameterPlane + Clone,
{
    base_curve: C,
    covering_map: fn(ComplexNum) -> ComplexNum,
    point_grid: PointGrid,
    compose_parameterizations: bool,
}

impl<C> CoveringMap<C>
where
    C: ParameterPlane + Clone,
{
    #[must_use]
    pub fn new(
        base_curve: C,
        covering_map: fn(ComplexNum) -> ComplexNum,
        point_grid: PointGrid,
    ) -> Self
    {
        Self {
            base_curve,
            covering_map,
            point_grid,
            compose_parameterizations: false,
        }
    }

    #[must_use]
    pub fn without_base_parameterization(
        base_curve: C,
        covering_map: fn(ComplexNum) -> ComplexNum,
        point_grid: PointGrid,
    ) -> Self
    {
        Self {
            base_curve,
            covering_map,
            point_grid,
            compose_parameterizations: true,
        }
    }
}

impl<C> ParameterPlane for CoveringMap<C>
where
    C: ParameterPlane + Clone,
{
    fn point_grid(&self) -> &PointGrid
    {
        &self.point_grid
    }

    fn point_grid_mut(&mut self) -> &mut PointGrid
    {
        &mut self.point_grid
    }

    fn stop_condition(&self, iter: Period, z: ComplexNum) -> EscapeState
    {
        self.base_curve.stop_condition(iter, z)
    }

    fn early_bailout(&self, start: ComplexNum, param: ComplexNum) -> EscapeState
    {
        self.base_curve.early_bailout(start, param)
    }

    fn max_iter(&self) -> Period
    {
        self.base_curve.max_iter()
    }

    fn max_iter_mut(&mut self) -> &mut Period
    {
        self.base_curve.max_iter_mut()
    }

    fn set_max_iter(&mut self, new_max_iter: Period)
    {
        self.base_curve.set_max_iter(new_max_iter);
    }

    fn check_periodicity(
        &self,
        iter: Period,
        z0: ComplexNum,
        z1: ComplexNum,
        base_param: ComplexNum,
    ) -> EscapeState
    {
        self.base_curve.check_periodicity(iter, z0, z1, base_param)
    }

    fn param_map(&self, c: ComplexNum) -> ComplexNum
    {
        if self.compose_parameterizations
        {
            let f = self.covering_map;
            let u = f(c);
            self.base_curve.param_map(u)
        }
        else
        {
            (self.covering_map)(c)
        }
    }

    fn start_point(&self, c: ComplexNum) -> ComplexNum
    {
        self.base_curve.start_point(c)
    }

    fn map(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        self.base_curve.map(z, c)
    }

    fn dynamical_derivative(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        self.base_curve.dynamical_derivative(z, c)
    }

    fn parameter_derivative(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        self.base_curve.parameter_derivative(z, c)
    }

    fn gradient(&self, z: ComplexNum, c: ComplexNum) -> (ComplexNum, ComplexNum)
    {
        self.base_curve.gradient(z, c)
    }

    fn encode_escape_result(&self, state: EscapeState, base_param: ComplexNum) -> PointInfo
    {
        self.base_curve.encode_escape_result(state, base_param)
    }

    fn name(&self) -> String
    {
        format!("Cover over {}", self.base_curve.name())
    }

    fn default_coloring(&self) -> crate::coloring::Coloring
    {
        self.base_curve.default_coloring()
    }

    fn default_julia_bounds(&self, param: ComplexNum) -> Bounds
    {
        self.base_curve.default_julia_bounds(param)
    }
}

pub trait HasDynamicalCovers: super::ParameterPlane + Clone
{
    fn marked_cycle_curve(self, _period: Period) -> CoveringMap<Self>
    {
        let param_map = |c| c;
        let bounds = self.point_grid().clone();

        println!("Marked cycle has not been implemented; falling back to base curve!");
        CoveringMap::new(self, param_map, bounds)
    }
    fn dynatomic_curve(self, _period: Period) -> CoveringMap<Self>
    {
        let param_map = |c| c;
        let bounds = self.point_grid().clone();

        println!("Dynatomic curve has not been implemented; falling back to base curve!");
        CoveringMap::new(self, param_map, bounds)
    }
    fn misiurewicz_curve(self, _preperiod: Period, _period: Period) -> CoveringMap<Self>
    {
        let param_map = |c| c;
        let bounds = self.point_grid().clone();

        println!("Misiurewicz curve has not been implemented; falling back to base curve!");
        CoveringMap::new(self, param_map, bounds)
    }
}
