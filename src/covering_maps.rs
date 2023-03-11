use crate::point_grid::PointGrid;
use crate::primitive_types::*;
use crate::traits::ParameterPlane;

#[derive(Clone, Copy)]
pub struct CoveringMap<C>
where
    C: ParameterPlane,
{
    base_curve: C,
    covering_map: fn(ComplexNum) -> ComplexNum,
    point_grid: PointGrid,
}

impl<C> CoveringMap<C>
where
    C: ParameterPlane,
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
    C: ParameterPlane,
{
    fn point_grid(&self) -> PointGrid {
        self.point_grid
    }

    fn point_grid_mut(&mut self) -> &mut PointGrid {
        &mut self.point_grid
    }

    fn point_grid_child(&self) -> PointGrid {
        self.base_curve.point_grid_child()
    }

    fn point_grid_child_mut(&mut self) -> &mut PointGrid {
        self.base_curve.point_grid_child_mut()
    }

    fn stop_condition(&self, iter: Period, z: ComplexNum) -> EscapeState {
        self.base_curve.stop_condition(iter, z)
    }

    fn check_periodicity(&self, iter: Period, z0: ComplexNum, z1: ComplexNum, base_param: ComplexNum) -> EscapeState {
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

    fn encode_escape_result(&self, state: EscapeState, base_param: ComplexNum) -> f64 {
        self.base_curve
            .encode_escape_result(state, base_param)
    }

    fn name(&self) -> String {
        format!("Cover over {}", self.base_curve.name())
    }
}
