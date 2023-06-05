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

macro_rules! parameter_plane_impl {
    () => {
        type Var = Cplx;
        type Param = Cplx;
        type MetaParam = NoParam;
        type Deriv = Cplx;
        type Child = JuliaSet<Self>;

        crate::macros::basic_plane_impl!();
    };
    ($child: ty) => {
        type Var = Cplx;
        type Param = Cplx;
        type MetaParam = NoParam;
        type Deriv = Cplx;
        type Child = $child;

        crate::macros::basic_plane_impl!();
    };
    ($var: ty, $param: ty, $deriv: ty, $meta_param: ty) => {
        type Var = $var;
        type Param = $param;
        type MetaParam = $meta_param;
        type Deriv = $deriv;
        type Child = JuliaSet<Self>;

        crate::macros::basic_plane_impl!();
    };
    ($var: ty, $param: ty, $deriv: ty, $meta_param: ty, $child: ty) => {
        type Var = $var;
        type Param = $param;
        type MetaParam = $meta_param;
        type Deriv = $deriv;
        type Child = $child;

        crate::macros::basic_plane_impl!();
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

macro_rules! fractal_menu_button {
    ($self: ident, $ui: ident, $name: expr, $fractal: ty) => {
        if $ui.button($name).clicked()
        {
            $self.change_fractal(
                || <$fractal>::default(),
                <$fractal as ParameterPlane>::Child::from,
            );
            $self.interface.consume_click();
            $ui.close_menu();
            return;
        }
    };
    ($self: ident, $ui: ident, $name: expr, $fractal: ty, $covering: ident, $($periods: expr),+) => {
        if $ui.button($name).clicked()
        {
            $self.change_fractal(|| <$fractal>::default().$covering($($periods),+), JuliaSet::from);
            $self.interface.consume_click();
            $ui.close_menu();
            return;
        }
    };
    ($self: ident, $ui: ident, $name: expr, $fractal: ident, $child: ident) => {
        if $ui.button($name).clicked()
        {
            $self.change_fractal(|| $fractal::default(), $child::from);
            $self.interface.consume_click();
            $ui.close_menu();
            return;
        }
    };
}

macro_rules! fractal_menu_button_mc {
    ($self: ident, $ui: ident, $fractal: ty, $period: expr) => {
        fractal_menu_button!(
            $self,
            $ui,
            format!("Period {}", $period),
            $fractal,
            marked_cycle_curve,
            $period
        );
    };
}

macro_rules! fractal_menu_button_dyn {
    ($self: ident, $ui: ident, $fractal: ty, $period: expr) => {
        fractal_menu_button!(
            $self,
            $ui,
            format!("Period {}", $period),
            $fractal,
            dynatomic_curve,
            $period
        );
    };
}

macro_rules! fractal_menu_button_mis {
    ($self: ident, $ui: ident, $fractal: ty, $preperiod: expr, $period: expr) => {
        fractal_menu_button!(
            $self,
            $ui,
            format!("Preperiod {}, Period {}", $preperiod, $period),
            $fractal,
            misiurewicz_curve,
            $preperiod,
            $period
        );
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

macro_rules! horner {
    ($c: expr) => ( $c );
    ($var: expr, $c: expr ) => ( $c );
    ($var: expr, $c: expr, $($cs:expr),+) => {
        $c + $var * horner!($var, $($cs),+)
    };
}

macro_rules! horner_monic {
    () => ( 1. );
    ($c: expr) => ( $c );
    ($var: expr, $c: expr ) => ( $c + $var );
    ($var: expr, $c: expr, $($cs:expr),+) => {
        $c + $var * horner_monic!($var, $($cs),+)
    };
}

macro_rules! profile_imports {
    () => {
        use crate::consts::*;
        use crate::dynamics::covering_maps::{CoveringMap, HasDynamicalCovers};
        use crate::dynamics::julia::JuliaSet;
        use crate::dynamics::ParameterPlane;
        use crate::macros::{basic_plane_impl, default_name, fractal_impl, parameter_plane_impl};
        use crate::point_grid::{Bounds, PointGrid};
        use crate::types::*;
        use std::any::type_name;

        #[cfg(feature = "serde")]
        use serde::{Deserialize, Serialize};
    };
}

pub(crate) use {
    basic_escape_encoding, basic_plane_impl, default_name, fractal_impl, fractal_menu_button,
    fractal_menu_button_dyn, fractal_menu_button_mc, fractal_menu_button_mis, horner, horner_monic,
    parameter_plane_impl, point_grid_getters, profile_imports,
};

pub(crate) use {max, min};
