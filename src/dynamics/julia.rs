use crate::point_grid::*;
use crate::primitive_types::*;
use crate::dynamics::ParameterPlane;

#[derive(Clone)]
pub struct JuliaSet<T>
where
    T: ParameterPlane + Clone,
{
    pub point_grid: PointGrid,
    pub max_iter: Period,
    pub parent: T,
    // pub map: Box<dyn Fn(ComplexNum, ComplexNum) -> ComplexNum>,
    // pub stop_condition: Box<dyn Fn(Period, ComplexNum) -> EscapeState>,
    // pub escape_encoding: Box<dyn Fn(EscapeState, ComplexNum) -> IterCount>,
    pub param: ComplexNum,
    // pub parent_params: Vec<ComplexNum>,
}

impl<T> From<T> for JuliaSet<T>
where
    T: ParameterPlane + Clone,
{
    fn from(parent: T) -> Self {
        let point_grid = parent
            .point_grid()
            .with_same_height(parent.default_julia_bounds());
        Self {
            point_grid,
            parent: parent.clone(),
            max_iter: parent.max_iter(),
            param: (0.).into(),
        }
    }
}

impl<T> ParameterPlane for JuliaSet<T>
where
    T: ParameterPlane + Clone,
{
    #[inline]
    fn map(&self, z: ComplexNum, _c: ComplexNum) -> ComplexNum {
        self.parent.map(z, self.param)
    }

    #[inline]
    fn dynamical_derivative(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum {
        self.parent.dynamical_derivative(z, self.param)
    }

    #[inline]
    fn parameter_derivative(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum {
        self.parent.parameter_derivative(z, self.param)
    }

    #[inline]
    fn gradient(&self, z: ComplexNum, c: ComplexNum) -> (ComplexNum, ComplexNum) {
        self.parent.gradient(z, self.param)
    }

    #[inline]
    fn point_grid(&self) -> PointGrid {
        self.point_grid
    }

    #[inline]
    fn point_grid_mut(&mut self) -> &mut PointGrid {
        &mut self.point_grid
    }

    #[inline]
    fn max_iter(&self) -> Period {
        self.max_iter
    }

    #[inline]
    fn max_iter_mut(&mut self) -> &mut Period {
        &mut self.max_iter
    }

    #[inline]
    fn param_map(&self, z: ComplexNum) -> ComplexNum {
        z
    }

    #[inline]
    fn start_point(&self, z: ComplexNum) -> ComplexNum {
        z
    }

    #[inline]
    fn set_max_iter(&mut self, new_max_iter: Period) {
        self.max_iter = new_max_iter;
    }

    fn encode_escape_result(&self, state: EscapeState, _base_param: ComplexNum) -> IterCount {
        self.parent.encode_escape_result(state, self.param)
    }

    fn stop_condition(&self, iter: Period, z: ComplexNum) -> EscapeState {
        self.parent.stop_condition(iter, z)
    }

    #[inline]
    fn set_param(&mut self, value: ComplexNum) {
        self.param = value;
    }

    #[inline]
    fn get_param(&self) -> ComplexNum {
        self.param
    }

    #[inline]
    fn julia_set(&self, _param: ComplexNum) -> Option<JuliaSet<Self>> {
        None
    }

    #[inline]
    fn default_julia_bounds(&self) -> Bounds {
        self.point_grid.bounds
    }

    #[inline]
    fn name(&self) -> String {
        "JuliaSet".to_owned()
    }
}
