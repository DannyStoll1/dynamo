use crate::primitive_types::ComplexNum;
use ndarray::Array2;

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

    pub fn new_infer(
        res_x: usize,
        min_x: f64,
        max_x: f64,
        min_y: f64,
        max_y: f64,
    ) -> Self {
        assert!(max_x > min_x);
        assert!(max_y > min_y);
        assert!(res_x > 0);

        let res_x_float = res_x as f64;
        let res_y_float = res_x_float * (max_y - min_y) / (max_x - min_x);
        let res_y = res_y_float as usize;

        Self::new(res_x, res_y, min_x, max_x, min_y, max_y)
    }

    pub fn with_new_bounds(&self, min_x: f64, max_x: f64, min_y: f64, max_y: f64) -> Self {
        Self::new_infer(self.res_x, min_x, max_x, min_y, max_y)
    }

    pub fn to_array(&self) -> Array2<ComplexNum> {
        let mut points = Array2::zeros((self.res_x, self.res_y));
        let size_x = self.max_x - self.min_x;
        let size_y = self.max_y - self.min_y;
        for i in 0..self.res_x {
            for j in 0..self.res_y {
                let re = self.min_x + (i as f64) / size_x;
                let im = self.min_y + (j as f64) / size_y;
                points[[i, j]] = ComplexNum::new(re, im);
            }
        }
        points
    }

    pub fn iter(&self) -> PointGridIterator {
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
