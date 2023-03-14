use crate::covering_maps::CoveringMap;
use crate::math_utils::*;
use crate::point_grid::*;
use crate::primitive_types::*;
use crate::traits::{HasDynamicalCovers, ParameterPlane};

use crate::macros::*;

use std::any::type_name;

#[derive(Clone, Copy, Debug)]
pub struct Mandelbrot {
    point_grid: PointGrid,
    point_grid_child: PointGrid,
    max_iter: Period,
}

impl Mandelbrot {
    const DEFAULT_MIN_X: Float = -2.2;
    const DEFAULT_MAX_X: Float = 0.65;
    const DEFAULT_MIN_Y: Float = -1.4;
    const DEFAULT_MAX_Y: Float = 1.4;
    fractal_impl!();
}

impl ParameterPlane for Mandelbrot {
    parameter_plane_impl!();
    default_name!();

    fn encode_escape_result(&self, state: EscapeState, _base_param: ComplexNum) -> IterCount {
        match state {
            EscapeState::NotYetEscaped => 0.,
            EscapeState::Bounded => 0.,
            EscapeState::Periodic { period: n, .. } => -(n as IterCount),
            EscapeState::Escaped {
                iters: iter,
                final_value: z,
            } => {
                let u = self.escape_radius().log2();
                let v = z.norm_sqr().log2();
                let residual = (v / u).log2();
                (iter as IterCount) - (residual as IterCount)
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
        let bounds: Bounds;

        match period {
            1 => {
                param_map = |c| 0.25 - c * c;
                bounds = Bounds {
                    min_x: -1.8,
                    max_x: 1.8,
                    min_y: -1.0,
                    max_y: 1.0,
                };
            }
            3 => {
                param_map = |c| -1.75 * (1. + 7. * c * c);
                bounds = Bounds {
                    min_x: -0.3,
                    max_x: 0.3,
                    min_y: -0.5,
                    max_y: 0.5,
                };
            }
            4 => {
                param_map = |c| {
                    let u = c * c;
                    -0.25 * u - 0.75 - 1. / c
                };
                bounds = Bounds {
                    min_x: -2.9,
                    max_x: 2.1,
                    min_y: -3.1,
                    max_y: 3.1,
                };
            }
            _ => {
                param_map = |c| c;
                bounds = self.point_grid.bounds;
            }
        };
        let grid = self.point_grid.with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }

    fn dynatomic_curve(self, period: Period) -> CoveringMap<Self> {
        let param_map: fn(ComplexNum) -> ComplexNum;
        let bounds: Bounds;

        match period {
            1 => {
                param_map = |c| 0.25 - c * c;
                bounds = Bounds {
                    min_x: -1.8,
                    max_x: 1.8,
                    min_y: -1.0,
                    max_y: 1.0,
                };
            }
            3 => {
                param_map = |c| {
                    let c2 = c * c;
                    let v = c2 * (c2 - 3. * c + 6.) - c - c + 2.;
                    let u = v + 1. / (c2 - c);
                    -0.25 * u / (c2 - c)
                };
                bounds = Bounds {
                    min_x: -2.5,
                    max_x: 3.5,
                    min_y: -3.,
                    max_y: 3.,
                };
            }
            _ => {
                param_map = |c| c;
                bounds = self.point_grid.bounds;
            }
        };
        let grid = self.point_grid.with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }
    fn misiurewicz_curve(self, preperiod: Period, period: Period) -> CoveringMap<Self> {
        let param_map: fn(ComplexNum) -> ComplexNum;
        let bounds: Bounds;

        match (preperiod, period) {
            (2, 1) => {
                param_map = |c| {
                    let c2 = c * c;
                    -2. * (c2 + 1.) / ((c2 - 1.) * (c2 - 1.))
                };
                bounds = Bounds {
                    min_x: -3.5,
                    max_x: 3.5,
                    min_y: -3.0,
                    max_y: 3.0,
                };
            }
            (2, 2) => {
                param_map = |c| {
                    let c2 = c * c;
                    -(c2 * (c2 + c + c + 2.) - c - c + 1.) / (4. * c2)
                };
                bounds = Bounds {
                    min_x: -4.,
                    max_x: 2.4,
                    min_y: -2.5,
                    max_y: 2.5,
                };
            }
            (_, _) => {
                param_map = |c| c;
                bounds = self.point_grid.bounds;
            }
        };
        let grid = self.point_grid.with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct QuadRatPer2 {
    point_grid: PointGrid,
    point_grid_child: PointGrid,
    max_iter: Period,
}

impl QuadRatPer2 {
    const DEFAULT_MIN_X: Float = -2.8;
    const DEFAULT_MAX_X: Float = 3.2;
    const DEFAULT_MIN_Y: Float = -2.8;
    const DEFAULT_MAX_Y: Float = 2.8;
    fractal_impl!();
}

impl ParameterPlane for QuadRatPer2 {
    parameter_plane_impl!();
    default_name!();

    fn encode_escape_result(&self, state: EscapeState, _base_param: ComplexNum) -> IterCount {
        match state {
            EscapeState::NotYetEscaped => 0.,
            EscapeState::Bounded => 0.,
            EscapeState::Periodic { period: n, .. } => -(n as IterCount),
            EscapeState::Escaped {
                iters: iter,
                final_value: z,
            } => {
                let u = self.escape_radius().log2();
                let v = z.norm_sqr().log2();
                let residual = ((v - 1.) / (u + u - 1.)).log2() + 1.;
                // (F - M) / (2L - M)
                (iter as IterCount) - (residual as IterCount) * 2.
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
        let bounds: Bounds;

        match period {
            1 => {
                param_map = |c| (4. - c * (c + 2.)) * c / 8.;
                bounds = Bounds {
                    min_x: -5.0,
                    max_x: 3.0,
                    min_y: -3.0,
                    max_y: 3.0,
                };
            }
            4 => {
                param_map = |c| {
                    let u = c * c;
                    u * c - 2. * u + 4. * c - 1.
                };
                bounds = Bounds {
                    min_x: -1.,
                    max_x: 1.4,
                    min_y: -2.2,
                    max_y: 2.2,
                };
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
                bounds = Bounds {
                    min_x: -8.,
                    max_x: 5.5,
                    min_y: -8.,
                    max_y: 1.5,
                };
            }
            _ => {
                param_map = |c| c;
                bounds = self.point_grid.bounds;
            }
        };
        let grid = self.point_grid.with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }

    fn misiurewicz_curve(self, preperiod: Period, period: Period) -> CoveringMap<Self> {
        let param_map: fn(ComplexNum) -> ComplexNum;
        let bounds: Bounds;

        match (preperiod, period) {
            (2, 1) => {
                param_map = |c| {
                    let c2 = c * c;
                    // -25*(131*t^4 - 102*t^3 - 106*t^2 - 8*t - 4)*t^2/(13*t^2 + 2*t + 2)^3
                    let denom = 13. * c2 + c + c + 2.;
                    let numer = c2 * (131. * c2 - 102. * c - 106.) - 8. * c - 4.;
                    25. * c2 * numer / (denom * denom * denom)
                };
                bounds = Bounds {
                    min_x: -3.4,
                    max_x: 3.4,
                    min_y: -5.1,
                    max_y: 5.1,
                };
            }
            (2, 2) => {
                param_map = |c| {
                    //(-t^4 + 2*t^2 + 1)/(2*t^4)
                    let c2 = c * c;
                    0.5 - (c2 + 0.5) / (c2 * c2)
                };
                bounds = Bounds {
                    min_x: -4.,
                    max_x: 4.,
                    min_y: -4.,
                    max_y: 4.,
                };
            }
            (_, _) => {
                param_map = |c| c;
                bounds = self.point_grid.bounds;
            }
        };
        let grid = self.point_grid.with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct QuadRatPer3 {
    point_grid: PointGrid,
    point_grid_child: PointGrid,
    max_iter: Period,
}

impl QuadRatPer3 {
    const DEFAULT_MIN_X: Float = -2.5;
    const DEFAULT_MAX_X: Float = 3.2;
    const DEFAULT_MIN_Y: Float = -2.5;
    const DEFAULT_MAX_Y: Float = 2.5;
    fractal_impl!();
}

impl ParameterPlane for QuadRatPer3 {
    parameter_plane_impl!();
    default_name!();

    fn start_point(&self, _c: ComplexNum) -> ComplexNum {
        0.0.into()
    }

    fn encode_escape_result(&self, state: EscapeState, base_param: ComplexNum) -> IterCount {
        match state {
            EscapeState::NotYetEscaped => 0.,
            EscapeState::Bounded => 0.,
            EscapeState::Periodic { period, .. } => -(period as IterCount),
            EscapeState::Escaped {
                iters,
                final_value: z,
            } => {
                let u = self.escape_radius().log2();
                let v = z.norm_sqr().log2();
                let q = ((base_param - 1.) / (4. * base_param)).norm().log2();
                let residual = ((u + q) / (v + q)).log2();
                (iters as IterCount) + (residual as IterCount) * 3.
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
        let bounds: Bounds;

        match period {
            1 => {
                param_map = |c| {
                    let pole = 1.32471795724475;
                    let c = 1. / c + pole;
                    let c2 = c * c;
                    let c3 = c2 * c;
                    (c3 - c + 1.) / (c3 - c2 - c2 + c + c + c - 1.)
                };
                bounds = Bounds {
                    min_x: -5.75,
                    max_x: 5.08,
                    min_y: -5.32,
                    max_y: 5.32,
                };
            }
            4 => {
                param_map = |c| {
                    let t = (13.0 as Float).sqrt();
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
                bounds = Bounds {
                    min_x: -3.9,
                    max_x: 3.9,
                    min_y: -2.6,
                    max_y: 2.6,
                };
            }
            _ => {
                param_map = |c| c;
                bounds = self.point_grid.bounds;
            }
        };
        let grid = self.point_grid.with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct QuadRatPer4 {
    point_grid: PointGrid,
    point_grid_child: PointGrid,
    max_iter: Period,
}

impl QuadRatPer4 {
    const DEFAULT_MIN_X: Float = -1.;
    const DEFAULT_MAX_X: Float = 0.2;
    const DEFAULT_MIN_Y: Float = -0.5;
    const DEFAULT_MAX_Y: Float = 0.5;
    fractal_impl!();
}

impl ParameterPlane for QuadRatPer4 {
    parameter_plane_impl!();
    default_name!();

    fn encode_escape_result(&self, state: EscapeState, c: ComplexNum) -> IterCount {
        match state {
            EscapeState::NotYetEscaped => 0.,
            EscapeState::Bounded => 0.,
            EscapeState::Periodic { period: n, .. } => -(n as IterCount),
            EscapeState::Escaped {
                iters,
                final_value: z,
            } => {
                let u = self.escape_radius().log2();
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
                (iters as IterCount) + (residual as IterCount) * 4.
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
        let grid: PointGrid;
        let bounds: Bounds;

        match period {
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
                bounds = Bounds {
                    min_x: -3.6,
                    max_x: 3.6,
                    min_y: -2.4,
                    max_y: 2.4,
                };
                grid = self.point_grid.with_same_height(bounds);
            }
            _ => {
                param_map = |c| c;
                grid = self.point_grid;
            }
        };
        CoveringMap::new(self, param_map, grid)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BurningShip {
    point_grid: PointGrid,
    point_grid_child: PointGrid,
    max_iter: Period,
}

impl BurningShip {
    const DEFAULT_MIN_X: Float = -2.2;
    const DEFAULT_MAX_X: Float = 1.25;
    // TODO: why are these flipped?
    const DEFAULT_MIN_Y: Float = -1.9;
    const DEFAULT_MAX_Y: Float = 0.6;
    fractal_impl!();
}

impl ParameterPlane for BurningShip {
    parameter_plane_impl!();
    default_name!();

    fn encode_escape_result(&self, state: EscapeState, _base_param: ComplexNum) -> IterCount {
        match state {
            EscapeState::NotYetEscaped => 0.,
            EscapeState::Bounded => 0.,
            EscapeState::Periodic { period, .. } => -(period as IterCount),
            EscapeState::Escaped {
                iters,
                final_value: z,
            } => {
                let u = self.escape_radius().log2();
                let v = z.norm_sqr().log2();
                let residual = (v / u).log2();
                (iters as IterCount) - (residual as IterCount)
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
    point_grid_child: PointGrid,
    max_iter: Period,
    shift: ComplexNum,
}

impl Sailboat {
    const DEFAULT_MIN_X: Float = -6.;
    const DEFAULT_MAX_X: Float = 6.;
    const DEFAULT_MIN_Y: Float = -6.;
    const DEFAULT_MAX_Y: Float = 6.;
    const JULIA_BOUNDS: Bounds = Bounds {
        min_x: -5.,
        max_x: 5.,
        min_y: -5.,
        max_y: 5.,
    };

    pub fn new(
        res_x: usize,
        res_y: usize,
        max_iter: Period,
        shift: ComplexNum,
        bounds: Bounds,
    ) -> Self {
        let point_grid = PointGrid::new(res_x, res_y, bounds);
        let point_grid_child = PointGrid::new(res_x, res_y, Self::JULIA_BOUNDS);

        Self {
            point_grid,
            point_grid_child,
            max_iter,
            shift,
        }
    }

    pub fn with_res_y(res_y: usize, max_iter: Period, shift: ComplexNum, bounds: Bounds) -> Self {
        let point_grid = PointGrid::with_res_y(res_y, bounds);
        let point_grid_child = PointGrid::with_res_y(res_y, Self::JULIA_BOUNDS);

        Self {
            point_grid,
            point_grid_child,
            max_iter,
            shift,
        }
    }

    pub fn with_res_x(res_x: usize, max_iter: Period, shift: ComplexNum, bounds: Bounds) -> Self {
        let point_grid = PointGrid::with_res_x(res_x, bounds);
        let point_grid_child = PointGrid::with_res_x(res_x, Self::JULIA_BOUNDS);
        Self {
            point_grid,
            point_grid_child,
            max_iter,
            shift,
        }
    }

    pub fn new_default(res_y: usize, max_iter: Period, shift: ComplexNum) -> Self {
        let bounds = Bounds {
            min_x: Self::DEFAULT_MIN_X,
            max_x: Self::DEFAULT_MAX_X,
            min_y: Self::DEFAULT_MIN_Y,
            max_y: Self::DEFAULT_MAX_Y,
        };
        Self::with_res_y(res_y, max_iter, shift, bounds)
    }
}

impl ParameterPlane for Sailboat {
    parameter_plane_impl!();

    fn encode_escape_result(&self, state: EscapeState, _base_param: ComplexNum) -> IterCount {
        match state {
            EscapeState::NotYetEscaped => 0.,
            EscapeState::Bounded => 0.,
            EscapeState::Periodic { period: n, .. } => -(n as IterCount),
            EscapeState::Escaped {
                iters,
                final_value: z,
            } => {
                let u = self.escape_radius().log2();
                let v = z.norm_sqr().log2();
                let residual = (v / u).log2();
                (iters as IterCount) - (residual as IterCount)
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

#[derive(Clone, Copy, Debug)]
pub struct Exponential {
    point_grid: PointGrid,
    point_grid_child: PointGrid,
    max_iter: Period,
}

impl Exponential {
    const DEFAULT_MIN_X: Float = -7.;
    const DEFAULT_MAX_X: Float = 7.;
    const DEFAULT_MIN_Y: Float = -7.;
    const DEFAULT_MAX_Y: Float = 7.;
    fractal_impl!();
}

impl ParameterPlane for Exponential {
    parameter_plane_impl!();
    default_name!();

    fn encode_escape_result(&self, state: EscapeState, _base_param: ComplexNum) -> IterCount {
        match state {
            EscapeState::NotYetEscaped => 0.,
            EscapeState::Bounded => 0.,
            EscapeState::Periodic { period: n, .. } => -(n as IterCount),
            EscapeState::Escaped {
                iters: iter,
                final_value: z,
            } => {
                if z.re < 0. {
                    return -1.;
                }
                if z.is_infinite() {
                    return (iter + 1) as IterCount;
                }
                let u = slog(self.escape_radius());
                let v = slog(z.norm_sqr());
                let residual = v - u;
                (iter as IterCount) - (residual as IterCount)
            }
        }
    }

    fn map(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum {
        z.exp() + c
    }

    fn param_map(&self, c: ComplexNum) -> ComplexNum {
        c
    }
}
