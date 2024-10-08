use super::{ComputeMode, DynamicalFamily, FamilyDefaults, HasJulia, MarkedPoints};
use crate::macros::basic_plane_impl;
use crate::orbit::EscapeResult;
use dynamo_color::{Coloring, IncoloringAlgorithm};
use dynamo_common::math_utils::newton::error::Error::NanEncountered;
use dynamo_common::math_utils::newton::find_target_newton_err_d;
use dynamo_common::prelude::*;
use dynamo_common::symbolic_dynamics::OrbitSchema;
use num_traits::{One, Zero};

use super::{EscapeEncoding, ExternalRays, InfinityFirstReturnMap, PlaneType};

#[derive(Clone)]
pub struct JuliaSet<T>
where
    T: DynamicalFamily,
{
    pub point_grid: PointGrid,
    pub max_iter: IterCount,
    pub min_iter: IterCount,
    pub parent: T,
    pub meta_params: T::MetaParam,
    pub local_param: T::Param,
    pub parent_selection: Cplx,
    pub compute_mode: ComputeMode,
}

impl<T> JuliaSet<T>
where
    T: DynamicalFamily + HasJulia,
{
    #[must_use]
    pub fn new(parent: T, parent_selection: Cplx, max_iter: IterCount) -> Self
    {
        let local_param = parent.param_map(parent_selection);
        let point_grid = parent
            .point_grid()
            .new_with_same_height(parent.default_bounds_child(parent_selection, &local_param));
        let min_iter = parent.min_iter();
        let meta_params = parent.get_meta_params();
        Self {
            point_grid,
            max_iter,
            min_iter,
            parent,
            meta_params,
            local_param,
            parent_selection,
            compute_mode: ComputeMode::SmoothPotential,
        }
    }

    #[must_use]
    pub fn with_param(mut self, c: T::Param) -> Self
    {
        self.set_param(c);
        self
    }

    pub fn map_and_multiplier_lazy(&self, z: T::Var) -> (T::Var, T::Deriv)
    {
        self.parent.map_and_multiplier(z, &self.local_param)
    }
}

impl<T> From<T> for JuliaSet<T>
where
    T: FamilyDefaults + HasJulia,
{
    fn from(parent: T) -> Self
    {
        let selection = parent.default_selection();
        let max_iter = parent.default_max_iter_child();
        Self::new(parent, selection, max_iter)
    }
}

