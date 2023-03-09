use crate::point_grid::PointGrid;
use crate::primitive_types::*;
use crate::traits::DynamicalPlane;

pub struct JuliaSet {
    point_grid: PointGrid,
    map: Box<dyn Fn(ComplexNum) -> ComplexNum>,
    stop_condition: Box<dyn Fn(i32, ComplexNum) -> EscapeState>,
    escape_encoding: Box<dyn Fn(i32, EscapeState) -> f64>,
}

impl JuliaSet {
    pub fn new(
        point_grid: PointGrid,
        map: Box<dyn Fn(ComplexNum) -> ComplexNum>,
        stop_condition: Box<dyn Fn(i32, ComplexNum) -> EscapeState>,
        escape_encoding: Box<dyn Fn(i32, EscapeState) -> f64>,
    ) -> Self {
        Self {
            point_grid,
            map,
            stop_condition,
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

    fn encode_escape_result(&self, iter: i32, state: EscapeState) -> f64 {
        (self.escape_encoding)(iter, state)
    }

    fn stop_condition(&self, iter: i32, z: ComplexNum) -> EscapeState {
        (self.stop_condition)(iter, z)
    }

    fn name(&self) -> String {
        "JuliaSet".to_owned()
    }
}
