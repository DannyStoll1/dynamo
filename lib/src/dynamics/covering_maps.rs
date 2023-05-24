use super::ParameterPlane;
use super::julia::JuliaSet;
use crate::point_grid::{Bounds, PointGrid};
use crate::types::{ComplexNum, ComplexVec, EscapeState, Period, PointInfo};

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

    fn early_bailout(
        &self,
        start: Self::Var,
        param: Self::Param,
    ) -> EscapeState<Self::Var, Self::Deriv>
    {
        self.base_curve.early_bailout(start, param)
    }

    fn min_iter(&self) -> Period
    {
        self.base_curve.min_iter()
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

    fn param_map(&self, c: ComplexNum) -> C::Param
    {
        if self.compose_parameterizations
        {
            let u = (self.covering_map)(c);
            self.base_curve.param_map(u)
        }
        else
        {
            (self.covering_map)(c).into()
        }
    }

    fn start_point(&self, p: ComplexNum, c: C::Param) -> C::Var
    {
        self.base_curve.start_point(p, c)
    }

    fn map(&self, z: Self::Var, c: Self::Param) -> Self::Var
    {
        self.base_curve.map(z, c)
    }

    fn map_and_multiplier(&self, z: Self::Var, c: Self::Param) -> (Self::Var, Self::Deriv)
    {
        self.base_curve.map_and_multiplier(z, c)
    }

    fn dynamical_derivative(&self, z: Self::Var, c: Self::Param) -> Self::Deriv
    {
        self.base_curve.dynamical_derivative(z, c)
    }

    fn parameter_derivative(&self, z: Self::Var, c: Self::Param) -> Self::Deriv
    {
        self.base_curve.parameter_derivative(z, c)
    }

    fn gradient(&self, z: Self::Var, c: Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        self.base_curve.gradient(z, c)
    }

    fn get_param(&self) -> Self::MetaParam
    {
        self.base_curve.get_param()
    }

    fn set_meta_param(&mut self, value: Self::MetaParam)
    {
        self.base_curve.set_meta_param(value)
    }

    fn encode_escape_result(
        &self,
        state: EscapeState<C::Var, C::Deriv>,
        base_param: C::Param,
    ) -> PointInfo<C::Deriv>
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

    fn critical_points_child(&self, param: C::Param) -> Vec<Self::Var>
    {
        self.base_curve.critical_points_child(param)
    }

    fn cycles_child(&self, param: C::Param, period: Period) -> Vec<Self::Var>
    {
        self.base_curve.cycles_child(param, period)
    }

    fn default_julia_bounds(&self, point: ComplexNum, param: C::Param) -> Bounds
    {
        self.base_curve.default_julia_bounds(point, param)
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
