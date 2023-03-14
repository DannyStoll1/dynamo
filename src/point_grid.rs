use crate::primitive_types::*;
use eframe::egui::Vec2;
use ndarray::Array2;
use rayon::iter::{IterBridge, ParallelBridge};

#[derive(Clone, Copy, Debug)]
pub struct Bounds {
    pub min_x: Float,
    pub max_x: Float,
    pub min_y: Float,
    pub max_y: Float,
}
impl Bounds {
    #[inline(always)]
    pub fn range_x(&self) -> Float {
        self.max_x - self.min_x
    }

    #[inline(always)]
    pub fn range_y(&self) -> Float {
        self.max_y - self.min_y
    }

    #[inline(always)]
    pub fn area(&self) -> Float {
        self.range_x() * self.range_y()
    }

    #[inline(always)]
    pub fn mid_x(&self) -> Float {
        (self.max_x + self.min_x) / 2.
    }

    #[inline(always)]
    pub fn mid_y(&self) -> Float {
        (self.max_y + self.min_y) / 2.
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PointGrid {
    pub res_x: usize,
    pub res_y: usize,
    pub bounds: Bounds,
}

impl PointGrid {
    pub fn new(res_x: usize, res_y: usize, bounds: Bounds) -> PointGrid {
        Self {
            res_x,
            res_y,
            bounds,
        }
    }

    pub fn infer_height(res_x: usize, bounds: Bounds) -> usize {
        debug_assert!(bounds.max_x > bounds.min_x);
        debug_assert!(bounds.max_y > bounds.min_y);
        debug_assert!(res_x > 0);

        let res_x_float = res_x as Float;
        let res_y_float =
            res_x_float * (bounds.max_y - bounds.min_y) / (bounds.max_x - bounds.min_x);
        res_y_float as usize
    }

    pub fn infer_width(res_y: usize, bounds: Bounds) -> usize {
        debug_assert!(bounds.max_x > bounds.min_x);
        debug_assert!(bounds.max_y > bounds.min_y);
        debug_assert!(res_y > 0);

        let res_y_float = res_y as Float;
        let res_x_float =
            res_y_float * (bounds.max_x - bounds.min_x) / (bounds.max_y - bounds.min_y);
        res_x_float as usize
    }

    pub fn with_res_x(res_x: usize, bounds: Bounds) -> Self {
        let res_y = Self::infer_height(res_x, bounds);

        Self::new(res_x, res_y, bounds)
    }

    pub fn with_res_y(res_y: usize, bounds: Bounds) -> Self {
        let res_x = Self::infer_width(res_y, bounds);

        Self::new(res_x, res_y, bounds)
    }

    pub fn with_same_height(&self, bounds: Bounds) -> Self {
        Self::with_res_y(self.res_y, bounds)
    }

    pub fn with_same_width(&self, bounds: Bounds) -> Self {
        Self::with_res_x(self.res_x, bounds)
    }

    pub fn with_new_width(&self, res_x: usize) -> Self {
        Self::with_res_x(res_x, self.bounds)
    }

    pub fn with_new_height(&self, res_y: usize) -> Self {
        Self::with_res_y(res_y, self.bounds)
    }

    pub fn map_pixel(&self, pixel_x: usize, pixel_y: usize) -> ComplexNum {
        let re =
            self.bounds.min_x + (pixel_x as Float) * (self.bounds.range_x()) / (self.res_x as Float);
        let im =
            self.bounds.min_y + (pixel_y as Float) * (self.bounds.range_y()) / (self.res_y as Float);
        ComplexNum::new(re, -im)
    }

    pub fn map_vec2(&self, pos: Vec2) -> ComplexNum {
        let re = self.bounds.min_x + (pos.x as Float) * (self.bounds.range_x()) / (self.res_x as Float);
        let im = self.bounds.max_y - (pos.y as Float) * (self.bounds.range_y()) / (self.res_y as Float);
        ComplexNum::new(re, im)
    }

    pub fn locate_point(&self, z: ComplexNum) -> Option<(usize, usize)> {
        if z.re >= self.bounds.max_x
            || z.re < self.bounds.min_x
            || z.im >= self.bounds.max_y
            || z.re < self.bounds.min_y
        {
            return None;
        }

        let x = (z.re - self.bounds.min_x) / (self.bounds.range_x());
        let y = (z.im - self.bounds.min_y) / (self.bounds.range_y());

        Some((x as usize, self.res_y - 1 - y as usize))
    }

    pub fn center(&self) -> ComplexNum {
        let re = self.bounds.mid_x();
        let im = self.bounds.mid_y();
        ComplexNum::new(re, im)
    }

    pub fn shift(&mut self, translation: ComplexNum) {
        self.bounds.min_x += translation.re;
        self.bounds.max_x += translation.re;
        self.bounds.min_y += translation.im;
        self.bounds.max_y += translation.im;
    }

    pub fn zoom(&mut self, scale: Float, base_point: ComplexNum) {
        self.shift(-base_point);
        self.bounds.min_x *= scale;
        self.bounds.max_x *= scale;
        self.bounds.min_y *= scale;
        self.bounds.max_y *= scale;
        self.shift(base_point);
    }

    pub fn rescale(&mut self, new_bounds: Bounds) {
        self.res_y = Self::infer_height(self.res_x, new_bounds);
        self.bounds.min_x = new_bounds.min_x;
        self.bounds.max_x = new_bounds.max_x;
        self.bounds.min_y = new_bounds.min_y;
        self.bounds.max_y = new_bounds.max_y;
    }

    pub fn resize_x(&mut self, res_x: usize) {
        self.res_x = res_x;
        self.res_y = Self::infer_height(res_x, self.bounds);
    }

    pub fn resize_y(&mut self, res_y: usize) {
        self.res_y = res_y;
        self.res_y = Self::infer_width(res_y, self.bounds);
    }

    pub fn to_array(&self) -> Array2<ComplexNum> {
        let mut points = Array2::zeros((self.res_x, self.res_y));
        let size_x = self.bounds.range_x();
        let size_y = self.bounds.range_y();
        points.indexed_iter_mut().for_each(|((i, j), value)| {
            let re = self.bounds.min_x + (i as Float) / size_x;
            let im = self.bounds.min_y + (j as Float) / size_y;
            *value = ComplexNum::new(re, im);
        });
        points
    }

    pub fn par_iter(&self) -> IterBridge<PointGridIterator> {
        self.into_iter().par_bridge()
    }

    pub fn iter(&self) -> PointGridIterator {
        PointGridIterator::new(self.res_x, self.res_y, self.bounds)
    }
}

impl IntoIterator for PointGrid {
    type Item = ((usize, usize), ComplexNum);
    type IntoIter = PointGridIterator;

    fn into_iter(self) -> PointGridIterator {
        PointGridIterator::new(self.res_x, self.res_y, self.bounds)
    }
}

pub struct PointGridIterator {
    step_x: Float,
    step_y: Float,
    res_x: usize,
    res_y: usize,
    min_x: Float,
    min_y: Float,
    idx_x: usize,
    idx_y: usize,
}

impl PointGridIterator {
    pub fn new(res_x: usize, res_y: usize, bounds: Bounds) -> Self {
        let step_x = bounds.range_x() / (res_x as Float);
        let step_y = bounds.range_y() / (res_y as Float);

        Self {
            step_x,
            step_y,
            res_x,
            res_y,
            min_x: bounds.min_x,
            min_y: bounds.min_y,
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

        let z = ComplexNum::new(
            self.idx_x as Float * self.step_x + self.min_x,
            self.idx_y as Float * self.step_y + self.min_y,
        );

        Some(((self.idx_x, self.idx_y), z))
    }
}
