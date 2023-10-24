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
        use dynamo_core::dynamics::{
            Equipotential, EscapeEncoding, ExternalRays, InfinityFirstReturnMap, ParameterPlane,
        };
        use dynamo_core::macros::{
            basic_escape_encoding, basic_plane_impl, default_bounds, default_name, fractal_impl,
        };
        use num_traits::ops::mul_add::MulAdd;
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

macro_rules! degree_impl {
    ($plane: ty, $deg: expr) => {
        impl InfinityFirstReturnMap for $plane
        {
            degree_impl!($deg);
        }
        impl EscapeEncoding for $plane {}
        impl ExternalRays for $plane {}
    };
    ($plane: ty, $deg: expr, $period: expr) => {
        impl InfinityFirstReturnMap for $plane
        {
            degree_impl!($deg, $period);
        }
        impl EscapeEncoding for $plane {}
        impl ExternalRays for $plane {}
    };
    ($deg: expr) => {
        #[inline]
        fn degree(&self) -> AngleNum
        {
            $deg
        }
        #[inline]
        fn degree_real(&self) -> Real
        {
            $deg as Real
        }
    };
    ($deg: expr, $period: expr) => {
        degree_impl!($deg);
        #[inline]
        fn escaping_period(&self) -> Period
        {
            $period
        }
    };
    ($deg: expr, $period: expr, $coeff: expr) => {
        degree_impl!($deg, $period);

        #[allow(unused_variables)]
        #[inline]
        fn escape_coeff(&self, c: Self::Param) -> Cplx
        {
            Cplx::from($coeff)
        }

        #[allow(unused_variables)]
        #[inline]
        fn escape_coeff_d(&self, c: Self::Param) -> (Cplx, Cplx)
        {
            (Cplx::from($coeff), ZERO)
        }
    };
}

