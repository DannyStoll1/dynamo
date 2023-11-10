pub(crate) use dynamo_common::macros::{horner, horner_monic};
pub(crate) use dynamo_core::macros::*;

macro_rules! profile_imports {
    () => {
        use crate::macros::parameter_plane_impl;
        use dynamo_common::math_utils::polynomial_roots::*;
        use dynamo_common::prelude::*;
        use dynamo_core::prelude::*;
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
        fn escape_coeff(&self, c: &Self::Param) -> Cplx
        {
            Cplx::from($coeff)
        }

        #[allow(unused_variables)]
        #[inline]
        fn escape_coeff_d(&self, c: &Self::Param) -> (Cplx, Cplx)
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
                state: EscapeResult<Self::Var, Self::Deriv>,
                _start: Self::Var,
                base_param: &Self::Param,
            ) -> PointInfo<Self::Deriv>
            {
                match state {
                    EscapeResult::Periodic { info, .. } => PointInfo::Periodic(info),
                    EscapeResult::KnownPotential(data) => PointInfo::PeriodicKnownPotential(data),
                    EscapeResult::Escaped { iters, final_value } => {
                        self.encode_escaping_point(iters, final_value, base_param)
                    }
                    EscapeResult::Bounded(final_value) => {
                        if final_value.norm_sqr() > 1e5 {
                            PointInfo::Wandering
                        } else {
                            PointInfo::Bounded
                        }
                    }
                    EscapeResult::Unknown => PointInfo::Unknown,
                }
            }

            fn encode_escaping_point(
                &self,
                iters: Period,
                z: Cplx,
                _base_param: &Cplx,
            ) -> PointInfo<Self::Deriv>
            {
                use dynamo_common::math_utils::slog;
                if z.is_nan() {
                    return PointInfo::Escaping {
                        potential: f64::from(iters) - 1.,
                    };
                }
                if z.is_infinite() {
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
            if deg_real.is_nan() || self.degree() <= 1 {
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

            for k in 0..RAY_DEPTH {
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

                    for _i in 0..self.escaping_phase() {
                        let (f, df_dz, df_dc) = self.gradient(z, c);
                        dz_dt = dz_dt * df_dz + df_dc * dc_dt;
                        z = f;
                    }

                    if num_iters > self.escaping_phase() {
                        // Conjugate to make f monic
                        z /= conj;
                        dz_dt = (dz_dt - z * conj_d) / conj;
                    }

                    for _i in self.escaping_phase()..num_iters {
                        let (f, df_dz, df_dc) = self.gradient(z, c);
                        dz_dt = dz_dt * df_dz + df_dc * dc_dt;
                        z = f;
                    }

                    if num_iters > self.escaping_phase() {
                        z *= conj;
                        dz_dt = dz_dt * conj + z * conj_d;
                    }

                    (z, dz_dt)
                };

                for target in targets {
                    match find_target_newton_err_d(fk_and_dfk, t_curr, target, error) {
                        Ok((sol, t_k, d_k)) => {
                            t_curr = sol;

                            if t_curr.is_nan() {
                                return Some(t_list);
                            }

                            t_list.push(t_curr);

                            dist = (2. * t_k.norm() * (t_k.norm()).log(deg_real)) / d_k.norm();
                            if dist < pixel_width {
                                return Some(t_list);
                            }
                        }
                        Err(NanEncountered) => {
                            return Some(t_list);
                        }
                        _ => {}
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

#[allow(unused_macros)]
macro_rules! ext_ray_impl_rk {
    ($step: literal, $esc: expr) => {
        fn external_ray_helper(&self, angle: RationalAngle) -> Option<Vec<Cplx>>
        {
            use dynamo_common::math_utils::runge_kutta_step;
            const R: Real = 48.0;
            const STEP_SIZE: Real = $step;
            let escape_radius: Real = $esc;
            let pixel_width = self.point_grid().pixel_width() * 0.03;

            let mut t: Cplx = R * angle.to_circle();
            let mut t_list = vec![t];

            let mut deriv_green = |t: Cplx| {
                if t.is_nan() {
                    panic!();
                }

                let (c, dc_dt) = self.param_map_d(t);
                let (mut z, mut dz_dt, dz_dc) = self.start_point_d(t, &c);
                dz_dt += dz_dc * dc_dt;

                for _ in 0..self.escaping_phase() {
                    let (f, df_dz, df_dc) = self.gradient(z, &c);
                    dz_dt = df_dz * dz_dt + df_dc * dc_dt;
                    z = f;
                }
                for iter in 0..self.max_iter() {
                    for _j in 0..self.escaping_period() {
                        if z.norm_sqr() > escape_radius {
                            let d_absz = z * dz_dt.conj();
                            let norm = d_absz.norm();
                            let direction = d_absz / norm;
                            let log_potential = norm.log2() / self.degree_real().powi(iter as i32);
                            return direction * log_potential;
                        }
                        let (f, df_dz, df_dc) = self.gradient(z, &c);
                        dz_dt = df_dz * dz_dt + df_dc * dc_dt;
                        z = f;
                    }
                }

                NAN
            };

            for _k in 0..RAY_DEPTH * RAY_SHARPNESS {
                let dt = runge_kutta_step(&mut deriv_green, t, STEP_SIZE);

                if dt.norm() < pixel_width {
                    return Some(t_list);
                }

                t -= dt;

                if t.is_nan() {
                    return Some(t_list);
                }

                t_list.push(t);
            }

            Some(t_list)
        }
    };
    ($step: literal) => {
        ext_ray_impl_rk!($step, 1e6);
    };
    () => {
        ext_ray_impl_rk!(3e-2);
    };
}

macro_rules! ext_ray_impl_nonmonic {
    () => {
        fn external_ray_helper(&self, angle: RationalAngle) -> Option<Vec<Cplx>>
        {
            use dynamo_common::math_utils::newton::{
                error::Error::NanEncountered, find_target_newton_err_d,
            };
            const R: Real = 256.0;
            let escape_radius_log = R.ln() * self.degree_real().abs();

            let deg_real = self.degree_real().abs();
            if deg_real.is_nan() {
                return None;
            }
            // let pixel_width = self.point_grid().pixel_width() * 0.08;
            let error = self.point_grid().res_x as Real * 1e-8;

            // let base_point = escape_radius * angle.to_circle();
            // Arbitrary starting guess that is likely to escape
            let base_point: Cplx = 65.0 * angle.to_circle();
            let mut t_list = vec![];

            // Target angle for the composite map at each step.
            // Initialized to value after self.escaping_phase() iterations.
            let target_angle_base = Real::from(self.angle_map_large_param(angle)) * TAU;
            // let mut target_angle = target_angle_base;

            let factor = (-deg_real.log2() / Real::from(RAY_SHARPNESS)).exp2();

            for k in 0..RAY_DEPTH {
                let num_iters = k * self.escaping_period() + self.escaping_phase();
                let fk_and_dfk = |t: Cplx| {
                    let (c, dc_dt) = self.param_map_d(t);
                    let (mut z, mut dz_dt, dz_dc) = self.start_point_d(t, &c);
                    dz_dt += dz_dc * dc_dt;

                    for _i in 0..num_iters {
                        let (f, df_dz, df_dc) = self.gradient(z, &c);
                        dz_dt = dz_dt * df_dz + df_dc * dc_dt;
                        z = f;
                    }

                    (z.into(), dz_dt.into())
                };

                let mut t_curr = *t_list.last().unwrap_or(&base_point);
                // let alpha = self.escape_coeff(self.param_map(t_curr)).arg();

                let mut u = Cplx::new(escape_radius_log, 0.);
                // let mut spiral = 0.0;

                for _j in 0..RAY_SHARPNESS {
                    let alpha = self.escape_coeff(&self.param_map(t_curr)).arg();

                    u.im = target_angle_base;
                    for _i in 0..k {
                        u.im *= deg_real;
                        u.im += alpha;
                        u.im %= TAU;
                    }

                    u.re *= factor;

                    let target = u.exp();
                    match find_target_newton_err_d(fk_and_dfk, t_curr, target, error) {
                        Ok((sol, _t_k, _d_k)) => {
                            if _j == 0 {
                                // println!("{}", (sol / t_curr).arg());
                            }
                            t_curr = sol;

                            if t_curr.is_nan() {
                                return Some(t_list);
                            }

                            t_list.push(t_curr);
                            //
                            // let dist = (2. * t_k.norm() * (t_k.norm()).log(deg_real)) / d_k.norm();
                            // if dist < pixel_width
                            // {
                            //     return Some(t_list);
                            // }
                        }
                        Err(NanEncountered) => {
                            // panic!("k = {k}, j = {}, t = {}", _j, t_curr);
                            return Some(t_list);
                        }
                        _ => {
                            // dbg!(_j, k, target, t_curr);
                        }
                    }
                }
            }

            Some(t_list)
        }
    };
}

pub(crate) use {
    cplx_arr, degree_impl, degree_impl_transcendental, ext_ray_impl_nonmonic, ext_ray_impl_rk,
    parameter_plane_impl, profile_imports,
};
