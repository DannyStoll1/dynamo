use crate::dynamics::ParameterPlane;
use crate::point_grid::{Bounds, PointGrid};
use crate::primitive_types::*;

#[derive(Clone)]
pub struct JuliaSet<T>
where
    T: ParameterPlane + Clone,
{
    pub point_grid: PointGrid,
    pub max_iter: Period,
    pub parent: T,
    pub param: ComplexNum,
}

impl<T> JuliaSet<T>
where
    T: ParameterPlane + Clone,
{
    #[must_use]
    pub fn new(parent: T, param: ComplexNum) -> Self
    {
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

impl<T> From<T> for JuliaSet<T>
where
    T: ParameterPlane + Clone,
{
    fn from(parent: T) -> Self
    {
        let param = ComplexNum::new(0., 0.);
        let point_grid = parent
            .point_grid()
            .with_same_height(parent.default_julia_bounds(param));
        let periodicity_tolerance = parent.periodicity_tolerance();
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
    #[inline]
    fn map(&self, z: ComplexNum, _c: ComplexNum) -> ComplexNum
    {
        self.parent.map(z, self.param)
    }

    #[inline]
    fn dynamical_derivative(&self, z: ComplexNum, _c: ComplexNum) -> ComplexNum
    {
        self.parent.dynamical_derivative(z, self.param)
    }

    #[inline]
    fn parameter_derivative(&self, z: ComplexNum, _c: ComplexNum) -> ComplexNum
    {
        self.parent.parameter_derivative(z, self.param)
    }

    #[inline]
    fn gradient(&self, z: ComplexNum, _c: ComplexNum) -> (ComplexNum, ComplexNum)
    {
        self.parent.gradient(z, self.param)
    }

    #[inline]
    fn point_grid(&self) -> PointGrid
    {
        self.point_grid
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
    fn param_map(&self, z: ComplexNum) -> ComplexNum
    {
        z
    }

    #[inline]
    fn start_point(&self, z: ComplexNum) -> ComplexNum
    {
        z
    }

    #[inline]
    fn set_max_iter(&mut self, new_max_iter: Period)
    {
        self.max_iter = new_max_iter;
    }

    fn encode_escape_result(&self, state: EscapeState, base_param: ComplexNum) -> PointInfo
    {
        self.parent.encode_escape_result(state, base_param)
    }

    #[inline]
    fn set_param(&mut self, value: ComplexNum)
    {
        self.param = value;
    }

    #[inline]
    fn get_param(&self) -> ComplexNum
    {
        self.param
    }

    #[inline]
    fn julia_set(&self, _param: ComplexNum) -> Option<JuliaSet<Self>>
    {
        None
    }

    #[inline]
    fn default_julia_bounds(&self, _param: ComplexNum) -> Bounds
    {
        self.point_grid.bounds
    }

    #[inline]
    fn name(&self) -> String
    {
        "JuliaSet".to_owned()
    }

    #[inline]
    fn periodicity_tolerance(&self) -> RealNum {
        self.parent.periodicity_tolerance()
    }
}
