use crate::primitive_types::ComplexNum;
use ndarray::Array2;
use rayon::iter::{IterBridge, ParallelBridge};

#[derive(Clone, Copy, Debug)]
pub struct PointGrid {
    pub res_x: usize,
    pub res_y: usize,
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
}

impl PointGrid {
    pub fn new(
        res_x: usize,
        res_y: usize,
        min_x: f64,
        max_x: f64,
        min_y: f64,
        max_y: f64,
    ) -> PointGrid {
        Self {
            res_x,
            res_y,
            min_x,
            max_x,
            min_y,
            max_y,
        }
    }

    pub fn infer_height(res_x: usize, min_x: f64, max_x: f64, min_y: f64, max_y: f64) -> usize {
        debug_assert!(max_x > min_x);
        debug_assert!(max_y > min_y);
        debug_assert!(res_x > 0);

        let res_x_float = res_x as f64;
        let res_y_float = res_x_float * (max_y - min_y) / (max_x - min_x);
        res_y_float as usize
    }

    pub fn infer_width(res_y: usize, min_x: f64, max_x: f64, min_y: f64, max_y: f64) -> usize {
        debug_assert!(max_x > min_x);
        debug_assert!(max_y > min_y);
        debug_assert!(res_y > 0);

        let res_y_float = res_y as f64;
        let res_x_float = res_y_float * (max_x - min_x) / (max_y - min_y);
        res_x_float as usize
    }

    pub fn new_infer(res_x: usize, min_x: f64, max_x: f64, min_y: f64, max_y: f64) -> Self {
        let res_y = Self::infer_height(res_x, min_x, max_x, min_y, max_y);

        Self::new(res_x, res_y, min_x, max_x, min_y, max_y)
    }

    pub fn with_new_bounds(&self, min_x: f64, max_x: f64, min_y: f64, max_y: f64) -> Self {
        Self::new_infer(self.res_x, min_x, max_x, min_y, max_y)
    }

    pub fn with_new_size(&self, res_x: usize) -> Self {
        Self::new_infer(res_x, self.min_x, self.max_x, self.min_y, self.max_y)
    }

    pub fn map_pixel(&self, pixel_x: usize, pixel_y: usize) -> ComplexNum {
        let re = self.min_x + (pixel_x as f64) * (self.max_x - self.min_x) / (self.res_x as f64);
        let im = self.min_y + (pixel_y as f64) * (self.max_y - self.min_y) / (self.res_y as f64);
        ComplexNum::new(re, im)
    }

    pub fn locate_point(&self, z: ComplexNum) -> Option<(usize, usize)> {
        if z.re >= self.max_x || z.re < self.min_x || z.im >= self.max_y || z.re < self.min_y {
            return None;
        }

        let x = (z.re - self.min_x) / (self.max_x - self.min_x);
        let y = (z.im - self.min_y) / (self.max_y - self.min_y);

        Some((x as usize, y as usize))
    }

    pub fn center(&self) -> ComplexNum {
        let re = (self.min_x + self.max_x) / 2.;
        let im = (self.min_y + self.max_y) / 2.;
        ComplexNum::new(re, im)
    }

    pub fn shift(&mut self, translation: ComplexNum) {
        self.min_x += translation.re;
        self.max_x += translation.re;
        self.min_x += translation.im;
        self.max_x += translation.im;
    }

    pub fn zoom(&mut self, scale: f64, base_point: ComplexNum) {
        self.shift(-base_point);
        self.min_x *= scale;
        self.max_x *= scale;
        self.min_y *= scale;
        self.max_y *= scale;
        self.shift(base_point);
    }

    pub fn rescale(&mut self, min_x: f64, max_x: f64, min_y: f64, max_y: f64) {
        self.res_y = Self::infer_height(self.res_x, min_x, max_x, min_y, max_y);
        self.min_x = min_x;
        self.max_x = max_x;
        self.min_y = min_y;
        self.max_y = max_y;
    }

    pub fn resize(&mut self, res_x: usize) {
        self.res_x = res_x;
        self.res_y = Self::infer_height(res_x, self.min_x, self.max_x, self.min_y, self.max_y);
    }

    pub fn to_array(&self) -> Array2<ComplexNum> {
        let mut points = Array2::zeros((self.res_x, self.res_y));
        let size_x = self.max_x - self.min_x;
        let size_y = self.max_y - self.min_y;
        points.indexed_iter_mut().for_each(|((i, j), value)| {
            let re = self.min_x + (i as f64) / size_x;
            let im = self.min_y + (j as f64) / size_y;
            *value = ComplexNum::new(re, im);
        });
        points
    }

    pub fn par_iter(&self) -> IterBridge<PointGridIterator> {
        self.into_iter().par_bridge()
    }

    pub fn iter(&self) -> PointGridIterator {
        PointGridIterator::new(
            self.res_x, self.res_y, self.min_x, self.max_x, self.min_y, self.max_y,
        )
    }
}

impl IntoIterator for PointGrid {
    type Item = ((usize, usize), ComplexNum);
    type IntoIter = PointGridIterator;

    fn into_iter(self) -> PointGridIterator {
        PointGridIterator::new(
            self.res_x, self.res_y, self.min_x, self.max_x, self.min_y, self.max_y,
        )
    }
}

pub struct PointGridIterator {
    step_x: f64,
    step_y: f64,
    res_x: usize,
    res_y: usize,
    min_x: f64,
    min_y: f64,
    idx_x: usize,
    idx_y: usize,
}

impl PointGridIterator {
    pub fn new(res_x: usize, res_y: usize, min_x: f64, max_x: f64, min_y: f64, max_y: f64) -> Self {
        let step_x = (max_x - min_x) / (res_x as f64);
        let step_y = (max_y - min_y) / (res_y as f64);

        Self {
            step_x,
            step_y,
            res_x,
            res_y,
            min_x,
            min_y,
            idx_x: 0,
            idx_y: 0,
        }
    }
}

impl Iterator for PointGridIterator {
    type Item = ((usize, usize), ComplexNum);

    fn next(&mut self) -> Option<Self::Item> {
        self.idx_x += 1;
        self.idx_y += self.idx_x / self.res_x;

        if self.idx_y == self.res_y {
            return None;
        }

        self.idx_x %= self.res_x;

        // println!("idx_x: {}, res_x: {}", self.idx_x, self.res_x);
        // println!("idx_y: {}, res_y: {}", self.idx_y, self.res_y);

        let z = ComplexNum::new(
            self.idx_x as f64 * self.step_x + self.min_x,
            self.idx_y as f64 * self.step_y + self.min_y,
        );

        Some(((self.idx_x, self.idx_y), z))
    }
}
