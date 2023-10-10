pub(crate) use fractal_common::macros::{horner, horner_monic};
pub(crate) use fractal_core::macros::*;

macro_rules! profile_imports {
    () => {
        use crate::macros::parameter_plane_impl;
        use fractal_common::consts::*;
        use fractal_common::math_utils::polynomial_roots::*;
        use fractal_common::point_grid::{Bounds, PointGrid};
        use fractal_common::prelude::*;
        use fractal_core::dynamics::covering_maps::{CoveringMap, HasDynamicalCovers};
        use fractal_core::dynamics::julia::JuliaSet;
        use fractal_core::dynamics::ParameterPlane;
        use fractal_core::macros::{
            basic_escape_encoding, basic_plane_impl, default_name, fractal_impl,
        };
        use std::any::type_name;

        #[cfg(feature = "serde")]
        use serde::{Deserialize, Serialize};
    };
}

macro_rules! parameter_plane_impl {
    () => {
        type Var = Cplx;
        type Param = Cplx;
        type MetaParam = NoParam;
        type Deriv = Cplx;
        type Child = JuliaSet<Self>;

        fractal_core::macros::basic_plane_impl!();
    };
    ($child: ty) => {
        type Var = Cplx;
        type Param = Cplx;
        type MetaParam = NoParam;
        type Deriv = Cplx;
        type Child = $child;

        fractal_core::macros::basic_plane_impl!();
    };
    ($var: ty, $param: ty, $deriv: ty, $meta_param: ty) => {
        type Var = $var;
        type Param = $param;
        type MetaParam = $meta_param;
        type Deriv = $deriv;
        type Child = JuliaSet<Self>;

        fractal_core::macros::basic_plane_impl!();
    };
    ($var: ty, $param: ty, $deriv: ty, $meta_param: ty, $child: ty) => {
        type Var = $var;
        type Param = $param;
        type MetaParam = $meta_param;
        type Deriv = $deriv;
        type Child = $child;

        fractal_core::macros::basic_plane_impl!();
    };
}

macro_rules! cplx_arr {
    ( [$($x:expr),*] ) => {
        [
            $(
                Cplx::new($x as f64, 0.0),
            )*
        ]
    };
}

pub(crate) use {cplx_arr, parameter_plane_impl, profile_imports};
