macro_rules! fractal_impl {
    () => {
        #[must_use]
        pub const fn new(res_x: usize, res_y: usize, max_iter: Period, bounds: Bounds) -> Self
        {
            let point_grid = PointGrid::new(res_x, res_y, bounds);

            Self {
                point_grid,
                max_iter,
            }
        }

        #[must_use]
        pub const fn with_res_x(res_x: usize, max_iter: Period, bounds: Bounds) -> Self
        {
            let point_grid = PointGrid::with_res_x(res_x, bounds);

            Self {
                point_grid,
                max_iter,
            }
        }

        #[must_use]
        pub const fn with_res_y(res_y: usize, max_iter: Period, bounds: Bounds) -> Self
        {
            let point_grid = PointGrid::with_res_y(res_y, bounds);

            Self {
                point_grid,
                max_iter,
            }
        }

        #[must_use]
        pub const fn new_default(res_y: usize, max_iter: Period) -> Self
        {
            let bounds = Self::DEFAULT_BOUNDS;
            Self::with_res_y(res_y, max_iter, bounds)
        }
    };
    ($min_x: expr, $max_x: expr, $min_y: expr, $max_y: expr) => {
        const DEFAULT_BOUNDS: Bounds = Bounds {
            min_x: $min_x,
            max_x: $max_x,
            min_y: $min_y,
            max_y: $max_y,
        };
        fractal_impl!();
    };
}

macro_rules! point_grid_getters {
    () => {
        #[inline]
        fn point_grid(&self) -> &PointGrid
        {
            &self.point_grid
        }

        #[inline]
        fn point_grid_mut(&mut self) -> &mut PointGrid
        {
            &mut self.point_grid
        }
    };
}

macro_rules! parameter_plane_impl {
    () => {
        type Var = ComplexNum;
        type Param = ComplexNum;
        type Deriv = ComplexNum;

        crate::macros::point_grid_getters!();

        #[inline]
        fn max_iter(&self) -> Period
        {
            self.max_iter
        }

        #[inline]
        fn max_iter_mut(&mut self) -> &mut Period
        {
            &mut self.max_iter
        }

        #[inline]
        fn set_max_iter(&mut self, new_max_iter: Period)
        {
            self.max_iter = new_max_iter
        }
    };
}

macro_rules! default_name {
    () => {
        fn name(&self) -> String
        {
            let full_struct_name = type_name::<Self>();
            let parts: Vec<&str> = full_struct_name.split("::").collect();
            if let Some(struct_name) = parts.last()
            {
                format!("{struct_name}")
            }
            else
            {
                "Unknown".to_owned()
            }
        }
    };
}

// macro_rules! covers_menu_marked_cycles {
//     ($x: expr) => {
//         ui.menu_button("Marked Periodic Point", |ui| {
//             if ui.button("Period 1").clicked()
//             {
//                 self.change_fractal(|res, iter| {
//                     QuadRatPer2::new_default(res, iter).marked_cycle_curve(1)
//                 });
//             }
//             else
//             {
//                 return;
//             }
//             self.consume_click();
//             ui.close_menu();
//         });
//     };
// }

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

macro_rules! profile_imports {
    () => {
        use crate::dynamics::covering_maps::{CoveringMap, HasDynamicalCovers};
        use crate::dynamics::ParameterPlane;
        use crate::macros::{default_name, fractal_impl, parameter_plane_impl};
        use crate::point_grid::{Bounds, PointGrid};
        use crate::types::*;
        use std::any::type_name;
    };
}

pub(crate) use {
    default_name, fractal_impl, parameter_plane_impl, point_grid_getters, profile_imports,
};

pub(crate) use {max, min};
