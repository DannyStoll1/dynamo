use crate::point_grid::PointGrid;
use crate::primitive_types::{ComplexNum, EscapeState, Period};
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
    const NUM_TRACKED_POINTS: usize = 1;

    fn stop_condition(&self, iter: i32, z: ComplexNum) -> EscapeState {
        self.base_curve.stop_condition(iter, z)
    }

    fn param_map(&self, c: ComplexNum) -> ComplexNum {
        let f = self.covering_map;
        f(c)
    }

    fn start_point(&self, c: ComplexNum) -> ComplexNum {
        self.base_curve.start_point(c)
    }

    fn map(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum {
        self.base_curve.map(z, c)
    }

    fn encode_escape_result(&self, iter: i32, state: EscapeState, base_param: ComplexNum) -> f64 {
        self.base_curve
            .encode_escape_result(iter, state, base_param)
    }
}