impl<T> DynamicalFamily for JuliaSet<T>
where
    T: HasJulia,
{
    type Var = T::Var;
    type Param = NoParam;
    type MetaParam = ParamStack<T::MetaParam, T::Param>;
    type Deriv = T::Deriv;
    basic_plane_impl!();

    #[inline]
    fn map(&self, z: Self::Var, _c: &Self::Param) -> Self::Var
    {
        self.parent.map(z, &self.local_param)
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, _c: &Self::Param) -> (Self::Var, Self::Deriv)
    {
        self.parent.map_and_multiplier(z, &self.local_param)
    }

    #[inline]
    fn gradient(&self, z: Self::Var, _c: &Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        let (f, df_dz) = self.map_and_multiplier(z, &NoParam);
        (f, df_dz, Self::Deriv::zero())
    }

    #[inline]
    fn escape_radius(&self) -> Real
    {
        self.parent.escape_radius()
    }

    #[inline]
    fn min_iter(&self) -> IterCount
    {
        self.min_iter
    }

    #[inline]
    fn extra_stop_condition(
        &self,
        z: Self::Var,
        _: &NoParam,
        iter: IterCount,
    ) -> Option<EscapeResult<Self::Var, Self::Deriv>>
    {
        self.parent.extra_stop_condition(z, &self.local_param, iter)
    }

    #[inline]
    fn param_map(&self, _z: Cplx) -> Self::Param
    {
        NoParam
    }

    #[inline]
    fn param_map_d(&self, _z: Cplx) -> (Self::Param, Self::Deriv)
    {
        (NoParam, Self::Deriv::zero())
    }

    #[inline]
    fn start_point(&self, point: Cplx, _param: &Self::Param) -> Self::Var
    {
        self.parent.dynam_map(point)
    }

    #[inline]
    fn start_point_d(
        &self,
        point: Cplx,
        _param: &Self::Param,
    ) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        let (z, dz_dt) = self.parent.dynam_map_d(point);
        (z, dz_dt, Self::Deriv::zero())
    }

    #[inline]
    fn cycle_active_plane(&mut self)
    {
        self.parent.cycle_active_plane();
    }

    #[inline]
    fn set_meta_param(
        &mut self,
        ParamStack {
            meta_params,
            local_param,
        }: Self::MetaParam,
    )
    {
        self.meta_params = meta_params;
        self.local_param = local_param;
    }

    #[inline]
    fn set_param(&mut self, local_param: T::Param)
    {
        self.local_param = local_param;
    }

    #[inline]
    fn get_meta_params(&self) -> Self::MetaParam
    {
        ParamStack {
            local_param: self.local_param.clone(),
            meta_params: self.meta_params.clone(),
        }
    }

    #[inline]
    fn get_param(&self) -> T::Param
    {
        self.local_param.clone()
    }

    // #[inline]
    // fn julia_set(&self, _param: ComplexNum) -> Option<JuliaSet<Self>>
    // {
    //     None
    // }

    #[inline]
    fn name(&self) -> String
    {
        let parent_name = self.parent.name();

        format!("Julia({parent_name})")
    }

    fn description(&self) -> String
    {
        self.parent.description()
    }

    #[inline]
    fn periodicity_tolerance(&self) -> Real
    {
        self.parent.periodicity_tolerance()
    }

    fn internal_potential_coloring(&self) -> IncoloringAlgorithm
    {
        self.parent.internal_potential_coloring()
    }

    fn potential_and_period_coloring(&self) -> IncoloringAlgorithm
    {
        self.parent.potential_and_period_coloring()
    }

    fn preperiod_coloring(&self) -> IncoloringAlgorithm
    {
        self.parent.preperiod_coloring()
    }

    #[inline]
    fn plane_type(&self) -> PlaneType
    {
        PlaneType::Dynamical
    }

    // /// Compute an external ray for a given angle in [0,1).
    // /// depth: Controls how deep the ray goes. Higher values bring the landing point closer to the
    // /// bifurcation locus. [Suggested starting value: 25]
    // /// sharpness: Controls the density of points used to approxmate the external ray. [Suggested starting value: 20]
    // fn external_ray(&self, theta: Real) -> Option<Vec<Cplx>>
    // {
    //     let escape_radius = 400.;
    //     let deg = self.degree().powi(self.escaping_period() as i32);
    //     if deg.is_nan()
    //     {
    //         return None;
    //     }
    //     let deg_log2 = deg.log2();
    //
    //     let pixel_width = self.point_grid().pixel_width() * 0.3;
    //     let error = self.point_grid().res_x as Real * 1e-8;
    //
    //     let angle = theta * TAU;
    //     let base_point = escape_radius * Cplx::new(0., angle).exp();
    //     let mut z_list = vec![base_point];
    //
    //     // degree raised to the power k
    //     let mut deg_k = 1.0;
    //
    //     for k in 0..RAY_DEPTH
    //     {
    //         let us = (0..RAY_SHARPNESS).map(|j| {
    //             escape_radius.ln()
    //                 * ((-Real::from(j) * deg_log2) / Real::from(RAY_SHARPNESS)).exp2()
    //         });
    //         let v = Cplx::new(0., angle * deg_k);
    //         deg_k *= deg;
    //         let targets = us.map(|u| (u + v).exp());
    //
    //         let mut temp_z = *z_list.last()?;
    //         let mut dist: Real;
    //
    //         let fk_and_dfk = |z: Cplx| {
    //             let mut z_k = z.into();
    //             let mut d_k = ONE;
    //             for _ in 0..k * self.escaping_period()
    //             {
    //                 let (f, df_dz) = self.map_and_multiplier_lazy(z_k);
    //                 d_k *= df_dz.into();
    //                 z_k = f;
    //             }
    //             (z_k.into(), d_k)
    //         };
    //
    //         for target in targets
    //         {
    //             let Some((sol, z_k, d_k)) =
    //                 find_target_newton_err_d(fk_and_dfk, temp_z, target, error)
    //             else
    //             {
    //                 return Some(z_list);
    //             };
    //
    //             temp_z = sol;
    //
    //             dist = (2. * z_k.norm() * (z_k.norm()).log(deg)) / d_k.norm();
    //
    //             if temp_z.is_nan()
    //             {
    //                 return Some(z_list);
    //             }
    //
    //             z_list.push(temp_z);
    //             if dist < pixel_width
    //             {
    //                 return Some(z_list);
    //             }
    //         }
    //     }
    //
    //     Some(z_list)
    // }

    // Try to find a (pre)periodic point near a given base point
    // fn find_nearby_preperiodic_point(
    //     &self,
    //     start_point: Cplx,
    //     OrbitSchema {
    //         period: n,
    //         preperiod: k,
    //     }: OrbitSchema,
    // ) -> Option<Cplx>
    // {
    //     if n == 0
    //     {
    //         return None;
    //     }
    //
    //     // Number of unitary divisors of n
    //     let num_factors = divisors(n).filter(|d| gcd(n / d, *d) == 1).count();
    //
    //     // Values and derivatives of (f^{m+k}(z0) - f^k(z0))^(mu(n/m)) for m a proper unitary divisor of n
    //     let mut values = vec![ZERO; num_factors - 1];
    //     let mut derivs = vec![ONE; num_factors - 1];
    //
    //     let diff = |t| {
    //         let c = NoParam;
    //         let (mut z, mut dz_dt) = self.start_point_d(t, c);
    //         let mut df_dz: Self::Deriv;
    //
    //         let mut term_count: usize = 0;
    //
    //         for _ in 0..k
    //         {
    //             (z, df_dz) = self.map_and_multiplier(z, c);
    //             dz_dt *= df_dz;
    //         }
    //
    //         let mut w = z.clone();
    //         let mut dw_dt = dz_dt.clone();
    //
    //         // Do first iteration manually to avoid division by zero
    //         (w, df_dz) = self.map_and_multiplier(w, c);
    //         dw_dt = dw_dt * df_dz;
    //
    //         for i in 1..n
    //         {
    //             let (q, r) = n.div_rem(&i);
    //             if r == 0
    //             {
    //                 let mu = moebius(q);
    //                 if mu == 1
    //                 {
    //                     values[term_count] = (w - z).into();
    //                     derivs[term_count] = (dw_dt - dz_dt).into();
    //                     term_count += 1;
    //                 }
    //                 else if mu == -1
    //                 {
    //                     let dg = (dz_dt - dw_dt).into();
    //                     let val = (w - z).into().inv();
    //                     values[term_count] = val;
    //                     derivs[term_count] = dg * val * val;
    //                     term_count += 1;
    //                 }
    //             }
    //
    //             (w, df_dz) = self.map_and_multiplier(w, c);
    //             dw_dt *= df_dz;
    //         }
    //         let val_n = (w - z).into();
    //         let dif_n = (dw_dt - dz_dt).into();
    //
    //         // Iteratively apply product rule to compute derivative
    //         values
    //             .iter()
    //             .zip(derivs.iter())
    //             .fold((val_n, dif_n), |(u, du), (v, dv)| (u * v, u * dv + v * du))
    //     };
    //
    //     find_root_newton(diff, start_point)
    // }
}

