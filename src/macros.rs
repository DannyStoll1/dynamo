macro_rules! fractal_impl {
    () => {
        pub fn new(
            res_x: usize,
            res_y: usize,
            max_iter: Period,
            bounds: Bounds,
        ) -> Self {
            let point_grid = PointGrid::new(res_x, res_y, bounds);
            let point_grid_child = PointGrid::new(res_x, res_y, Self::JULIA_BOUNDS);

            Self {
                point_grid,
                point_grid_child,
                max_iter,
            }
        }

        pub fn with_res_x(
            res_x: usize,
            max_iter: Period,
            bounds: Bounds,
        ) -> Self {
            let point_grid = PointGrid::with_res_x(res_x, bounds);
            let point_grid_child = PointGrid::with_res_x(res_x, Self::JULIA_BOUNDS);

            Self {
                point_grid,
                point_grid_child,
                max_iter,
            }
        }

        pub fn with_res_y(
            res_y: usize,
            max_iter: Period,
            bounds: Bounds
        ) -> Self {
            let point_grid = PointGrid::with_res_y(res_y, bounds);
            let point_grid_child = PointGrid::with_res_y(res_y, Self::JULIA_BOUNDS);

            Self {
                point_grid,
                point_grid_child,
                max_iter,
            }
        }

        pub fn new_default(res_y: usize, max_iter: Period) -> Self {
            let bounds = Self::DEFAULT_BOUNDS;
            Self::with_res_y(
                res_y,
                max_iter,
                bounds,
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

        fn max_iter(&self) -> Period {
            self.max_iter
        }

        fn max_iter_mut(&mut self) -> &mut Period {
            &mut self.max_iter
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

// macro_rules! debug {
//     ($x:expr) => { println!("{} = {:?}", stringify!($x), $x ); };
//     ($x:expr, $($xs:expr),+) => {
//         debug!($x);
//         debug!($xs);
//     };
// }

pub(crate) use {max, min};
