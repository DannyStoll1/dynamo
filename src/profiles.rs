use image::ImageBuffer;
use ndarray::Array2;

use crate::covering_maps::CoveringMap;
use crate::point_grid::PointGrid;
use crate::primitive_types::{ComplexNum, EscapeState, Period};
use crate::traits::ParameterPlane;

#[derive(Clone, Copy, Debug)]
pub struct Mandelbrot {
    point_grid: PointGrid,
    max_iter: i32,
}

impl Mandelbrot {
    const ESCAPE_RADIUS: f64 = 1e12_f64;
    const DEFAULT_MIN_X: f64 = -2.2;
    const DEFAULT_MAX_X: f64 = 0.65;
    const DEFAULT_MIN_Y: f64 = -1.4;
    const DEFAULT_MAX_Y: f64 = 1.4;

    pub fn new(
        res_x: usize,
        max_iter: i32,
        min_x: f64,
        max_x: f64,
        min_y: f64,
        max_y: f64,
    ) -> Self {
        let point_grid = PointGrid::new_infer(res_x, min_x, max_x, min_y, max_y);

        Self {
            point_grid,
            max_iter,
        }
    }

    pub fn new_default(res_x: usize, max_iter: i32) -> Self {
        Self::new(
            res_x,
            max_iter,
            Self::DEFAULT_MIN_X,
            Self::DEFAULT_MAX_X,
            Self::DEFAULT_MIN_Y,
            Self::DEFAULT_MAX_Y,
        )
    }

    pub fn marked_cycle(self, period: Period) -> CoveringMap<Self> {
        let param_map: fn(ComplexNum) -> ComplexNum;
        let bounds: PointGrid;

        match period {
            1 => {
                param_map = |c| 0.25 - c * c;
                bounds = self.point_grid.with_new_bounds(-1.8, 1.8, -1.0, 1.0);
            }
            3 => {
                param_map = |c| -1.75 * (1. + 7. * c * c);
                bounds = self.point_grid.with_new_bounds(-0.3, 0.3, -0.5, 0.5);
            }
            4 => {
                param_map = |c| {
                    let u = c * c;
                    -c - (0.75 * u + 0.25) / u
                };
                bounds = self.point_grid.with_new_bounds(-1.7, 1.2, -1.4, 1.2);
            }
            _ => {
                param_map = |c| c;
                bounds = self.point_grid;
            }
        };
        CoveringMap::new(self, param_map, bounds)
    }
}

impl ParameterPlane for Mandelbrot {
    fn point_grid(&self) -> PointGrid {
        self.point_grid
    }
    const NUM_TRACKED_POINTS: usize = 1;

    fn stop_condition(&self, iter: i32, z: ComplexNum) -> EscapeState {
        if iter > self.max_iter {
            return EscapeState::Bounded;
        }

        let r = z.norm_sqr();
        if r > Self::ESCAPE_RADIUS {
            return EscapeState::Escaped(z);
        } else {
            return EscapeState::NotYetEscaped;
        }
    }

    fn encode_escape_result(&self, iter: i32, state: EscapeState, _base_param: ComplexNum) -> f64 {
        match state {
            EscapeState::NotYetEscaped => 0.,
            EscapeState::Bounded => -1.,
            EscapeState::Escaped(z) => {
                let u = Self::ESCAPE_RADIUS.log2();
                let v = z.norm_sqr().log2();
                let residual = (v / u).log2();
                (iter as f64) - residual
                // let u = Self::ESCAPE_RADIUS.log2();
                // let v = r.log2();
                // (iter as f64) + u + u - v / u
            }
        }
    }

    fn map(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum {
        z * z + c
    }
}

// impl HasMarkedCycleCurves for Mandelbrot {
//     fn marked_cycle_param_map(period: Period, c: ComplexNum) -> ComplexNum
//     {
//         match period {
//             3 => return -7. * (1. + 7. * (c * c)) / 4.,
//             _ => return c,
//         }
//     }
// }

#[derive(Clone, Copy, Debug)]
pub struct QuadRatPer2 {
    point_grid: PointGrid,
    max_iter: i32,
}

impl QuadRatPer2 {
    const ESCAPE_RADIUS: f64 = 1.0e+13_f64;
    const DEFAULT_MIN_X: f64 = -2.8;
    const DEFAULT_MAX_X: f64 = 3.2;
    const DEFAULT_MIN_Y: f64 = -2.8;
    const DEFAULT_MAX_Y: f64 = 2.8;

    pub fn new(
        res_x: usize,
        max_iter: i32,
        min_x: f64,
        max_x: f64,
        min_y: f64,
        max_y: f64,
    ) -> Self {
        let point_grid = PointGrid::new_infer(res_x, min_x, max_x, min_y, max_y);

        Self {
            point_grid,
            max_iter,
        }
    }

