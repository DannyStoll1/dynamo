use crate::{
    coloring::{coloring_algorithm::ColoringAlgorithm, Coloring},
    iter_plane::IterPlane,
    point_grid::{Bounds, PointGrid},
    types::{ComplexNum, ComplexVec, EscapeState, IterCount, ONE_COMPLEX, OrbitInfo, Period, PointInfo, RealNum, TAU, TWO},
};
use dyn_clone::DynClone;
use ndarray::{Array2, Axis};
use num_cpus;
use rayon::iter::{ParallelBridge, ParallelIterator};
use std::cell::RefCell;
use thread_local::ThreadLocal;

pub mod covering_maps;
pub mod julia;
pub mod orbit;


use julia::JuliaSet;
use orbit::{CycleDetectedOrbit, Orbit};

pub trait ParameterPlane: Sync + Send + DynClone
{
    fn point_grid(&self) -> &PointGrid;

    fn point_grid_mut(&mut self) -> &mut PointGrid;

    fn early_bailout(&self, _start: ComplexNum, _param: ComplexNum) -> EscapeState
    {
        EscapeState::NotYetEscaped
    }

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
                self.periodicity_tolerance().powf(0.75),
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
        1e12
    }

    #[inline]
    fn periodicity_tolerance(&self) -> RealNum
    {
        // self.point_grid().bounds.area() * 1e-8
        1e-14
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

    fn encode_escape_result(&self, state: EscapeState, base_param: ComplexNum) -> PointInfo
    {
        match state
        {
            EscapeState::NotYetEscaped | EscapeState::Bounded => PointInfo::Bounded,
            EscapeState::Periodic {
                period,
                preperiod,
                multiplier,
                final_error,
            } => PointInfo::Periodic {
                period,
                preperiod,
                multiplier,
                final_error,
            },
            EscapeState::Escaped { iters, final_value } =>
            {
                self.encode_escaping_point(iters, final_value, base_param)
            }
        }
    }

    fn encode_escaping_point(
        &self,
        iters: Period,
        z: ComplexNum,
        _base_param: ComplexNum,
    ) -> PointInfo
    {
        if z.is_nan()
        {
            return PointInfo::Escaping {
                potential: f64::from(iters) - 1.,
            };
        }

        let u = self.escape_radius().log2();
        let v = z.norm_sqr().log2();
        let residual = (v / u).log2();
        PointInfo::Escaping {
            potential: f64::from(iters) - (residual as IterCount),
        }
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

    fn run_point(&self, start: ComplexNum, param: ComplexNum) -> PointInfo
    {
        let orbit = CycleDetectedOrbit::new(
            |z, c| self.map(z, c),
            |z, c| self.dynamical_derivative(z, c),
            |i, z| self.stop_condition(i, z),
            |i, z0, z1, c| self.check_periodicity(i, z0, z1, c),
            |z, c| self.early_bailout(z, c),
            start,
            param,
        );
        if let Some((_, state)) = orbit.last()
        {
            self.encode_escape_result(state, param)
        }
        else
        {
            PointInfo::Bounded
        }
    }

    fn compute_escape_times(&self) -> Array2<PointInfo>
    {
        let orbits = ThreadLocal::new();

        let chunk_size = self.point_grid().res_y / num_cpus::get(); // or another value that gives optimal performance

        let mut iter_counts = Array2::from_elem(
            (self.point_grid().res_x, self.point_grid().res_y),
            PointInfo::Bounded,
        );
        iter_counts
            .axis_chunks_iter_mut(Axis(1), chunk_size)
            .enumerate()
            .par_bridge()
            .for_each(|(chunk_idx, mut chunk)| {
                for ((x, local_y), count) in chunk.indexed_iter_mut()
                {
                    let y = chunk_idx * chunk_size + local_y;
                    let point = self.point_grid().map_pixel(x, y);
                    let param = self.param_map(point);
                    let start = self.start_point(param);

                    let mut orbit = orbits
                        .get_or(|| {
                            RefCell::new(CycleDetectedOrbit::new(
                                |z, c| self.map(z, c),
                                |z, c| self.dynamical_derivative(z, c),
                                |i, z| self.stop_condition(i, z),
                                |i, z0, z1, c| self.check_periodicity(i, z0, z1, c),
                                |z, c| self.early_bailout(z, c),
                                param,
                                start,
                            ))
                        })
                        .borrow_mut();

                    orbit.reset(param, start);
                    let result = orbit.run_until_complete();
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
            point_grid: self.point_grid().clone(),
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
                        // break;
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
            // let mut dist = pixel_width * 2.;
            let mut dist: f64;

            for target in targets
            {
                let mut old_c;
                let mut c_k;
                let mut d_k;

                let mut difference: f64;

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
        orbit.map(|(z, _s)| z).collect()
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

    fn get_orbit_and_info(&self, param: ComplexNum) -> (ComplexVec, OrbitInfo)
    {
        let start = self.start_point(param);
        let orbit = CycleDetectedOrbit::new(
            |z, c| self.map(z, c),
            |z, c| self.dynamical_derivative(z, c),
            |i, z| self.stop_condition(i, z),
            |i, z0, z1, c| self.check_periodicity(i, z0, z1, c),
            |z, c| self.early_bailout(z, c),
            start,
            param,
        );
        let mut final_state = EscapeState::Bounded;
        let trajectory: ComplexVec = orbit
            .map(|(z, s)| {
                final_state = s;
                z
            })
            .collect();
        let result = self.encode_escape_result(final_state, param);
        (
            trajectory,
            OrbitInfo {
                point: start,
                param,
                result,
            },
        )
    }

    fn default_julia_bounds(&self, _param: ComplexNum) -> Bounds
    {
        Bounds::centered_square(2.2)
    }

    fn default_julia_param(&self) -> ComplexNum
    {
        ComplexNum::new(0., 0.)
    }

    fn julia_set(&self, point: ComplexNum) -> Option<JuliaSet<Self>>
    where
        Self: Clone,
    {
        let param = self.param_map(point);
        let point_grid = self
            .point_grid()
            .with_same_height(self.default_julia_bounds(param));

        Some(JuliaSet {
            point_grid,
            max_iter: self.max_iter(),
            parent: self.clone(),
            param,
        })
    }

    fn default_coloring(&self) -> Coloring
    {
        let mut coloring = Coloring::default();
        coloring.set_algorithm(ColoringAlgorithm::PeriodMultiplier);
        coloring
    }

    fn preperiod_smooth_coloring(&self) -> ColoringAlgorithm
    {
        ColoringAlgorithm::PreperiodSmooth {
            periodicity_tolerance: self.periodicity_tolerance(),
            fill_rate: 0.04,
        }
    }

    fn name(&self) -> String;
}
