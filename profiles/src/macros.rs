pub(crate) use dynamo_common::macros::{horner, horner_monic};
pub(crate) use dynamo_core::macros::*;

macro_rules! profile_imports {
    () => {
        use crate::macros::parameter_plane_impl;
        use dynamo_common::consts::*;
        use dynamo_common::math_utils::polynomial_roots::*;
        use dynamo_common::point_grid::{Bounds, PointGrid};
        use dynamo_common::prelude::*;
        use dynamo_core::dynamics::covering_maps::{CoveringMap, HasDynamicalCovers};
        use dynamo_core::dynamics::julia::JuliaSet;
        use dynamo_core::dynamics::ParameterPlane;
        use dynamo_core::macros::{
            basic_escape_encoding, basic_plane_impl, default_bounds, default_name, dynamo_impl,
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

        dynamo_core::macros::basic_plane_impl!();
    };
    ($child: ty) => {
        type Var = Cplx;
        type Param = Cplx;
        type MetaParam = NoParam;
        type Deriv = Cplx;
        type Child = $child;

        dynamo_core::macros::basic_plane_impl!();
    };
    ($var: ty, $param: ty, $deriv: ty, $meta_param: ty) => {
        type Var = $var;
        type Param = $param;
        type MetaParam = $meta_param;
        type Deriv = $deriv;
        type Child = JuliaSet<Self>;

        dynamo_core::macros::basic_plane_impl!();
    };
    ($var: ty, $param: ty, $deriv: ty, $meta_param: ty, $child: ty) => {
        type Var = $var;
        type Param = $param;
        type MetaParam = $meta_param;
        type Deriv = $deriv;
        type Child = $child;

        dynamo_core::macros::basic_plane_impl!();
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
