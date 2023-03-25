use crate::dynamics::ParameterPlane;
use crate::point_grid::*;
use crate::primitive_types::*;

#[derive(Clone, Copy)]
pub struct CoveringMap<C>
where
    C: ParameterPlane + Clone,
{
    base_curve: C,
    covering_map: fn(ComplexNum) -> ComplexNum,
    point_grid: PointGrid,
}

impl<C> CoveringMap<C>
where
    C: ParameterPlane + Clone,
{
    pub fn new(
        base_curve: C,
        covering_map: fn(ComplexNum) -> ComplexNum,
        point_grid: PointGrid,
    ) -> Self {
        Self {
            base_curve,
            covering_map,
            point_grid,
        }
    }
}

impl<C> ParameterPlane for CoveringMap<C>
where
    C: ParameterPlane + Clone,
{
    fn point_grid(&self) -> PointGrid {
        self.point_grid
    }

    fn point_grid_mut(&mut self) -> &mut PointGrid {
        &mut self.point_grid
    }

    fn stop_condition(&self, iter: Period, z: ComplexNum) -> EscapeState {
        self.base_curve.stop_condition(iter, z)
    }

    fn max_iter(&self) -> Period {
        self.base_curve.max_iter()
    }

    fn max_iter_mut(&mut self) -> &mut Period {
        self.base_curve.max_iter_mut()
    }

    fn set_max_iter(&mut self, new_max_iter: Period) {
        self.base_curve.set_max_iter(new_max_iter);
    }

    fn check_periodicity(
        &self,
        iter: Period,
        z0: ComplexNum,
        z1: ComplexNum,
        base_param: ComplexNum,
    ) -> EscapeState {
        self.base_curve.check_periodicity(iter, z0, z1, base_param)
    }

    fn param_map(&self, c: ComplexNum) -> ComplexNum {
        let f = self.covering_map;
        let u = f(c);
        self.base_curve.param_map(u)
    }

    fn start_point(&self, c: ComplexNum) -> ComplexNum {
        self.base_curve.start_point(c)
    }

    fn map(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum {
        self.base_curve.map(z, c)
    }

    fn dynamical_derivative(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum {
        self.base_curve.dynamical_derivative(z, c)
    }

    fn parameter_derivative(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum {
        self.base_curve.parameter_derivative(z, c)
    }

    fn gradient(&self, z: ComplexNum, c: ComplexNum) -> (ComplexNum, ComplexNum) {
        self.base_curve.gradient(z, c)
    }

    fn encode_escape_result(&self, state: EscapeState, base_param: ComplexNum) -> IterCount {
        self.base_curve.encode_escape_result(state, base_param)
    }

    fn name(&self) -> String {
        format!("Cover over {}", self.base_curve.name())
    }

    fn default_julia_bounds(&self) -> Bounds {
        self.base_curve.default_julia_bounds()
    }
}
