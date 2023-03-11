use crate::point_grid::PointGrid;
use crate::primitive_types::*;
use crate::traits::DynamicalPlane;

pub struct JuliaSet {
    point_grid: PointGrid,
    map: Box<dyn Fn(ComplexNum) -> ComplexNum>,
    stop_condition: Box<dyn Fn(Period, ComplexNum) -> EscapeState>,
    check_periodicity: Box<dyn Fn(Period, ComplexNum, ComplexNum) -> EscapeState>,
    escape_encoding: Box<dyn Fn(EscapeState) -> f64>,
}

impl JuliaSet {
    pub fn new(
        point_grid: PointGrid,
        map: Box<dyn Fn(ComplexNum) -> ComplexNum>,
        stop_condition: Box<dyn Fn(Period, ComplexNum) -> EscapeState>,
        check_periodicity: Box<dyn Fn(Period, ComplexNum, ComplexNum) -> EscapeState>,
        escape_encoding: Box<dyn Fn(EscapeState) -> f64>,
    ) -> Self {
        Self {
            point_grid,
            map,
            stop_condition,
            check_periodicity,
            escape_encoding,
        }
    }
}

impl DynamicalPlane for JuliaSet {
    fn map(&self, z: ComplexNum) -> ComplexNum {
        (self.map)(z)
    }

    fn point_grid(&self) -> PointGrid {
        self.point_grid
    }

    fn encode_escape_result(&self, state: EscapeState) -> f64 {
        (self.escape_encoding)(state)
    }

    fn stop_condition(&self, iter: Period, z: ComplexNum) -> EscapeState {
        (self.stop_condition)(iter, z)
    }

    fn check_periodicity(
        &self,
        iter: Period,
        z0: ComplexNum,
        z1: ComplexNum,
    ) -> EscapeState {
        (self.check_periodicity)(iter, z0, z1)
    }

    fn name(&self) -> String {
        "JuliaSet".to_owned()
    }
}