impl<T> FamilyDefaults for JuliaSet<T>
where
    T: HasJulia,
{
    #[inline]
    fn default_selection(&self) -> Cplx
    {
        self.parent.start_point(ZERO, &self.local_param).into()
    }

    fn default_coloring(&self) -> Coloring
    {
        self.parent.default_coloring_child()
    }

    #[inline]
    fn default_bounds(&self) -> Bounds
    {
        self.parent
            .default_bounds_child(self.parent_selection, &self.local_param)
    }
}

impl<T> HasJulia for JuliaSet<T>
where
    T: HasJulia,
{
    #[inline]
    fn default_bounds_child(&self, _point: Cplx, _param: &Self::Param) -> Bounds
    {
        self.point_grid.bounds.clone()
    }
}

impl<P> MarkedPoints for JuliaSet<P>
where
    P: HasJulia + MarkedPoints,
{
    #[inline]
    fn critical_points_child(&self, _param: &Self::Param) -> Vec<Self::Var>
    {
        self.parent.critical_points_child(&self.local_param)
    }

    #[inline]
    fn critical_points(&self) -> Vec<Self::Var>
    {
        self.parent.critical_points_child(&self.local_param)
    }

    #[inline]
    fn cycles_child(&self, _param: &Self::Param, period: Period) -> Vec<Self::Var>
    {
        self.parent.cycles_child(&self.local_param, period)
    }

    #[inline]
    fn cycles(&self, period: Period) -> Vec<Self::Var>
    {
        self.parent.cycles_child(&self.local_param, period)
    }

    #[inline]
    fn precycles(&self, orbit_schema: OrbitSchema) -> Vec<Self::Var>
    {
        self.parent.precycles_child(&self.local_param, orbit_schema)
    }
}

