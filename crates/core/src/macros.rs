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
    ($param_name: ident, $param_value: expr, $bounds_fn: ident) => {
        fn default() -> Self
        {
            let bounds = $bounds_fn($param_value);
            let point_grid = PointGrid::new_by_res_y(1024, bounds);
            Self {
                point_grid,
                max_iter: 1024,
                $param_name: $param_value,
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
        $crate::macros::point_grid_getters!();

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
macro_rules! param_map {
    () => {
        #[inline]
        fn param_map(&self, t: Cplx) -> Self::Param
        {
            t
        }

        #[inline]
        fn param_map_d(&self, t: Cplx) -> (Self::Param, Self::Deriv)
        {
            (t, ONE)
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
            if let Some(struct_name) = parts.last() {
                format!("{struct_name}")
            } else {
                "Unknown".to_owned()
            }
        }
    };
}

#[macro_export]
macro_rules! default_bounds {
    () => {
        #[inline]
        fn default_bounds(&self) -> Bounds
        {
            Self::DEFAULT_BOUNDS
        }
    };
    ($min_x: expr, $max_x: expr, $min_y: expr, $max_y: expr) => {
        #[inline]
        fn default_bounds(&self) -> Bounds
        {
            Bounds {
                min_x: $min_x,
                max_x: $max_x,
                min_y: $min_y,
                max_y: $max_y,
            };
        }
    };
    ($bounds: expr) => {
        #[inline]
        fn default_bounds(&self) -> Bounds
        {
            $bounds
        }
    };
}

#[macro_export]
macro_rules! default_bounds_impl {
    ($struct: ty) => {
        impl FamilyDefaults for $struct
        {
            default_bounds!();
        }
    };
    ($struct: ty $(,$args:expr)*) => {
        impl FamilyDefaults for $struct
        {
            default_bounds!($($args),*);
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
            _base_param: &Self::Param,
        ) -> PointInfo<Self::Deriv>
        {
            if z.is_nan() {
                return PointInfo::Escaping {
                    potential: f64::from(iters) - 1.,
                    phase: None,
                };
            }

            let u = self.escape_radius().ln();
            let v = z.norm_sqr().ln();
            let residual = (v / u).log($degree);
            let potential = IterCount::from(iters) - IterCount::from(residual);
            PointInfo::Escaping {
                potential,
                phase: None,
            }
        }
    };
    (None, $period: expr) => {
        fn encode_escaping_point(
            &self,
            iters: Period,
            z: Self::Var,
            _base_param: &Self::Param,
        ) -> PointInfo<Self::Deriv>
        {
            if z.is_nan() {
                return PointInfo::Escaping {
                    potential: IterCount::from(iters - $period),
                    phase: Some(iters % $period),
                };
            }

            let u = self.escape_radius().ln();
            let v = z.norm_sqr().ln();
            let residual = (v / u).log2();
            let potential =
                ($period as IterCount).mul_add(-IterCount::from(residual), IterCount::from(iters));
            PointInfo::Escaping {
                potential,
                phase: Some(iters % $period),
            }
        }
    };
    ($degree: expr, $period: expr) => {
        fn encode_escaping_point(
            &self,
            iters: Period,
            z: Self::Var,
            _base_param: &Self::Param,
        ) -> PointInfo<Self::Deriv>
        {
            if z.is_nan() {
                return PointInfo::Escaping {
                    potential: IterCount::from(iters - $period),
                    phase: Some(iters % $period),
                };
            }

            let u = self.escape_radius().ln();
            let v = z.norm_sqr().ln();
            let residual = (v / u).log($degree);
            let potential =
                ($period as IterCount).mul_add(-IterCount::from(residual), IterCount::from(iters));
            PointInfo::Escaping {
                potential,
                phase: Some(iters % $period),
            }
        }
    };
}

pub use {
    basic_escape_encoding, basic_plane_impl, default_bounds, default_bounds_impl, default_name,
    fractal_impl, param_map, point_grid_getters,
};
