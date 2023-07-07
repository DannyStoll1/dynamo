#[macro_export]
macro_rules! fractal_impl {
    () => {
        fn default() -> Self
        {
            let bounds = Self::DEFAULT_BOUNDS;
            let point_grid = PointGrid::new_by_res_y(1024, bounds);
            Self {
                point_grid,
                max_iter: 1024,
            }
        }
    };
    ($bounds: expr) => {
        fn default() -> Self
        {
            let point_grid = PointGrid::new_by_res_y(1024, $bounds);
            Self {
                point_grid,
                max_iter: 1024,
            }
        }
    };
    ($param_name: ident, $param_value: expr) => {
        fn default() -> Self
        {
            let bounds = Self::DEFAULT_BOUNDS;
            let point_grid = PointGrid::new_by_res_y(1024, bounds);
            Self {
                point_grid,
                max_iter: 1024,
                $param_name: $param_value,
            }
        }
    };
    ($min_x: expr, $max_x: expr, $min_y: expr, $max_y: expr) => {
        fn default() -> Self
        {
            let bounds = Bounds {
                min_x: $min_x,
                max_x: $max_x,
                min_y: $min_y,
                max_y: $max_y,
            };
            let point_grid = PointGrid::new_by_res_y(1024, bounds);
            Self {
                point_grid,
                max_iter: 1024,
            }
        }
    };
}

#[macro_export]
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

        #[inline]
        fn with_point_grid(mut self, point_grid: PointGrid) -> Self
        {
            self.point_grid = point_grid;
            self
        }
    };
}

#[macro_export]
macro_rules! basic_plane_impl {
    () => {
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

        #[inline]
        fn with_max_iter(mut self, max_iter: Period) -> Self
        {
            self.max_iter = max_iter;
            self
        }
    };
}

#[macro_export]
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

#[macro_export]
macro_rules! basic_escape_encoding {
    ($degree: expr) => {
        fn encode_escaping_point(
            &self,
            iters: Period,
            z: Self::Var,
            _base_param: Self::Param,
        ) -> PointInfo<Self::Deriv>
        {
            if z.is_nan()
            {
                return PointInfo::Escaping {
                    potential: f64::from(iters) - 1.,
                };
            }

            let u = self.escape_radius().log2();
            let v = z.norm_sqr().log2();
            let residual = (v / u).log($degree);
            let potential = IterCount::from(iters) - IterCount::from(residual);
            PointInfo::Escaping { potential }
        }
    };
    ($degree: expr, $period: expr) => {
        fn encode_escaping_point(
            &self,
            iters: Period,
            z: Self::Var,
            _base_param: Self::Param,
        ) -> PointInfo<Self::Deriv>
        {
            if z.is_nan()
            {
                return PointInfo::Escaping {
                    potential: f64::from(iters) - 1.,
                };
            }

            let u = self.escape_radius().log2();
            let v = z.norm_sqr().log2();
            let residual = (v / u).log($degree);
            let potential =
                ($period as IterCount).mul_add(-IterCount::from(residual), IterCount::from(iters));
            PointInfo::Escaping { potential }
        }
    };
}

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

pub use {basic_escape_encoding, basic_plane_impl, default_name, fractal_impl, point_grid_getters};

pub(crate) use {max, min};