    pub fn new_default(res_x: usize, max_iter: i32) -> Self {
        Self::new(
            res_x,
            max_iter,
            Self::DEFAULT_MIN_X,
            Self::DEFAULT_MAX_X,
            Self::DEFAULT_MIN_Y,
            Self::DEFAULT_MAX_Y,
        )
    }

    pub fn marked_cycle(self, period: Period) -> CoveringMap<Self> {
        let param_map: fn(ComplexNum) -> ComplexNum;
        let bounds: PointGrid;

        match period {
            1 => {
                param_map = |c| (4. - c * (c + 2.)) * c / 8.;
                bounds = self.point_grid.with_new_bounds(-5.0, 3.0, -3.0, 3.0);
            }
            // 3 => {
            //     param_map = |c| -1.75 * (1. + 7. * c * c);
            //     bounds = self.point_grid.with_new_bounds(-0.3, 0.3, -0.5, 0.5);
            // }
            4 => {
                param_map = |c| {
                    let u = c * c;
                    u * c - 2. * u + 4. * c - 1.
                };
                bounds = self.point_grid.with_new_bounds(-1., 1.4, -2.2, 2.2);
            }
            5 => {
                param_map = |c| {
                    // t = sqrt(-2235)
                    // ((-2043332879690812551104*t + 322671215001188162496)*c^6 + (-7211787718815174272*t + 38457203855637713472)*c^5 + (-10445615819508480*t + 113836835145028800)*c^4 + (-7931553616080*t + 135137329840080)*c^3 + (-3321323160*t + 79799557200)*c^2 + (-724598*t + 23400162)*c + (-64*t + 2724))/((-165726073638468871360*t + 59671792608719217337728)*c^6 + (-532082528560799520*t + 218792941658814953376)*c^5 + (-681491680626360*t + 334169395252260120)*c^4 + (-435333784880*t + 272101938829200)*c^3 + (-138715290*t + 124564255830)*c^2 + (-17640*t + 30391956)*c + 3087)
                    let pole = ComplexNum::new(-1.02913187270464, 0.0515641552714143);
                    let angle = ComplexNum::new(1., 0.);

                    let c = angle / c + pole;

                    let a0 = ComplexNum::new(-5448., 6051.30068662928);
                    let a1 = ComplexNum::new(-29961.7951344430, 43861.6394739337);
                    let a2 = ComplexNum::new(-65413.6552992732, 128711.643030672);
                    let a3 = ComplexNum::new(-70918.9407863760, 196781.349743989);
                    let a4 = ComplexNum::new(-38246.2351271793, 165912.340564512);
                    let a5 = ComplexNum::new(-8271.84813212745, 73334.1979222552);
                    let a6 = ComplexNum::new(-44.4328369324866, 13302.1458570374);

                    let b0 = ComplexNum::new(-6174., 0.);
                    let b1 = ComplexNum::new(-38914.1562099872, 1067.79113428438);
                    let b2 = ComplexNum::new(-102108.377281498, 5375.65061551438);
                    let b3 = ComplexNum::new(-142796.822391875, 10800.6040082957);
                    let b4 = ComplexNum::new(-112272.282050380, 10824.4340747047);
                    let b5 = ComplexNum::new(-47060.6753568701, 5410.56489483889);
                    let b6 = ComplexNum::new(-8216.99273808066, 1078.88069817905);

                    let numer = a0 + c * (a1 + c * (a2 + c * (a3 + c * (a4 + c * (a5 + c * a6)))));
                    let denom = b0 + c * (b1 + c * (b2 + c * (b3 + c * (b4 + c * (b5 + c * b6)))));

                    -numer / denom
                };
                bounds = self.point_grid.with_new_bounds(-8., 5.5, -1.5, 8.);
            }
            _ => {
                param_map = |c| c;
                bounds = self.point_grid;
            }
        };
        CoveringMap::new(self, param_map, bounds)
    }
}

impl ParameterPlane for QuadRatPer2 {
    const NUM_TRACKED_POINTS: usize = 1;

    fn point_grid(&self) -> PointGrid {
        self.point_grid
    }
    fn stop_condition(&self, iter: i32, z: ComplexNum) -> EscapeState {
        if iter > self.max_iter {
            return EscapeState::Bounded;
        }

        let r = z.norm_sqr();
        if r > Self::ESCAPE_RADIUS {
            return EscapeState::Escaped(z);
        } else {
            return EscapeState::NotYetEscaped;
        }
    }

    fn encode_escape_result(&self, iter: i32, state: EscapeState, _base_param: ComplexNum) -> f64 {
        match state {
            EscapeState::NotYetEscaped => 0.,
            EscapeState::Bounded => -1.,
            EscapeState::Escaped(z) => {
                let u = Self::ESCAPE_RADIUS.log2();
                let v = z.norm_sqr().log2();
                let residual = ((v - 1.) / (u + u - 1.)).log2() + 1.;
                // (F - M) / (2L - M)
                (iter as f64) - residual * 2.
            }
        }
    }

