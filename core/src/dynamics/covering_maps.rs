use super::julia::JuliaSet;
use super::ParameterPlane;
use fractal_common::point_grid::{Bounds, PointGrid};
use fractal_common::types::param_stack::ParamList;
use fractal_common::types::*;

#[derive(Clone)]
pub struct CoveringMap<C>
where
    C: ParameterPlane + Clone,
{
    base_curve: C,
    covering_map: fn(Cplx) -> Cplx,
    point_grid: PointGrid,
    compose_parameterizations: bool,
}

impl<C> CoveringMap<C>
where
    C: ParameterPlane + Clone,
{
    #[must_use]
    pub fn new(base_curve: C, covering_map: fn(Cplx) -> Cplx, point_grid: PointGrid) -> Self
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
        covering_map: fn(Cplx) -> Cplx,
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

    fn with_point_grid(mut self, point_grid: PointGrid) -> Self
    {
        self.point_grid = point_grid;
        self
    }

    fn early_bailout(
        &self,
        start: Self::Var,
        param: Self::Param,
    ) -> EscapeState<Self::Var, Self::Deriv>
    {
        self.base_curve.early_bailout(start, param)
    }

    #[inline]
    fn degree(&self) -> f64
    {
        self.base_curve.degree()
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

    fn param_map(&self, c: Cplx) -> C::Param
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

    #[inline]
    fn start_point(&self, p: Cplx, c: C::Param) -> C::Var
    {
        self.base_curve.start_point(p, c)
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
    fn dynamical_derivative(&self, z: Self::Var, c: Self::Param) -> Self::Deriv
    {
        self.base_curve.dynamical_derivative(z, c)
    }

    #[inline]
    fn parameter_derivative(&self, z: Self::Var, c: Self::Param) -> Self::Deriv
    {
        self.base_curve.parameter_derivative(z, c)
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
    fn encode_escape_result(
        &self,
        state: EscapeState<C::Var, C::Deriv>,
        base_param: C::Param,
    ) -> PointInfo<C::Var, C::Deriv>
    {
        self.base_curve.encode_escape_result(state, base_param)
    }

    #[inline]
    fn name(&self) -> String
    {
        format!("Cover over {}", self.base_curve.name())
    }

    #[inline]
    fn default_coloring(&self) -> fractal_common::coloring::Coloring
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
    fn precycles_child(&self, c: Self::Param, preperiod: Period, period: Period) -> Vec<Self::Var>
    {
        self.base_curve.precycles_child(c, preperiod, period)
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
