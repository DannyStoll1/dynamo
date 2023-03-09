use crate::covering_maps::CoveringMap;
use crate::math_utils::weierstrass_p;
use crate::point_grid::PointGrid;
use crate::primitive_types::{ComplexNum, EscapeState, Period};
use crate::traits::{HasDynamicalCovers, ParameterPlane};

use crate::macros::{fractal_impl, parameter_plane_impl};

use std::any::type_name;

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
}

fractal_impl!(Mandelbrot);

impl ParameterPlane for Mandelbrot {
    parameter_plane_impl!();

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
        z * z + c
    }
}

impl HasDynamicalCovers for Mandelbrot {
    fn marked_cycle_curve(self, period: Period) -> CoveringMap<Self> {
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
                    // -c - (0.75 * u + 0.25) / u
                    -0.25 * u - 0.75 - 1. / c
                };
                bounds = self.point_grid.with_new_bounds(-2.9, 2.1, -3.75, 3.75);
            }
            _ => {
                param_map = |c| c;
                bounds = self.point_grid;
            }
        };
        CoveringMap::new(self, param_map, bounds)
    }

    fn dynatomic_curve(self, period: Period) -> CoveringMap<Self> {
        let param_map: fn(ComplexNum) -> ComplexNum;
        let bounds: PointGrid;

        match period {
            1 => {
                param_map = |c| 0.25 - c * c;
                bounds = self.point_grid.with_new_bounds(-1.8, 1.8, -1.0, 1.0);
            }
            3 => {
                param_map = |c| {
                    let c2 = c * c;
                    let v = c2 * (c2 - 3. * c + 6.) - c - c + 2.;
                    let u = v + 1. / (c2 - c);
                    -0.25 * u / (c2 - c)
                };
                bounds = self.point_grid.with_new_bounds(-2.5, 3.5, -3., 3.);
            }
            _ => {
                param_map = |c| c;
                bounds = self.point_grid;
            }
        };
        CoveringMap::new(self, param_map, bounds)
    }
    fn misiurewicz_curve(self, preperiod: Period, period: Period) -> CoveringMap<Self> {
        let param_map: fn(ComplexNum) -> ComplexNum;
        let bounds: PointGrid;

        match (preperiod, period) {
            (2, 1) => {
                param_map = |c| {
                    let c2 = c * c;
                    -2. * (c2 + 1.) / ((c2 - 1.) * (c2 - 1.))
                };
                bounds = self.point_grid.with_new_bounds(-3.5, 3.5, -3.0, 3.0);
            }
            (2, 2) => {
                param_map = |c| {
                    let c2 = c * c;
                    -(c2 * (c2 + c + c + 2.) - c - c + 1.) / (4. * c2)
                };
                bounds = self.point_grid.with_new_bounds(-4., 2.4, -2.5, 2.5);
            }
            (_, _) => {
                param_map = |c| c;
                bounds = self.point_grid;
            }
        };
        CoveringMap::new(self, param_map, bounds)
    }
}

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
}

fractal_impl!(QuadRatPer2);

