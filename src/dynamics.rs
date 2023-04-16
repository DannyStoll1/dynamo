use crate::{
    covering_maps::CoveringMap,
    iter_plane::IterPlane,
    orbit_info::OrbitInfo,
    palette::ColorPalette,
    point_grid::{Bounds, PointGrid},
    primitive_types::*,
};
use dyn_clone::DynClone;
use ndarray::Array2;
use rayon::iter::{ParallelBridge, ParallelIterator};

pub mod julia;
use julia::JuliaSet;

pub mod orbit;
use orbit::{CycleDetectedOrbit, Orbit};

pub trait ParameterPlane: Sync + Send + DynClone
{
    fn point_grid(&self) -> PointGrid;

    fn point_grid_mut(&mut self) -> &mut PointGrid;

    fn stop_condition(&self, iter: Period, z: ComplexNum) -> EscapeState
    {
        if iter > self.max_iter()
        {
            return EscapeState::Bounded;
        }

        let r = z.norm_sqr();
        if r > self.escape_radius() || z.is_nan()
        {
            EscapeState::Escaped {
                iters: iter,
                final_value: z,
            }
        }
        else
        {
            EscapeState::NotYetEscaped
        }
    }

    fn check_periodicity(
        &self,
        iter: Period,
        z_slow: ComplexNum,
        z_fast: ComplexNum,
        base_param: ComplexNum,
    ) -> EscapeState
    {
        if iter > self.max_iter()
        {
            return EscapeState::Bounded;
        }

        let r = z_fast.norm_sqr();
        if r > self.escape_radius() || z_fast.is_nan()
        {
            EscapeState::Escaped {
                iters: iter,
                final_value: z_fast,
            }
        }
        else if (z_fast - z_slow).norm_sqr() < self.periodicity_tolerance()
        {
            if let Some((period, multiplier)) = self.compute_period(
                z_fast,
                base_param,
                self.periodicity_tolerance() * 10.,
                iter as usize,
            )
            {
                EscapeState::Periodic {
                    preperiod: iter,
                    period,
                    multiplier,
                    final_error: z_slow - z_fast,
                }
            }
            else
            {
                EscapeState::NotYetEscaped
            }
        }
        else
        {
            EscapeState::NotYetEscaped
        }
    }

    fn max_iter(&self) -> Period;

    fn max_iter_mut(&mut self) -> &mut Period;

    fn set_max_iter(&mut self, new_max_iter: Period);

    #[inline]
    fn escape_radius(&self) -> RealNum
    {
        1e16
    }

    #[inline]
    fn periodicity_tolerance(&self) -> RealNum
    {
        self.point_grid().bounds.area() * 1e-12
    }

    fn compute_period(
        &self,
        z0: ComplexNum,
        c: ComplexNum,
        tolerance: RealNum,
        patience: usize,
    ) -> Option<(Period, ComplexNum)>
    {
        let mut z = z0;
        let mut dz = ONE_COMPLEX;
        for i in 1..=patience
        {
            z = self.map(z, c);
            dz *= self.dynamical_derivative(z, c);
            if (z - z0).norm_sqr() <= tolerance
            {
                if let Ok(period) = Period::try_from(i)
                {
                    return Some((period, dz));
                }
                return None;
            }
        }
        None
    }

    #[inline]
    fn start_point(&self, param: ComplexNum) -> ComplexNum
    {
        self.param_map(param)
    }

    #[inline]
    fn param_map(&self, point: ComplexNum) -> ComplexNum
    {
        point
    }