macro_rules! degree_impl_transcendental {
    ($plane: ty) => {
        impl InfinityFirstReturnMap for $plane
        {
            degree_impl_transcendental!();
        }

        impl EscapeEncoding for $plane
        {
            fn encode_escape_result(
                &self,
                state: EscapeState<Self::Var, Self::Deriv>,
                base_param: Self::Param,
            ) -> PointInfo<Self::Var, Self::Deriv>
            {
                match state
                {
                    EscapeState::NotYetEscaped | EscapeState::Bounded => PointInfo::Wandering,
                    EscapeState::Periodic(data) => PointInfo::Periodic(data),
                    EscapeState::KnownPotential(data) => PointInfo::PeriodicKnownPotential(data),
                    EscapeState::Escaped { iters, final_value } =>
                    {
                        self.encode_escaping_point(iters, final_value, base_param)
                    }
                }
            }

            fn encode_escaping_point(
                &self,
                iters: Period,
                z: Cplx,
                _base_param: Cplx,
            ) -> PointInfo<Self::Var, Self::Deriv>
            {
                use dynamo_common::math_utils::slog;
                if z.is_nan()
                {
                    return PointInfo::Escaping {
                        potential: f64::from(iters) - 1.,
                    };
                }
                if z.is_infinite()
                {
                    return PointInfo::Escaping {
                        potential: f64::from(iters) + 1.,
                    };
                }
                let u = slog(self.escape_radius());
                let v = slog(z.norm_sqr());
                let residual = v - u;
                let potential = f64::from(iters) - (residual as IterCount);
                PointInfo::Escaping { potential }
            }
        }

        impl ExternalRays for $plane {}
    };
    () => {
        #[inline]
        fn degree(&self) -> AngleNum
        {
            0
        }
        #[inline]
        fn degree_real(&self) -> Real
        {
            Real::NAN
        }
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

#[allow(unused_macros)]
macro_rules! ext_ray_impl_nonmonic_conj {
    () => {
        fn external_ray_helper(&self, angle: RationalAngle) -> Option<Vec<Cplx>>
        {
            use dynamo_common::math_utils::newton::{
                error::Error::NanEncountered, find_target_newton_err_d,
            };

            const R: Real = 16.0;
            let escape_radius_log2 = R.log2() * self.degree_real().abs();

            let deg_real = self.degree_real().abs();
            if deg_real.is_nan() || self.degree() <= 1
            {
                return None;
            }
            let deg_log2 = deg_real.log2();

            let pixel_width = self.point_grid().pixel_width() * 0.03;
            let error = self.point_grid().res_x as Real * 1e-8;

            // let base_point = escape_radius * angle.to_circle();
            // Arbitrary starting guess that is likely to escape
            let base_point: Cplx = 65.0 * angle.to_circle();
            let mut t_list = vec![];

            // degree of each additional batch of iterations
            let deg = self.degree();

            // Target angle for the composite map at each step.
            // Initialized to value after self.escaping_phase() iterations.
            let mut target_angle = self.angle_map_large_param(angle);

            for k in 0..RAY_DEPTH
            {
                // Relative log2-norms of targets
                // jth target norm = escape_radius^deg^(-j/S)
                // u_j = log2(escape_radius) * deg^(-j/S)
                let us = (0..RAY_SHARPNESS).map(|j| {
                    escape_radius_log2
                        * ((-Real::from(j) * deg_log2) / Real::from(RAY_SHARPNESS)).exp2()
                });

                let v = target_angle.to_circle();
                let targets = us.map(|u| u.exp2() * v);

                let mut t_curr = *t_list.last().unwrap_or(&base_point);
                let mut dist: Real;

                let num_iters = k * self.escaping_period() + self.escaping_phase();

                let fk_and_dfk = |t: Cplx| {
                    let (c, dc_dt) = self.param_map_d(t);
                    let (mut z, mut dz_dt, dz_dc) = self.start_point_d(t, c);
                    dz_dt += dz_dc * dc_dt;

                    let (a, da_dc) = self.escape_coeff_d(c);
                    let da_dt = da_dc * dc_dt;
                    // let a_arg = a.ln();
                    // let a_arg_d = a_d / a;
                    let pow = 1. / (self.degree_real() - 1.);
                    let conj = a.powf(pow);
                    let conj_d = conj * da_dt / a * pow;

                    for _i in 0..self.escaping_phase()
                    {
                        let (f, df_dz, df_dc) = self.gradient(z, c);
                        dz_dt = dz_dt * df_dz + df_dc * dc_dt;
                        z = f;
                    }

                    if num_iters > self.escaping_phase()
                    {
                        // Conjugate to make f monic
                        z /= conj;
                        dz_dt = (dz_dt - z * conj_d) / conj;
                    }

                    for _i in self.escaping_phase()..num_iters
                    {
                        let (f, df_dz, df_dc) = self.gradient(z, c);
                        dz_dt = dz_dt * df_dz + df_dc * dc_dt;
                        z = f;
                    }

                    if num_iters > self.escaping_phase()
                    {
                        z *= conj;
                        dz_dt = dz_dt * conj + z * conj_d;
                    }

                    (z, dz_dt)
                };

                for target in targets
                {
                    match find_target_newton_err_d(fk_and_dfk, t_curr, target, error)
                    {
                        Ok((sol, t_k, d_k)) =>
                        {
                            t_curr = sol;

                            if t_curr.is_nan()
                            {
                                return Some(t_list);
                            }

                            t_list.push(t_curr);

                            dist = (2. * t_k.norm() * (t_k.norm()).log(deg_real)) / d_k.norm();
                            if dist < pixel_width
                            {
                                return Some(t_list);
                            }
                        }
                        Err(NanEncountered) =>
                        {
                            return Some(t_list);
                        }
                        _ =>
                        {}
                    }
                }
                target_angle *= deg;
            }

            Some(t_list)
        }
    };
    ($plane: ty) => {
        impl ExternalRays for $plane
        {
            ext_ray_impl_nonmonic!();
        }
    };
}

macro_rules! ext_ray_impl_nonmonic {
    () => {
        fn external_ray_helper(&self, angle: RationalAngle) -> Option<Vec<Cplx>>
        {
            use dynamo_common::math_utils::newton::{
                error::Error::NanEncountered, find_target_newton_err_d,
            };
            const R: Real = 16.0;
            let escape_radius_log2 = R.log2() * self.degree_real().abs();

            let deg_real = self.degree_real().abs();
            if deg_real.is_nan()
            {
                return None;
            }
            let deg_log2 = deg_real.log2();

            let pixel_width = self.point_grid().pixel_width() * 0.03;
            let error = self.point_grid().res_x as Real * 1e-8;

            // let base_point = escape_radius * angle.to_circle();
            // Arbitrary starting guess that is likely to escape
            let base_point: Cplx = 65.0 * angle.to_circle();
            let mut t_list = vec![];

            // degree of each additional batch of iterations
            let deg = self.degree();

            // Target angle for the composite map at each step.
            // Initialized to value after self.escaping_phase() iterations.
            let mut target_angle = self.angle_map_large_param(angle);

            for k in 0..RAY_DEPTH
            {
                // Relative log2-norms of targets
                // jth target norm = escape_radius^deg^(-j/S)
                // u_j = log2(escape_radius) * deg^(-j/S)
                let us = (0..RAY_SHARPNESS).map(|j| {
                    escape_radius_log2
                        * ((-Real::from(j) * deg_log2) / Real::from(RAY_SHARPNESS)).exp2()
                });

                let v = target_angle.to_circle();
                let targets = us.map(|u| u.exp2() * v);

                let mut t_curr = *t_list.last().unwrap_or(&base_point);
                let mut dist: Real;

                let num_iters = k * self.escaping_period() + self.escaping_phase();

                let fk_and_dfk = |t: Cplx| {
                    let (c, dc_dt) = self.param_map_d(t);
                    let (mut z, mut dz_dt, dz_dc) = self.start_point_d(t, c);
                    dz_dt += dz_dc * dc_dt;

                    let (a, a_d) = self.escape_coeff_d(c);
                    let a_arg = a.ln();
                    let a_arg_d = a_d / a;

                    // Correction term to fix the angle
                    let mut corr_arg = ZERO;
                    let mut corr_arg_d = ZERO;

                    for _i in 0..num_iters
                    {
                        let (f, df_dz, df_dc) = self.gradient(z, c);
                        dz_dt = dz_dt * df_dz + df_dc * dc_dt;
                        z = f;
                    }

                    for _i in
                        (self.escaping_phase()..num_iters).step_by(self.escaping_period() as usize)
                    {
                        corr_arg = corr_arg * self.degree_real() + a_arg;
                        corr_arg.im %= TAU;
                        corr_arg_d = corr_arg_d * self.degree_real() + a_arg_d;
                    }

                    if num_iters > self.escaping_phase()
                    {
                        let corr = corr_arg.exp();
                        let corr_d = corr * corr_arg_d;

                        let zcorr = z / corr;
                        let zcorr_d = (dz_dt - zcorr * corr_d) / corr;

                        return (zcorr, zcorr_d);
                    }
                    (z.into(), dz_dt.into())
                };

                for target in targets
                {
                    match find_target_newton_err_d(fk_and_dfk, t_curr, target, error)
                    {
                        Ok((sol, t_k, d_k)) =>
                        {
                            t_curr = sol;

                            if t_curr.is_nan()
                            {
                                return Some(t_list);
                            }

                            t_list.push(t_curr);

                            dist = (2. * t_k.norm() * (t_k.norm()).log(deg_real)) / d_k.norm();
                            if dist < pixel_width
                            {
                                return Some(t_list);
                            }
                        }
                        Err(NanEncountered) =>
                        {
                            return Some(t_list);
                        }
                        _ =>
                        {}
                    }
                }
                target_angle *= deg;
            }

            Some(t_list)
        }
    };
}

pub(crate) use {
    cplx_arr, degree_impl, degree_impl_transcendental, ext_ray_impl_nonmonic, parameter_plane_impl,
    profile_imports,
};
