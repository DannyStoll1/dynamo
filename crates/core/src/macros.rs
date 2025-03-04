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
                compute_mode: ComputeMode::default(),
            }
        }
    };
    ($param_name: ident, $param_value: expr_2021) => {
        fn default() -> Self
        {
            let bounds = Self::DEFAULT_BOUNDS;
            let point_grid = PointGrid::new_by_res_y(1024, bounds);
            Self {
                point_grid,
                max_iter: 1024,
                compute_mode: ComputeMode::default(),
                $param_name: $param_value,
            }
        }
    };
    ($param_name: ident, $param_value: expr_2021, $bounds_fn: ident) => {
        fn default() -> Self
        {
            let bounds = $bounds_fn($param_value);
            let point_grid = PointGrid::new_by_res_y(1024, bounds);
            Self {
                point_grid,
                max_iter: 1024,
                compute_mode: ComputeMode::default(),
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

        #[inline]
        fn compute_mode(&self) -> ComputeMode
        {
            self.compute_mode
        }

        #[inline]
        fn compute_mode_mut(&mut self) -> &mut ComputeMode
        {
            &mut self.compute_mode
        }

        #[inline]
        fn set_compute_mode(&mut self, compute_mode: ComputeMode)
        {
            self.compute_mode = compute_mode;
        }
    };
}

#[macro_export]
macro_rules! basic_plane_impl {
    () => {
        $crate::macros::point_grid_getters!();

        #[inline]
        fn max_iter(&self) -> IterCount
        {
            self.max_iter
        }

        #[inline]
        fn max_iter_mut(&mut self) -> &mut IterCount
        {
            &mut self.max_iter
        }

        #[inline]
        fn set_max_iter(&mut self, new_max_iter: IterCount)
        {
            self.max_iter = new_max_iter
        }

        #[inline]
        fn with_max_iter(mut self, max_iter: IterCount) -> Self
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
    ($min_x: expr_2021, $max_x: expr_2021, $min_y: expr_2021, $max_y: expr_2021) => {
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
    ($bounds: expr_2021) => {
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
    ($struct: ty $(,$args:expr_2021)*) => {
        impl FamilyDefaults for $struct
        {
            default_bounds!($($args),*);
        }
    };
}

#[macro_export]
macro_rules! basic_escape_encoding {
    ($degree: expr_2021) => {
        fn encode_escaping_point(
            &self,
            iters: IterCount,
            z: Self::Var,
            _base_param: &Self::Param,
        ) -> PointInfo<Self::Deriv>
        {
            if z.is_nan() {
                return PointInfo::Escaping {
                    potential: iters as IterCountSmooth - 1.,
                    phase: None,
                };
            }

            let u = self.escape_radius().ln();
            let v = z.norm_sqr().ln();
            let residual = (v / u).log($degree);
            let potential = (iters as IterCountSmooth) - IterCountSmooth::from(residual);
            PointInfo::Escaping {
                potential,
                phase: None,
            }
        }
    };
    (None, $period: expr_2021) => {
        fn encode_escaping_point(
            &self,
            iters: IterCount,
            z: Self::Var,
            _base_param: &Self::Param,
        ) -> PointInfo<Self::Deriv>
        {
            let phase = Some((iters % $period) as Period);
            if z.is_nan() {
                return PointInfo::Escaping {
                    potential: (iters - $period) as IterCountSmooth,
                    phase,
                };
            }

            let u = self.escape_radius().ln();
            let v = z.norm_sqr().ln();
            let residual = (v / u).log2();
            let potential = ($period as IterCountSmooth)
                .mul_add(-IterCountSmooth::from(residual), (iters as IterCountSmooth));
            PointInfo::Escaping { potential, phase }
        }
    };
    ($degree: expr_2021, $period: expr_2021) => {
        fn encode_escaping_point(
            &self,
            iters: IterCount,
            z: Self::Var,
            _base_param: &Self::Param,
        ) -> PointInfo<Self::Deriv>
        {
            if z.is_nan() {
                return PointInfo::Escaping {
                    potential: IterCountSmooth::from(iters - $period),
                    phase: Some(iters % $period),
                };
            }

            let u = self.escape_radius().ln();
            let v = z.norm_sqr().ln();
            let residual = (v / u).log($degree);
            let potential = ($period as IterCountSmooth)
                .mul_add(-IterCountSmooth::from(residual), (iters as IterCountSmooth));
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
