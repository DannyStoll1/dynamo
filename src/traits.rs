use crate::covering_maps::CoveringMap;
use crate::palette::ColorPalette;
use crate::point_grid::PointGrid;
use crate::primitive_types::{ComplexNum, EscapeState, Period};
use ndarray::Array2;

pub trait FractalImage {
    fn point_grid(&self) -> PointGrid;
    fn draw(&self, palette: ColorPalette, filename: &str);
}

pub trait ParameterPlane: Copy {
    const NUM_TRACKED_POINTS: usize;

    fn point_grid(&self) -> PointGrid;

    fn stop_condition(&self, iter: i32, z: ComplexNum) -> EscapeState;

    fn start_point(&self, c: ComplexNum) -> ComplexNum {
        c
    }

    fn param_map(&self, c: ComplexNum) -> ComplexNum {
        c
    }

    fn map(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum;

    fn encode_escape_result(&self, iter: i32, state: EscapeState, base_param: ComplexNum) -> f64;

    fn compute_escape_times(&self) -> Array2<f64> {
        let mut iter_counts = Array2::zeros((self.point_grid().res_x, self.point_grid().res_y));
        for ((x, y), point) in self.point_grid().iter() {
            let c = self.param_map(point);
            let mut z = self.start_point(c);
            let mut iter = 0;
            while let EscapeState::NotYetEscaped = self.stop_condition(iter, z) {
                z = self.map(z, c);
                iter += 1;
            }

            let result = self.stop_condition(iter, z);

            let iter_count = self.encode_escape_result(iter, result, c);
            iter_counts[(x, y)] = iter_count;
        }
        iter_counts
    }

    fn compute(&self) -> IterPlane {
        let iter_counts = self.compute_escape_times();
        IterPlane {
            iter_counts,
            point_grid: self.point_grid(),
        }
    }

    fn to_cover(self, covering_map: fn(ComplexNum) -> ComplexNum) -> CoveringMap<Self> {
        CoveringMap::new(self, covering_map, self.point_grid())
    }
}

pub struct IterPlane {
    iter_counts: Array2<f64>,
    point_grid: PointGrid,
}

impl FractalImage for IterPlane {
    fn point_grid(&self) -> PointGrid {
        self.point_grid
    }
    fn draw(&self, palette: ColorPalette, filename: &str) {
        let mut image = image::ImageBuffer::new(
            self.point_grid().res_x as u32,
            self.point_grid().res_y as u32,
        );

        for (x, y, pixel) in image.enumerate_pixels_mut() {
            let iter_count = self.iter_counts[(x as usize, y as usize)];
            *pixel = palette.color_map(iter_count);
        }
        image.save(filename).unwrap();
    }
}
