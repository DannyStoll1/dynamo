use fractal_common::point_grid::*;
use crate::primitive_types::*;

pub trait DynamicalPlane {
    fn point_grid(&self) -> PointGrid;

    fn stop_condition(&self, iter: Period, z: ComplexNum) -> EscapeState;

    fn check_periodicity(&self, iter: Period, z0: ComplexNum, z1: ComplexNum) -> EscapeState;

    fn param_map(&self, z: ComplexNum) -> ComplexNum {
        z
    }

    fn map(&self, z: ComplexNum) -> ComplexNum;

    fn encode_escape_result(&self, state: EscapeState) -> IterCount;

    fn compute_point(&self, start: ComplexNum) -> EscapeState {
        let mut tortoise = start;
        let mut hare = start;
        let mut iter = 0;
        while let EscapeState::NotYetEscaped = self.check_periodicity(iter, tortoise, hare) {
            tortoise = self.map(tortoise);
            hare = self.map(hare);

            // Check if we have escaped halfway through
            let mid_result = self.stop_condition(iter, hare);
            if let EscapeState::Escaped { iters, final_value } = mid_result {
                return EscapeState::Escaped {
                    iters: 2 * iters + 1,
                    final_value,
                };
            }

            hare = self.map(hare);
            iter += 1;
        }

        self.check_periodicity(iter, tortoise, hare)
    }

    fn compute_escape_times(&self) -> Array2<IterCount> {
        let mut iter_counts = Array2::zeros((self.point_grid().res_x, self.point_grid().res_y));
        iter_counts.indexed_iter_mut().for_each(|((x, y), count)| {
            let start = self.point_grid().map_pixel(x, y);
            let result = self.compute_point(start);
            *count = self.encode_escape_result(result);
        });
        iter_counts
    }

    fn compute(&self) -> IterPlane {
        let iter_counts = self.compute_escape_times();
        IterPlane {
            iter_counts,
            point_grid: self.point_grid(),
        }
    }

    fn name(&self) -> String;
}