    fn map(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum;
    fn dynamical_derivative(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum;
    fn parameter_derivative(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum;
    fn gradient(&self, z: ComplexNum, c: ComplexNum) -> (ComplexNum, ComplexNum)
    {
        (
            self.dynamical_derivative(z, c),
            self.parameter_derivative(z, c),
        )
    }

    fn encode_escape_result(&self, state: EscapeState, base_param: ComplexNum) -> IterCount
    {
        match state
        {
            EscapeState::NotYetEscaped => 0.,
            EscapeState::Bounded => 0.,
            EscapeState::Periodic {
                period,
                preperiod,
                multiplier,
                final_error,
            } => self.encode_periodic_point(period, preperiod, multiplier, final_error),
            EscapeState::Escaped { iters, final_value } =>
            {
                self.encode_escaping_point(iters, final_value)
            }
        }
    }

    fn encode_periodic_point(
        &self,
        period: Period,
        preperiod: Period,
        multiplier: ComplexNum,
        final_error: ComplexNum,
    ) -> IterCount
    {
        let u = period as IterCount;
        // let mut w = -(final_error.norm_sqr() / self.periodicity_tolerance()).log(multiplier.norm())
        //     as IterCount;
        // if w.is_infinite() || w.is_nan()
        // {
        //     w = -0.2;
        // }
        // let v = preperiod as IterCount + u * w;
        // 0.02 is the internal coloring rate. Larger numbers mean faster darkening of the
        //   interiors of hyperbolic components.
        // -(u + 0.99 * (v * 0.02 / u).tanh())
        -(u + 0.99 * multiplier.norm())
    }

    fn encode_escaping_point(&self, iters: Period, z: ComplexNum) -> IterCount
    {
        if z.is_nan()
        {
            return (iters as IterCount) - 1.;
        }

        let u = self.escape_radius().log2();
        let v = z.norm_sqr().log2();
        let residual = (v / u).log2();
        (iters as IterCount) - (residual as IterCount)
    }

    fn run_until_escape(
        &self,
        start: ComplexNum,
        param: ComplexNum,
        escape_radius: RealNum,
        max_iter: Period,
    ) -> EscapeState
    {
        let stop_condition = |i: Period, z: ComplexNum| {
            if i > max_iter
            {
                EscapeState::Bounded
            }
            else if z.norm_sqr() > escape_radius || z.is_nan()
            {
                EscapeState::Escaped {
                    iters: i,
                    final_value: z,
                }
            }
            else
            {
                EscapeState::NotYetEscaped
            }
        };

        let orbit = Orbit::new(|z, c| self.map(z, c), stop_condition, start, param);
        if let Some((_, state)) = orbit.last()
        {
            state
        }
        else
        {
            EscapeState::NotYetEscaped
        }
    }

    fn run_point(&self, start: ComplexNum, param: ComplexNum) -> EscapeState
    {
        let orbit = CycleDetectedOrbit::new(
            |z, c| self.map(z, c),
            |i, z| self.stop_condition(i, z),
            |i, z0, z1, c| self.check_periodicity(i, z0, z1, c),
            start,
            param,
        );
        if let Some((_, state)) = orbit.last()
        {
            state
        }
        else
        {
            EscapeState::NotYetEscaped
        }
    }

    fn compute_escape_times(&self) -> Array2<IterCount>
    {
        let mut iter_counts = Array2::zeros((self.point_grid().res_x, self.point_grid().res_y));
        iter_counts
            .indexed_iter_mut()
            .par_bridge()
            .for_each(|((x, y), count)| {
                let point = self.point_grid().map_pixel(x, y);
                let param = self.param_map(point);
                let start = self.start_point(param);
                let orbit = CycleDetectedOrbit::new(
                    |z, c| self.map(z, c),
                    |i, z| self.stop_condition(i, z),
                    |i, z0, z1, c| self.check_periodicity(i, z0, z1, c),
                    start,
                    param,
                );

                if let Some((_, result)) = orbit.last()
                {
                    *count = self.encode_escape_result(result, point);
                }
            });
        iter_counts
    }

    fn compute(&self) -> IterPlane
    {
        let iter_counts = self.compute_escape_times();
        IterPlane {
            iter_counts,
            point_grid: self.point_grid(),
        }
    }

    #[inline]
    fn get_param(&self) -> ComplexNum
    {
        (0.).into()
    }

    #[inline]
    fn set_param(&mut self, _value: ComplexNum) {}

    fn external_angle(&self, point: ComplexNum) -> Option<RealNum>
    {
        let c = self.param_map(point);
        let z = self.start_point(c);
        if let EscapeState::Escaped { iters, final_value } =
            self.run_until_escape(z, c, 10., self.max_iter())
        {
            let error = 1e-12;
            let mut curr = c;
            let mut difference: ComplexNum;
            let mut target = final_value;
            let r = final_value.norm_sqr();
            while target.norm_sqr() <= r.powi(10)
            {
                target *= 1.01;
                dbg!(target);
                loop
                {
                    dbg!(curr);
                    // Newton's method to try to approximate
                    // outward points on external ray
                    let mut z_k = self.start_point(curr);
                    let mut d_k = ONE_COMPLEX;
                    for _ in 0..iters
                    {
                        d_k = d_k * self.dynamical_derivative(z_k, curr)
                            + self.parameter_derivative(z_k, curr);
                        z_k = self.map(z_k, curr);
                    }

                    if z_k.is_nan()
                    {
                        println!("nan encountered!");
                        return Some(curr.arg() / TAU);
                        break;
                    }

                    difference = (target - z_k) / d_k;
                    curr += difference;
                    dbg!(z_k, d_k, difference);

                    if difference.norm_sqr() < error
                    {
                        break;
                    }
                }
            }
            return Some(curr.arg() / TAU);
        }
        None
    }

    fn external_ray(
        &self,
        theta: RealNum,
        depth: u32,
        sharpness: u32,
        pixel_count: u32,
    ) -> Option<Vec<ComplexNum>>
    {
        let escape_radius = 20.;
        let pixel_width = self.point_grid().pixel_width();
        let error = f64::from(pixel_count * pixel_count) * 1e-8;

        let base_point = ComplexNum::new(escape_radius, theta).exp();
        let mut c_list = vec![base_point];

        for k in 0..depth
        {
            let us = (0..sharpness)
                .map(|j| escape_radius.ln() * TWO.powf((-f64::from(j)) / f64::from(sharpness)));
            let v = ComplexNum::new(0., theta * TWO.powi(k as i32));
            let targets = us.map(|u| (u + v).exp());

            let mut temp_c = *c_list.last()?;
            let mut dist = pixel_width * 2.;

            for target in targets
            {
                let mut old_c;
                let mut c_k;
                let mut d_k;

                let mut difference = f64::INFINITY;

                loop
                {
                    c_k = temp_c;
                    d_k = ComplexNum::new(1., 0.);
                    old_c = temp_c;
                    for _ in 0..k
                    {
                        d_k = d_k * self.dynamical_derivative(c_k, c_k)
                            + self.parameter_derivative(c_k, c_k);
                        c_k = self.map(c_k, old_c);
                    }
                    if c_k.is_nan()
                    {
                        break;
                    }
                    temp_c = old_c - (c_k - target) / d_k;
                    difference = (old_c - temp_c).norm_sqr();

                    if error > difference
                    {
                        break;
                    }
                }

                dist = (2. * c_k.norm() * (c_k.norm()).log2()) / d_k.norm();

                if dist < pixel_width
                {
                    return Some(c_list);
                }
            }
            c_list.push(temp_c);
        }

        Some(c_list)
    }

    // fn find_c_k_and_d_k(
    //     &self,
    //     temp_c: &mut ComplexNum,
    //     target: ComplexNum,
    //     iters: u32,
    //     error: f64,
    // ) -> (ComplexNum, ComplexNum) {
    //     let mut c_k;
    //     let mut d_k;
    //     let mut old_c;
    //     let mut difference = f64::INFINITY;
    //
    //     loop {
    //         c_k = *temp_c;
    //         d_k = ComplexNum::new(1., 0.);
    //         old_c = c_k;
    //
    //         for _ in 0..iters {
    //             d_k = self.dynamical_derivative(c_k, c_k) * d_k;
    //             c_k = self.map(c_k, old_c);
    //         }
    //
    //         if c_k.is_nan() {
    //             break;
    //         }
    //
    //         *temp_c = old_c - (c_k - target) / d_k;
    //         difference = (old_c - *temp_c).norm_sqr();
    //
    //         if error > difference {
    //             break;
    //         }
    //     }
    //
    //     (c_k, d_k)
    // }

    // fn external_ray(
    //     &self,
    //     theta: RealNum,
    //     // base_point: ComplexNum,
    //     depth: u32,
    //     sharpness: u32,
    //     pixel_count: u32,
    // ) -> Option<Vec<ComplexNum>> {
    //     let escape_radius = 50.;
    //     let pixel_width = self.point_grid().pixel_width();
    //     let mut m: u32;
    //     let mut r_m: RealNum;
    //     let mut t_m: ComplexNum;
    //     let mut temp_c: ComplexNum;
    //     let mut old_c: ComplexNum;
    //     let error = f64::from(pixel_count) * 0.0001;
    //     let mut difference: RealNum;
    //     let mut dist: RealNum;
    //     let mut c_k: ComplexNum;
    //     let mut d_k: ComplexNum;
    //
    //     let base_point = escape_radius * (TAUI * theta).exp();
    //     let mut c_list = vec![base_point];
    //
    //     for k in 0..depth {
    //         for j in 0..sharpness {
    //             let u = escape_radius.ln() * TWO.powf((1. - f64::from(j)) / f64::from(sharpness));
    //             let v = ComplexNum::new(0., theta * TWO.powi(k as i32));
    //             t_m = (u + v).exp();
    //             temp_c = *c_list.last()?;
    //
    //             // Repeat Newton's method until points are close together.
    //             loop {
    //                 old_c = temp_c;
    //                 // Recursive formula for iterates of q(z) = z^2 + c
    //                 c_k = old_c;
    //                 d_k = ComplexNum::new(1., 0.);
    //                 for _ in 0..k {
    //                     d_k = self.dynamical_derivative(c_k, c_k) * d_k;
    //                     c_k = self.map(c_k, old_c);
    //                 }
    //                 temp_c = old_c - (c_k - t_m) / d_k; // Newton map
    //                 difference = (old_c - temp_c).norm();
    //
    //                 if error > difference {
    //                     break;
    //                 }
    //             }
    //
    //             dist = (2. * c_k.norm() * (c_k.norm()).log2()) / d_k.norm();
    //             c_list.push(temp_c);
    //
    //             if dist < pixel_width {
    //                 break;
    //             }
    //         }
    //     }
    //     Some(c_list)
    // }

    fn get_orbit(&self, param: ComplexNum) -> Vec<ComplexNum>
    {
        let start = self.start_point(param);
        let orbit = Orbit::new(
            |z, c| self.map(z, c),
            |i, z| self.stop_condition(i, z),
            start,
            param,
        );
        orbit.map(|(z, s)| z).collect()
    }
    // fn orbit<F,S> (&self, param: ComplexNum) -> Orbit<F, S>
    // where
    //     F: Fn(ComplexNum, ComplexNum) -> ComplexNum + 'static,
    //     S: Fn(Period, ComplexNum) -> EscapeState + 'static,
    // {
    //     let start = self.start_point(param);
    //     Orbit::new(
    //         |z, c| self.map(z, c),
    //         |i, z| self.stop_condition(i, z),
    //         start,
    //         param,
    //     );
    // }
    //
    // fn cycle_detected_orbit(&self, param: ComplexNum) -> CycleDetectedOrbit<F, S, C>
    // where
    //     F: Fn(ComplexNum, ComplexNum) -> ComplexNum,
    //     S: Fn(Period, ComplexNum) -> EscapeState,
    //     C: Fn(Period, ComplexNum, ComplexNum, ComplexNum) -> EscapeState,
    // {
    //     let start = self.start_point(param);
    //     CycleDetectedOrbit::new(
    //         |z, c| self.map(z, c),
    //         |i, z| self.stop_condition(i, z),
    //         |i, z0, z1, c| self.check_periodicity(i, z0, z1, c),
    //         start,
    //         param,
    //     );
    // }

    fn get_orbit_info(&self, point: ComplexNum) -> OrbitInfo
    {
        let param = self.param_map(point);
        let start = self.start_point(param);
        let result = self.run_point(start, param);
        OrbitInfo {
            point,
            param,
            result,
        }
    }

    fn default_julia_bounds(&self) -> Bounds;

    fn julia_set(&self, point: ComplexNum) -> Option<JuliaSet<Self>>
    where
        Self: Clone,
    {
        let param = self.param_map(point);
        let point_grid = self
            .point_grid()
            .with_same_height(self.default_julia_bounds());
        Some(JuliaSet {
            point_grid,
            max_iter: self.max_iter(),
            parent: self.clone(),
            param,
            // parent_params: Vec::new(),
        })
    }

    fn name(&self) -> String;
}

pub trait HasDynamicalCovers: ParameterPlane + Clone
{
    fn marked_cycle_curve(self, _period: Period) -> CoveringMap<Self>
    {
        let param_map = |c| c;
        let bounds = self.point_grid();

        println!("Marked cycle has not been implemented; falling back to base curve!");
        CoveringMap::new(self, param_map, bounds)
    }
    fn dynatomic_curve(self, _period: Period) -> CoveringMap<Self>
    {
        let param_map = |c| c;
        let bounds = self.point_grid();

        println!("Dynatomic curve has not been implemented; falling back to base curve!");
        CoveringMap::new(self, param_map, bounds)
    }
    fn misiurewicz_curve(self, _preperiod: Period, _period: Period) -> CoveringMap<Self>
    {
        let param_map = |c| c;
        let bounds = self.point_grid();

        println!("Misiurewicz curve has not been implemented; falling back to base curve!");
        CoveringMap::new(self, param_map, bounds)
    }
}