    fn map(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum {
        (z * z + c) / (z * z - 1.)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct QuadRatPer3 {
    point_grid: PointGrid,
    max_iter: i32,
}

impl QuadRatPer3 {
    const ESCAPE_RADIUS: f64 = 1.0e+13_f64;
    const DEFAULT_MIN_X: f64 = -2.5;
    const DEFAULT_MAX_X: f64 = 3.2;
    const DEFAULT_MIN_Y: f64 = -2.5;
    const DEFAULT_MAX_Y: f64 = 2.5;

    pub fn new(
        res_x: usize,
        max_iter: i32,
        min_x: f64,
        max_x: f64,
        min_y: f64,
        max_y: f64,
    ) -> Self {
        let point_grid = PointGrid::new_infer(res_x, min_x, max_x, min_y, max_y);

        Self {
            point_grid,
            max_iter,
        }
    }

    pub fn new_default(res_x: usize, max_iter: i32) -> Self {
        Self::new(
            res_x,
            max_iter,
            Self::DEFAULT_MIN_X,
            Self::DEFAULT_MAX_X,
            Self::DEFAULT_MIN_Y,
            Self::DEFAULT_MAX_Y,
        )
    }
}

impl ParameterPlane for QuadRatPer3 {
    const NUM_TRACKED_POINTS: usize = 1;

    fn point_grid(&self) -> PointGrid {
        self.point_grid
    }
    fn stop_condition(&self, iter: i32, z: ComplexNum) -> EscapeState {
        if iter > self.max_iter {
            return EscapeState::Bounded;
        }

        let r = z.norm_sqr();
        if r > Self::ESCAPE_RADIUS {
            return EscapeState::Escaped(z);
        } else {
            return EscapeState::NotYetEscaped;
        }
    }

    fn start_point(&self, _c: ComplexNum) -> ComplexNum {
        0.0.into()
    }

    fn encode_escape_result(&self, iter: i32, state: EscapeState, base_param: ComplexNum) -> f64 {
        match state {
            EscapeState::NotYetEscaped => 0.,
            EscapeState::Bounded => -1.,
            EscapeState::Escaped(z) => {
                let u = Self::ESCAPE_RADIUS.log2();
                let v = z.norm_sqr().log2();
                let q = ((base_param - 1.) / (4. * base_param)).norm().log2();
                let residual = ((u + q) / (v + q)).log2();
                (iter as f64) + residual * 3.
            }
        }
    }

    fn map(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum {
        (z * z + c * c * c - c - 1.) / (z * z - c * c)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BurningShip {
    point_grid: PointGrid,
    max_iter: i32,
}

impl BurningShip {
    const ESCAPE_RADIUS: f64 = 1e12_f64;
    const DEFAULT_MIN_X: f64 = -2.2;
    const DEFAULT_MAX_X: f64 = 1.25;
    // TODO: why are these flipped?
    const DEFAULT_MIN_Y: f64 = -1.9;
    const DEFAULT_MAX_Y: f64 = 0.6;

    pub fn new(
        res_x: usize,
        max_iter: i32,
        min_x: f64,
        max_x: f64,
        min_y: f64,
        max_y: f64,
    ) -> Self {
        let point_grid = PointGrid::new_infer(res_x, min_x, max_x, min_y, max_y);

        Self {
            point_grid,
            max_iter,
        }
    }

    pub fn new_default(res_x: usize, max_iter: i32) -> Self {
        Self::new(
            res_x,
            max_iter,
            Self::DEFAULT_MIN_X,
            Self::DEFAULT_MAX_X,
            Self::DEFAULT_MIN_Y,
            Self::DEFAULT_MAX_Y,
        )
    }
}

impl ParameterPlane for BurningShip {
    fn point_grid(&self) -> PointGrid {
        self.point_grid
    }
    const NUM_TRACKED_POINTS: usize = 1;

    fn stop_condition(&self, iter: i32, z: ComplexNum) -> EscapeState {
        if iter > self.max_iter {
            return EscapeState::Bounded;
        }

        let r = z.norm_sqr();
        if r > Self::ESCAPE_RADIUS {
            return EscapeState::Escaped(z);
        } else {
            return EscapeState::NotYetEscaped;
        }
    }

    fn encode_escape_result(&self, iter: i32, state: EscapeState, _base_param: ComplexNum) -> f64 {
        match state {
            EscapeState::NotYetEscaped => 0.,
            EscapeState::Bounded => -1.,
            EscapeState::Escaped(z) => {
                let u = Self::ESCAPE_RADIUS.log2();
                let v = z.norm_sqr().log2();
                let residual = (v / u).log2();
                (iter as f64) - residual
            }
        }
    }

    fn map(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum {
        let z = ComplexNum::new(z.re.abs(), z.im.abs());
        z * z + c
    }

    fn param_map(&self, c: ComplexNum) -> ComplexNum {
        c
    }
}