impl<P> InfinityFirstReturnMap for JuliaSet<P>
where
    P: HasJulia + InfinityFirstReturnMap,
{
    #[inline]
    fn degree_real(&self) -> f64
    {
        self.parent.degree_real()
    }

    #[inline]
    fn degree(&self) -> AngleNum
    {
        self.parent.degree()
    }

    #[inline]
    fn escaping_period(&self) -> Period
    {
        self.parent.escaping_period()
    }

    /// Always 0 for dynamical planes, since large parameter here means large starting value
    #[inline]
    fn escaping_phase(&self) -> Period
    {
        0
    }

    #[inline]
    fn escape_coeff_d(&self, _c: &Self::Param) -> (Cplx, Cplx)
    {
        (self.parent.escape_coeff_d(&self.local_param).0, ZERO)
    }
}

impl<P: EscapeEncoding + HasJulia> EscapeEncoding for JuliaSet<P>
{
    fn encode_escape_result(
        &self,
        result: EscapeResult<Self::Var, Self::Deriv>,
        start: Self::Var,
        NoParam: &Self::Param,
    ) -> PointInfo<Self::Deriv>
    {
        self.parent
            .encode_escape_result(result, start, &self.local_param)
    }
}

impl<P> ExternalRays for JuliaSet<P>
where
    P: HasJulia + InfinityFirstReturnMap,
{
    #[allow(clippy::similar_names)]
    #[allow(clippy::many_single_char_names)]
    fn external_ray_helper(&self, angle: RationalAngle) -> Option<Vec<Cplx>>
    {
        const R: Real = 16.0;
        let escape_radius_log2 = R.log2() * self.degree_real().abs();

        let deg_real = self.degree_real().abs();
        if deg_real.is_nan() || self.degree() <= 1 {
            return None;
        }

        let pixel_width = self.point_grid().pixel_width() * 0.03;
        let error = self.point_grid().res_x as Real * 1e-8;

        // let base_point = escape_radius * angle.to_circle();
        // Arbitrary starting guess that is likely to escape
        let base_point: Cplx = 65.0 * angle.to_circle();
        let mut t_list = vec![];

        let deg_real = self.degree_real();
        let a = self.escape_coeff(&NoParam);
        let angle_boost = a.arg() / TAU;

        // Target angle for the composite map at each step.
        // Initialized to value after self.escaping_phase() iterations.
        let mut target_angle = Real::from(self.angle_map_large_param(angle));

        let factor = (-deg_real.log2() / Real::from(RAY_SHARPNESS)).exp2();
        let du = a.norm().log2() / Real::from(RAY_SHARPNESS);

        for k in 0..RAY_DEPTH {
            let v = target_angle.to_circle();
            let mut u = escape_radius_log2;

            // Relative log2-norms of targets
            // jth target norm = escape_radius^deg^(-j/S)
            // u_j = log2(escape_radius) * deg^(-j/S)
            let us = (0..RAY_SHARPNESS).map(|_| {
                let u_i = u;
                u *= factor;
                u -= du;
                u_i
            });

            let targets = us.map(|u| u.exp2() * v);

            let mut t_curr = *t_list.last().unwrap_or(&base_point);
            let mut dist: Real;

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

            for target in targets {
                match find_target_newton_err_d(fk_and_dfk, t_curr, target, error) {
                    Ok((sol, t_k, d_k)) => {
                        // dbg!(target, sol);
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
            // target_angle *= deg;
            target_angle = target_angle.mul_add(deg_real, angle_boost) % 1.;
        }

        Some(t_list)
    }
}
