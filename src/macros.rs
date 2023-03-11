macro_rules! fractal_impl {
    () => {
        const EPSILON: f64 = 1e-12;

        pub fn new(
            res_x: usize,
            max_iter: Period,
            min_x: f64,
            max_x: f64,
            min_y: f64,
            max_y: f64,
        ) -> Self {
            let point_grid = PointGrid::new_infer(res_x, min_x, max_x, min_y, max_y);
            let point_grid_child = PointGrid::new_infer(res_x, -5., 5., -5., 5.);

            Self {
                point_grid,
                point_grid_child,
                max_iter,
            }
        }

        pub fn new_default(res_x: usize, max_iter: Period) -> Self {
            Self::new(
                res_x,
                max_iter,
                Self::DEFAULT_MIN_X,
                Self::DEFAULT_MAX_X,
                Self::DEFAULT_MIN_Y,
                Self::DEFAULT_MAX_Y,
            )
        }
    };
}

macro_rules! point_grid_getters {
    () => {
        fn point_grid(&self) -> PointGrid {
            self.point_grid
        }
        fn point_grid_mut(&mut self) -> &mut PointGrid {
            &mut self.point_grid
        }
        fn point_grid_child(&self) -> PointGrid {
            self.point_grid_child
        }
        fn point_grid_child_mut(&mut self) -> &mut PointGrid {
            &mut self.point_grid_child
        }
    };
}

macro_rules! parameter_plane_impl {
    () => {
        crate::macros::point_grid_getters!();

        fn stop_condition(&self, iter: Period, z: ComplexNum) -> EscapeState {
            if iter > self.max_iter {
                return EscapeState::Bounded;
            }

            let r = z.norm_sqr();
            if r > Self::ESCAPE_RADIUS {
                EscapeState::Escaped{iters: iter, final_value: z}
            } else {
                EscapeState::NotYetEscaped
            }
        }

        fn check_periodicity(&self, iter: Period, z0: ComplexNum, z1: ComplexNum, base_param: ComplexNum) -> EscapeState {
            if iter > self.max_iter {
                return EscapeState::Bounded;
            }

            let r = z1.norm_sqr();
            if r > Self::ESCAPE_RADIUS {
                EscapeState::Escaped{iters: 2*iter, final_value: z1}
            } else if (z1 - z0).norm_sqr() < Self::EPSILON {
                if let Some(period) = self.compute_period(z1, base_param, Self::EPSILON, iter as usize) {
                    EscapeState::Periodic{preperiod: iter, period}
                }
                else {
                    EscapeState::NotYetEscaped
                }
            }
            else {
                EscapeState::NotYetEscaped
            }
        }
    };
}

macro_rules! default_name {
    () => {
        fn name(&self) -> String {
            let full_struct_name = type_name::<Self>();
            let parts: Vec<&str> = full_struct_name.split("::").collect();
            if let Some(struct_name) = parts.last() {
                format!("{struct_name}")
            } else {
                "Unknown".to_owned()
            }
        }
    };
}

pub(crate) use {fractal_impl, parameter_plane_impl, point_grid_getters, default_name};

macro_rules! max {
    ($x:expr) => ( $x );
    ($x:expr, $($xs:expr),+) => {
        std::cmp::max($x, max!( $($xs),+ ))
    };
}

macro_rules! min {
    ($x:expr) => ( $x );
    ($x:expr, $($xs:expr),+) => {
        std::cmp::min($x, min!( $($xs),+ ))
    };
}

pub(crate) use {max, min};



