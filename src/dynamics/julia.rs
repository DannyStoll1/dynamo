use crate::coloring::{coloring_algorithm::ColoringAlgorithm, Coloring};
use crate::dynamics::ParameterPlane;
use crate::point_grid::{Bounds, PointGrid};
use crate::types::{ComplexNum, ComplexVec, EscapeState, Period, PointInfo, RealNum};

#[derive(Clone)]
pub struct JuliaSet<T>
where
    T: ParameterPlane + Clone,
{
    pub point_grid: PointGrid,
    pub max_iter: Period,
    pub parent: T,
    pub param: T::Param,
}

impl<T> JuliaSet<T>
where
    T: ParameterPlane + Clone,
{
    #[must_use]
    pub fn new(parent: T, param: T::Param, max_iter: Period) -> Self
    {
        let point_grid = parent
            .point_grid()
            .with_same_height(parent.default_julia_bounds(param));
        Self {
            point_grid,
            max_iter,
            parent,
            param,
        }
    }
}

impl<T> From<T> for JuliaSet<T>
where
    T: ParameterPlane + Clone,
{
    fn from(parent: T) -> Self
    {
        let param = T::Param::default();
        let point_grid = parent
            .point_grid()
            .with_same_height(parent.default_julia_bounds(param));
        Self {
            point_grid,
            parent: parent.clone(),
            max_iter: parent.max_iter(),
            param,
        }
    }
}

impl<T> ParameterPlane for JuliaSet<T>
where
    T: ParameterPlane + Clone,
{
    type Var = T::Var;
    type Param = T::Param;
    type Deriv = T::Deriv;

    #[inline]
    fn map(&self, z: Self::Var, c: Self::Param) -> Self::Var
    {
        self.parent.map(z, c)
    }

    #[inline]
    fn dynamical_derivative(&self, z: Self::Var, c: Self::Param) -> Self::Deriv
    {
        self.parent.dynamical_derivative(z, c)
    }

    #[inline]
    fn parameter_derivative(&self, z: Self::Var, c: Self::Param) -> Self::Deriv
    {
        self.parent.parameter_derivative(z, c)
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, c: Self::Param) -> (Self::Var, Self::Deriv) {
        self.parent.map_and_multiplier(z, c)
    }

    #[inline]
    fn gradient(&self, z: Self::Var, c: Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        self.parent.gradient(z, c)
    }

    #[inline]
    fn point_grid(&self) -> &PointGrid
    {
        &self.point_grid
    }

    #[inline]
    fn point_grid_mut(&mut self) -> &mut PointGrid
    {
        &mut self.point_grid
    }

    #[inline]
    fn max_iter(&self) -> Period
    {
        self.max_iter
    }

    #[inline]
    fn max_iter_mut(&mut self) -> &mut Period
    {
        &mut self.max_iter
    }

    #[inline]
    fn param_map(&self, _z: ComplexNum) -> Self::Param
    {
        self.param
    }

    #[inline]
    fn start_point(&self, point: ComplexNum, _param: Self::Param) -> Self::Var
    {
        point.into()
    }

    #[inline]
    fn set_max_iter(&mut self, new_max_iter: Period)
    {
        self.max_iter = new_max_iter;
    }

    fn encode_escape_result(
        &self,
        state: EscapeState<Self::Var, Self::Deriv>,
        base_param: Self::Param,
    ) -> PointInfo<Self::Var, Self::Deriv>
    {
        self.parent.encode_escape_result(state, base_param)
    }

    #[inline]
    fn set_param(&mut self, value: Self::Param)
    {
        self.param = value;
    }

    #[inline]
    fn get_param(&self) -> Self::Param
    {
        self.param
    }

    #[inline]
    fn julia_set(&self, _param: ComplexNum) -> Option<JuliaSet<Self>>
    {
        None
    }

    #[inline]
    fn default_julia_bounds(&self, _param: Self::Param) -> Bounds
    {
        self.point_grid.bounds.clone()
    }

    #[inline]
    fn critical_points(&self, _param: Self::Param) -> Vec<Self::Var>
    {
        self.parent.critical_points(self.param)
    }

    #[inline]
    fn cycles(&self, _param: Self::Param, period: Period) -> Vec<Self::Var>
    {
        self.parent.cycles(self.param, period)
    }

    #[inline]
    fn name(&self) -> String
    {
        "JuliaSet".to_owned()
    }

    #[inline]
    fn periodicity_tolerance(&self) -> RealNum
    {
        self.parent.periodicity_tolerance()
    }

    fn default_coloring(&self) -> Coloring
    {
        let mut coloring = Coloring::default();
        let _periodicity_tolerance = self.periodicity_tolerance();
        coloring.set_algorithm(self.preperiod_smooth_coloring());
        coloring
    }

    fn preperiod_smooth_coloring(&self) -> ColoringAlgorithm
    {
        ColoringAlgorithm::PreperiodSmooth {
            periodicity_tolerance: self.periodicity_tolerance(),
            fill_rate: 0.015,
        }
    }
}