impl ParameterPlane for QuadRatPer2 {
    parameter_plane_impl!();

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

impl HasDynamicalCovers for QuadRatPer2 {
    fn marked_cycle_curve(self, period: Period) -> CoveringMap<Self> {
        let param_map: fn(ComplexNum) -> ComplexNum;
        let bounds: PointGrid;

        match period {
            1 => {
                param_map = |c| (4. - c * (c + 2.)) * c / 8.;
                bounds = self.point_grid.with_new_bounds(-5.0, 3.0, -3.0, 3.0);
            }
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

    fn misiurewicz_curve(self, preperiod: Period, period: Period) -> CoveringMap<Self> {
        let param_map: fn(ComplexNum) -> ComplexNum;
        let bounds: PointGrid;

        match (preperiod, period) {
            (2, 1) => {
                param_map = |c| {
                    let c2 = c * c;
                    // -25*(131*t^4 - 102*t^3 - 106*t^2 - 8*t - 4)*t^2/(13*t^2 + 2*t + 2)^3
                    let denom = 13. * c2 + c + c + 2.;
                    let numer = c2 * (131. * c2 - 102. * c - 106.) - 8. * c - 4.;
                    25. * c2 * numer / (denom * denom * denom)
                };
                bounds = self.point_grid.with_new_bounds(-3.4, 3.4, -5.1, 5.1);
            }
            (2, 2) => {
                param_map = |c| {
                    //(-t^4 + 2*t^2 + 1)/(2*t^4)
                    let c2 = c * c;
                    0.5 - (c2 + 0.5) / (c2 * c2)
                };
                bounds = self.point_grid.with_new_bounds(-4., 4., -4., 4.);
            }
            (_, _) => {
                param_map = |c| c;
                bounds = self.point_grid;
            }
        };
        CoveringMap::new(self, param_map, bounds)
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
}
fractal_impl!(QuadRatPer3);

impl ParameterPlane for QuadRatPer3 {
    parameter_plane_impl!();

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
impl HasDynamicalCovers for QuadRatPer3 {
    fn marked_cycle_curve(self, period: Period) -> CoveringMap<Self> {
        let param_map: fn(ComplexNum) -> ComplexNum;
        let bounds: PointGrid;

        match period {
            1 => {
                param_map = |c| {
                    let pole = 1.32471795724475;
                    let c = 1. / c + pole;
                    let c2 = c * c;
                    let c3 = c2 * c;
                    (c3 - c + 1.) / (c3 - c2 - c2 + c + c + c - 1.)
                };
                bounds = self.point_grid.with_new_bounds(-5.75, 5.08, -5.32, 5.32);
            }
            4 => {
                param_map = |c| {
                    let t = 13.0_f64.sqrt();
                    let g2 = ComplexNum::new(-8.0 / 3.0, 0.);
                    let g3 = ComplexNum::new(1.0 / 27.0, 0.);

                    let (p, dp) = weierstrass_p(g2, g3, c, 0.01);
                    let x = p - 1. / 3.;
                    let y = (dp + 1.) / x - t - 1.;

                    let u = x / 2.;
                    let v = y / 4.;
                    let xx = -(t + 1.) * u + (t + 3.) * v + (t + 4.);
                    let yy = u - v - (t + 1.) / 4.;
                    let zz = -x + 2. * v + (t + 3.) / 2.;

                    let x0 = yy / zz;
                    let x1 = xx * yy / (zz * zz);

                    let s0 = x1 / x0;
                    let s1 = 1. / x0;

                    s0 * s1 + s1 + (t + 4.)
                    // let l = s0^2*s1 + s0*s1 + (2*t)*s0 + (t - 1);
                };
                bounds = self.point_grid.with_new_bounds(-3.9, 3.9, -2.6, 2.6);
            }
            _ => {
                param_map = |c| c;
                bounds = self.point_grid;
            }
        };
        CoveringMap::new(self, param_map, bounds)
    }
}


#[derive(Clone, Copy, Debug)]
pub struct QuadRatPer4 {
    point_grid: PointGrid,
    max_iter: i32,
}

impl QuadRatPer4 {
    const ESCAPE_RADIUS: f64 = 1.0e+22_f64;
    const DEFAULT_MIN_X: f64 = -1.;
    const DEFAULT_MAX_X: f64 = 0.2;
    const DEFAULT_MIN_Y: f64 = -0.5;
    const DEFAULT_MAX_Y: f64 = 0.5;
}
fractal_impl!(QuadRatPer4);

impl ParameterPlane for QuadRatPer4 {
    parameter_plane_impl!();

    fn encode_escape_result(&self, iter: i32, state: EscapeState, c: ComplexNum) -> f64 {
        match state {
            EscapeState::NotYetEscaped => 0.,
            EscapeState::Bounded => -1.,
            EscapeState::Escaped(z) => {
                let u = Self::ESCAPE_RADIUS.log2();
                let v = z.norm_sqr().log2();
                let c2 = c * c;
                let _2c = c + c;
                let c12 = c2 - _2c + 1.; // (c-1)^2

                let d0 = c2 + c - 1.; // c^2 + c - 1
                let d1 = d0 - _2c - _2c + 2.; // c^2 - 3c + 1
                let d2 = c2 + c2 + d1; // 3c^2 - 3c + 1

                // (2*a - 1) * (a - 1)^5 * a^5 * (a^2 - 3*a + 1)^-2 * (3a^2 - 3a + 1)^-2 * (a^2 + a - 1)^-2
                let q_numer = (_2c - 1.) * c2 * c2 * c * c12 * c12 * (c - 1.);
                let q_denom = d0 * d0 + d1 * d1 + d2 * d2;
                let q = (q_numer / q_denom).norm().log2();
                let residual = ((u + q) / (v + q)).log2();
                (iter as f64) + residual * 4.
            }
        }
    }

    fn param_map(&self, c: ComplexNum) -> ComplexNum {
        let pole = 2.61803398874989;
        1. / c + pole
    }

    fn start_point(&self, c: ComplexNum) -> ComplexNum {
        (c + c) * (c + c - 1.) / (c * (c + 1.) - 1.)
    }

    #[inline]
    fn map(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum {
        (c * z - c - c - z + 1.) * (z - c) / (z * z * (c - 1.))
    }
}

impl HasDynamicalCovers for QuadRatPer4 {
    fn marked_cycle_curve(self, period: Period) -> CoveringMap<Self> {
        let param_map: fn(ComplexNum) -> ComplexNum;
        let bounds: PointGrid;

        match period {
            // 1 => {
            //     param_map = |c| (4. - c * (c + 2.)) * c / 8.;
            //     bounds = self.point_grid.with_new_bounds(-5.0, 3.0, -3.0, 3.0);
            // }
            3 => {
                param_map = |c| {
                    // (18)^(2/3) / 3
                    let alpha = ComplexNum::new(2.28942848510666, 0.);
                    let g2 = alpha;
                    let g3 = ComplexNum::new(-19. / 12., 0.);

                    let (p, dp) = weierstrass_p(g2, g3, c, 0.01);
                    let x = (alpha * p + 1.) / 3.;
                    let y = (dp - 1.5) / x;

                    let xx = x + 1.;
                    let yy = y - 3. * x - 3.;

                    let x0 = yy / x;
                    let _s1 = x0 * xx / x;

                    x / xx
                };
                bounds = self.point_grid.with_new_bounds(-3.6, 3.6, -2.4, 2.4);
            }
            _ => {
                param_map = |c| c;
                bounds = self.point_grid;
            }
        };
        CoveringMap::new(self, param_map, bounds)
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
}
fractal_impl!(BurningShip);

impl ParameterPlane for BurningShip {
    parameter_plane_impl!();

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

#[derive(Clone, Copy, Debug)]
pub struct Sailboat {
    point_grid: PointGrid,
    max_iter: i32,
    shift: ComplexNum,
}

impl Sailboat {
    const ESCAPE_RADIUS: f64 = 1e12_f64;
    const DEFAULT_MIN_X: f64 = -6.;
    const DEFAULT_MAX_X: f64 = 6.;
    const DEFAULT_MIN_Y: f64 = -6.;
    const DEFAULT_MAX_Y: f64 = 6.;

    pub fn new(
        res_x: usize,
        max_iter: i32,
        shift: ComplexNum,
        min_x: f64,
        max_x: f64,
        min_y: f64,
        max_y: f64,
    ) -> Self {
        let point_grid = PointGrid::new_infer(res_x, min_x, max_x, min_y, max_y);

        Self {
            point_grid,
            max_iter,
            shift,
        }
    }

    pub fn new_default(res_x: usize, max_iter: i32, shift: ComplexNum) -> Self {
        Self::new(
            res_x,
            max_iter,
            shift,
            Self::DEFAULT_MIN_X,
            Self::DEFAULT_MAX_X,
            Self::DEFAULT_MIN_Y,
            Self::DEFAULT_MAX_Y,
        )
    }
}

impl ParameterPlane for Sailboat {
    fn point_grid(&self) -> PointGrid {
        self.point_grid
    }

    fn stop_condition(&self, iter: i32, z: ComplexNum) -> EscapeState {
        if iter > self.max_iter {
            return EscapeState::Bounded;
        }

        let r = z.norm_sqr();
        if r > Self::ESCAPE_RADIUS {
            EscapeState::Escaped(z)
        } else {
            EscapeState::NotYetEscaped
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
        let z = ComplexNum::new(z.re.abs(), z.im.abs()) + self.shift;
        z * z + c
    }

    fn param_map(&self, c: ComplexNum) -> ComplexNum {
        c
    }

    fn name(&self) -> String {
        let shift = self.shift;
        format!("Sailboat({shift})")
    }
}