// }
//
// impl Mandelbrot {
//     const ESCAPE_RADIUS: f64 = 1e12_f64;
//     const DEFAULT_MIN_X: f64 = -2.2;
//     const DEFAULT_MAX_X: f64 = 0.65;
//     const DEFAULT_MIN_Y: f64 = -1.4;
//     const DEFAULT_MAX_Y: f64 = 1.4;
//
//     pub fn new(
//         res_x: usize,
//         max_iter: i32,
//         min_x: f64,
//         max_x: f64,
//         min_y: f64,
//         max_y: f64,
//     ) -> Self {
//         let point_grid = PointGrid::new_infer(res_x, min_x, max_x, min_y, max_y);
//
//         Self {
//             point_grid,
//             max_iter,
//         }
//     }
//
//     pub fn new_default(res_x: usize, max_iter: i32) -> Self {
//         Self::new(
//             res_x,
//             max_iter,
//             Self::DEFAULT_MIN_X,
//             Self::DEFAULT_MAX_X,
//             Self::DEFAULT_MIN_Y,
//             Self::DEFAULT_MAX_Y,
//         )
//     }
// }
//
// impl HasDynamicalCovers for Mandelbrot {
//     fn marked_cycle_curve(self, period: Period) -> CoveringMap<Self> {
//         let param_map: fn(ComplexNum) -> ComplexNum;
//         let bounds: PointGrid;
//
//         match period {
//             1 => {
//                 param_map = |c| 0.25 - c * c;
//                 bounds = self.point_grid.with_new_bounds(-1.8, 1.8, -1.0, 1.0);
//             }
//             3 => {
//                 param_map = |c| -1.75 * (1. + 7. * c * c);
//                 bounds = self.point_grid.with_new_bounds(-0.3, 0.3, -0.5, 0.5);
//             }
//             4 => {
//                 param_map = |c| {
//                     let u = c * c;
//                     // -c - (0.75 * u + 0.25) / u
//                     -0.25 * u - 0.75 - 1. / c
//                 };
//                 bounds = self.point_grid.with_new_bounds(-2.9, 2.1, -3.75, 3.75);
//             }
//             _ => {
//                 param_map = |c| c;
//                 bounds = self.point_grid;
//             }
//         };
//         CoveringMap::new(self, param_map, bounds)
//     }
//
//     fn dynatomic_curve(self, period: Period) -> CoveringMap<Self> {
//         let param_map: fn(ComplexNum) -> ComplexNum;
//         let bounds: PointGrid;
//
//         match period {
//             1 => {
//                 param_map = |c| 0.25 - c * c;
//                 bounds = self.point_grid.with_new_bounds(-1.8, 1.8, -1.0, 1.0);
//             }
//             3 => {
//                 param_map = |c| {
//                     let c2 = c * c;
//                     let v = c2 * (c2 - 3. * c + 6.) - c - c + 2.;
//                     let u = v + 1. / (c2 - c);
//                     -0.25 * u / (c2 - c)
//                 };
//                 bounds = self.point_grid.with_new_bounds(-2.5, 3.5, -3., 3.);
//             }
//             _ => {
//                 param_map = |c| c;
//                 bounds = self.point_grid;
//             }
//         };
//         CoveringMap::new(self, param_map, bounds)
//     }
//     fn misiurewicz_curve(self, preperiod: Period, period: Period) -> CoveringMap<Self> {
//         let param_map: fn(ComplexNum) -> ComplexNum;
//         let bounds: PointGrid;
//
//         match (preperiod, period) {
//             (2, 1) => {
//                 param_map = |c| {
//                     let c2 = c * c;
//                     -2. * (c2 + 1.) / ((c2 - 1.) * (c2 - 1.))
//                 };
//                 bounds = self.point_grid.with_new_bounds(-3.5, 3.5, -3.0, 3.0);
//             }
//             (2, 2) => {
//                 param_map = |c| {
//                     let c2 = c * c;
//                     -(c2 * (c2 + c + c + 2.) - c - c + 1.) / (4. * c2)
//                 };
//                 bounds = self.point_grid.with_new_bounds(-4., 2.4, -2.5, 2.5);
//             }
//             (_, _) => {
//                 param_map = |c| c;
//                 bounds = self.point_grid;
//             }
//         };
//         CoveringMap::new(self, param_map, bounds)
//     }
// }
//
// }
//
//
