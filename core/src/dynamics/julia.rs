use crate::dynamics::ParameterPlane;
use crate::macros::basic_plane_impl;
use fractal_common::coloring::{algorithms::ColoringAlgorithm, Coloring};
use fractal_common::point_grid::{Bounds, PointGrid};
use fractal_common::types::{Cplx, EscapeState, ParamList, ParamStack, Period, PointInfo, Real};

#[derive(Clone)]
pub struct JuliaSet<T>
where
    T: ParameterPlane + Clone,
{
    pub point_grid: PointGrid,
    pub max_iter: Period,
    pub min_iter: Period,
    pub parent: T,
    pub meta_params: T::MetaParam,
    pub local_param: T::Param,
    pub parent_selection: Cplx,
}

impl<T> JuliaSet<T>
where
    T: ParameterPlane + Clone,
{
    #[must_use]
    pub fn new(parent: T, parent_selection: Cplx, _max_iter: Period) -> Self
    {
        let local_param = parent.param_map(parent_selection);
        let point_grid = parent
            .point_grid()
            .new_with_same_height(parent.default_julia_bounds(parent_selection, local_param));
        Self {
            point_grid,
            parent: parent.clone(),
            max_iter: parent.max_iter(),
            min_iter: parent.min_iter(),
            meta_params: parent.get_meta_params(),
            local_param,
            parent_selection,
        }
    }
}

impl<T> From<T> for JuliaSet<T>
where
    T: ParameterPlane + Clone,
{
    fn from(parent: T) -> Self
    {
        let parent_selection = parent.default_selection();
        let local_param = parent.param_map(parent_selection);
        let point_grid = parent
            .point_grid()
            .new_with_same_height(parent.default_julia_bounds(parent_selection, local_param));
        Self {
            point_grid,
            parent: parent.clone(),
            max_iter: parent.max_iter(),
            min_iter: parent.min_iter(),
            meta_params: parent.get_meta_params(),
            local_param,
            parent_selection,
        }
    }
}

impl<T> ParameterPlane for JuliaSet<T>
where
    T: ParameterPlane,
{
    type Var = T::Var;
    type Param = T::Param;
    type MetaParam = ParamStack<T::MetaParam, T::Param>;
    type Deriv = T::Deriv;
    type Child = Self;
    basic_plane_impl!();

    #[inline]
    fn map(&self, z: Self::Var, _c: Self::Param) -> Self::Var
    {
        self.parent.map(z, self.local_param)
    }

    #[inline]
    fn dynamical_derivative(&self, z: Self::Var, _c: Self::Param) -> Self::Deriv
    {
        self.parent.dynamical_derivative(z, self.local_param)
    }

    #[inline]
    fn parameter_derivative(&self, z: Self::Var, _c: Self::Param) -> Self::Deriv
    {
        self.parent.parameter_derivative(z, self.local_param)
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, _c: Self::Param) -> (Self::Var, Self::Deriv)
    {
        self.parent.map_and_multiplier(z, self.local_param)
    }

    #[inline]
    fn gradient(&self, z: Self::Var, _c: Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        self.parent.gradient(z, self.local_param)
    }

    #[inline]
    fn min_iter(&self) -> Period
    {
        self.min_iter
    }

    #[inline]
    fn param_map(&self, _z: Cplx) -> Self::Param
    {
        self.local_param
    }

    #[inline]
    fn start_point(&self, point: Cplx, _param: Self::Param) -> Self::Var
    {
        self.parent.dynam_map(point)
    }

    #[inline]
    fn cycle_active_plane(&mut self)
    {
        self.parent.cycle_active_plane();
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
    fn set_meta_param(
        &mut self,
        ParamStack {
            meta_params,
            local_param,
        }: Self::MetaParam,
    )
    {
        self.meta_params = meta_params;
        self.local_param = local_param;
    }

    #[inline]
    fn set_param(&mut self, local_param: Self::Param)
    {
        self.local_param = local_param;
    }

    #[inline]
    fn get_meta_params(&self) -> Self::MetaParam
    {
        ParamStack {
            local_param: self.local_param,
            meta_params: self.meta_params,
        }
    }

    #[inline]
    fn get_param(&self) -> Self::Param
    {
        self.local_param
    }

    // #[inline]
    // fn julia_set(&self, _param: ComplexNum) -> Option<JuliaSet<Self>>
    // {
    //     None
    // }

    #[inline]
    fn degree(&self) -> f64
    {
        self.parent.degree()
    }

    #[inline]
    fn default_bounds(&self) -> Bounds
    {
        self.parent
            .default_julia_bounds(self.parent_selection, self.local_param)
    }

    #[inline]
    fn default_julia_bounds(&self, _point: Cplx, _param: Self::Param) -> Bounds
    {
        self.point_grid.bounds.clone()
    }

    #[inline]
    fn critical_points_child(&self, _param: Self::Param) -> Vec<Self::Var>
    {
        self.parent.critical_points_child(self.local_param)
    }

    #[inline]
    fn critical_points(&self) -> Vec<Self::Var>
    {
        self.parent.critical_points_child(self.local_param)
    }

    #[inline]
    fn cycles_child(&self, _param: Self::Param, period: Period) -> Vec<Self::Var>
    {
        self.parent.cycles_child(self.local_param, period)
    }

    #[inline]
    fn cycles(&self, period: Period) -> Vec<Self::Var>
    {
        self.parent.cycles_child(self.local_param, period)
    }

    #[inline]
    fn name(&self) -> String
    {
        "JuliaSet".to_owned()
    }

    #[inline]
    fn periodicity_tolerance(&self) -> Real
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
        ColoringAlgorithm::InternalPotential {
            periodicity_tolerance: self.periodicity_tolerance(),
        }
    }

    fn preperiod_period_smooth_coloring(&self) -> ColoringAlgorithm
    {
        ColoringAlgorithm::PreperiodPeriodSmooth {
            periodicity_tolerance: self.periodicity_tolerance(),
            fill_rate: 0.015,
        }
    }

    fn is_dynamical(&self) -> bool
    {
        true
    }
}
