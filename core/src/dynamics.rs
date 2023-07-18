use fractal_common::{
    coloring::{algorithms::ColoringAlgorithm, Coloring},
    consts::ONE,
    iter_plane::IterPlane,
    math_utils::{newton_until_convergence, newton_until_convergence_d},
    point_grid::{self, Bounds, PointGrid},
    types::param_stack::Summarize,
    types::{
        Cplx, Dist, EscapeState, IterCount, Norm, OrbitInfo, ParamList, Period, PointInfo, Real,
    },
};
use ndarray::{Array2, Axis};
use num_cpus;
use rayon::iter::{ParallelBridge, ParallelIterator};
use std::fmt::Display;
use std::{cell::RefCell, f64::consts::TAU};
use thread_local::ThreadLocal;

pub mod covering_maps;
pub mod julia;
pub mod newton;
pub mod orbit;
// pub mod simple_parameter_plane;
// pub mod functions;

use julia::JuliaSet;
use orbit::{CycleDetectedOrbitFloyd, SimpleOrbit};
use std::ops::{MulAssign, Sub};

use self::orbit::OrbitParams;
// pub use simple_parameter_plane::SimpleParameterPlane;

pub trait ParameterPlane: Sync + Send + Clone
{
    type Var: Norm<Real> + Dist<Real> + Send + Default + From<Cplx> + Into<Cplx> + Display;
    type Param: From<Cplx> + Clone + Copy + Send + Sync + Default + PartialEq + Summarize;
    type MetaParam: ParamList + Clone + Copy + Send + Sync + Default + Summarize;
    type Deriv: Norm<Real> + Send + Default + From<f64> + MulAssign + Display + Into<Cplx>;
    type Child: ParameterPlane + From<Self>;

    fn point_grid(&self) -> &PointGrid;
    fn point_grid_mut(&mut self) -> &mut PointGrid;

    #[must_use]
    fn with_point_grid(self, point_grid: PointGrid) -> Self;

    #[must_use]
    fn with_bounds(self, bounds: Bounds) -> Self
    {
        let point_grid = self.point_grid().new_with_same_height(bounds);
        self.with_point_grid(point_grid)
    }

    #[must_use]
    fn with_res_y(mut self, res_y: usize) -> Self
    {
        self.point_grid_mut().resize_y(res_y);
        self
    }

    fn max_iter(&self) -> Period;
    fn max_iter_mut(&mut self) -> &mut Period;
    fn set_max_iter(&mut self, new_max_iter: Period);

    #[must_use]
    fn with_max_iter(self, max_iter: Period) -> Self;

    fn name(&self) -> String;
    fn map(&self, z: Self::Var, c: Self::Param) -> Self::Var;
    fn dynamical_derivative(&self, z: Self::Var, c: Self::Param) -> Self::Deriv;
    fn parameter_derivative(&self, z: Self::Var, c: Self::Param) -> Self::Deriv;

    fn early_bailout(
        &self,
        _start: Self::Var,
        _c: Self::Param,
    ) -> EscapeState<Self::Var, Self::Deriv>
    {
        EscapeState::NotYetEscaped
    }

    // Minimum iterations before cycle detection is allowed
    fn min_iter(&self) -> Period
    {
        0
    }

    #[inline]
    fn escape_radius(&self) -> Real
    {
        1e12
    }

    #[inline]
    fn periodicity_tolerance(&self) -> Real
    {
        // self.point_grid().bounds.area() * 1e-8
        1e-14
    }

    fn start_point(&self, _point: Cplx, c: Self::Param) -> Self::Var;
    // fn start_point(&self, _point: Cplx, c: Self::Param) -> Self::Var
    // {
    //     c.into()
    // }

    fn param_map(&self, point: Cplx) -> Self::Param
    {
        point.into()
    }

    fn dynam_map(&self, point: Cplx) -> Self::Var
    {
        point.into()
    }

    fn map_and_multiplier(&self, z: Self::Var, c: Self::Param) -> (Self::Var, Self::Deriv)
    {
        (self.map(z, c), self.dynamical_derivative(z, c))
    }

    fn gradient(&self, z: Self::Var, c: Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        let (fz, df_dz) = self.map_and_multiplier(z, c);
        (fz, df_dz, self.parameter_derivative(z, c))
    }

    fn encode_escape_result(
        &self,
        state: EscapeState<Self::Var, Self::Deriv>,
        c: Self::Param,
    ) -> PointInfo<Self::Deriv>
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
                self.encode_escaping_point(iters, final_value, c)
            }
        }
    }

    fn encode_escaping_point(
        &self,
        iters: Period,
        z: Self::Var,
        _c: Self::Param,
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
        let residual = (v / u).log2();
        PointInfo::Escaping {
            potential: f64::from(iters) - (residual as IterCount),
        }
    }

    fn compute(&self) -> IterPlane<Self::Deriv>
    {
        let mut iter_plane = IterPlane::create(self.point_grid().clone());
        self.compute_into(&mut iter_plane);
        iter_plane
    }

    fn compute_into(&self, iter_plane: &mut IterPlane<Self::Deriv>)
    {
        if self.point_grid().is_nan()
        {
            return;
        }

        let orbits = ThreadLocal::new();

        let chunk_size = self.point_grid().res_y / num_cpus::get(); // or another value that gives optimal performance

        iter_plane.iter_counts
            .axis_chunks_iter_mut(Axis(1), chunk_size)
            .enumerate()
            .par_bridge()
            .for_each(|(chunk_idx, mut chunk)| {
                let orbit_params = OrbitParams {
                    max_iter: self.max_iter(),
                    min_iter: self.min_iter(),
                    periodicity_tolerance: self.periodicity_tolerance(),
                    escape_radius: self.escape_radius(),
                };

                chunk.indexed_iter_mut().for_each(|((x, local_y), count)| {
                    let y = chunk_idx * chunk_size + local_y;
                    let point = self.point_grid().map_pixel(x, y);
                    let param = self.param_map(point);
                    let start = self.start_point(point, param);
                    let mut orbit = orbits
                        .get_or(|| {
                            RefCell::new(CycleDetectedOrbitFloyd::new(
                                |c, z| self.map(c, z),
                                |c, z| self.map_and_multiplier(c, z),
                                |c, z| self.early_bailout(c, z),
                                start,
                                param,
                                &orbit_params,
                            ))
                        })
                        .borrow_mut();

                    orbit.reset(param, start);
                    let result = orbit.run_until_complete();
                    *count = self.encode_escape_result(result, param);
                });
            });
    }

    #[inline]
    fn get_meta_params(&self) -> Self::MetaParam
    {
        Self::MetaParam::default()
    }

    #[inline]
    fn get_param(&self) -> <Self::MetaParam as ParamList>::Param
    {
        <Self::MetaParam as ParamList>::Param::default()
    }

    #[inline]
    fn set_meta_param(&mut self, _value: Self::MetaParam) {}

    #[inline]
    fn set_param(&mut self, _value: <Self::MetaParam as ParamList>::Param) {}

    #[inline]
    #[must_use]
    fn with_param(mut self, param: <Self::MetaParam as ParamList>::Param) -> Self
    {
        self.set_param(param);
        self
    }

    #[inline]
    fn critical_points_child(&self, _c: Self::Param) -> Vec<Self::Var>
    {
        vec![]
    }

    #[inline]
    fn critical_points(&self) -> Vec<Self::Var>
    {
        vec![]
    }

    fn cycles_child(&self, _c: Self::Param, _period: Period) -> Vec<Self::Var>
    {
        vec![]
    }

    fn cycles(&self, _period: Period) -> Vec<Self::Var>
    {
        vec![]
    }

    fn external_ray(
        &self,
        theta: Real,
        depth: u32,
        sharpness: u32,
        pixel_count: u32,
    ) -> Option<Vec<Cplx>>
    {
        let escape_radius = 40.;
        let deg = self.degree();
        if deg.is_nan()
        {
            return None;
        }

        let pixel_width = self.point_grid().pixel_width();
        let error = f64::from(pixel_count * pixel_count) * 1e-12;

        let angle = theta * TAU;
        let base_point = escape_radius * Cplx::new(0., angle).exp();
        let mut c_list = vec![base_point];

        for k in 0..depth
        {
            let us = (0..sharpness).map(|j| {
                escape_radius.ln() * ((-f64::from(j) * deg.log2()) / f64::from(sharpness)).exp2()
            });
            let v = Cplx::new(0., angle * deg.powi(k as i32));
            let targets = us.map(|u| (u + v).exp());

            let mut temp_c = *c_list.last()?;
            let mut dist: f64;

            let fk_and_dfk = |mut c_k: Cplx| {
                let mut d_k = ONE;
                let old_c = c_k;
                for _ in 0..k
                {
                    let (f, df_dz, df_dc) = self.gradient(c_k.into(), old_c.into());
                    d_k = d_k * df_dz.into() + df_dc.into();
                    c_k = f.into();
                }
                (c_k, d_k)
            };

            for target in targets
            {
                let (sol, c_k, d_k) = newton_until_convergence_d(fk_and_dfk, temp_c, target, error);

                temp_c = sol;

                dist = (2. * c_k.norm() * (c_k.norm()).log(deg)) / d_k.norm();

                if dist < pixel_width
                {
                    return Some(c_list);
                }
            }
            c_list.push(temp_c);
        }

        Some(c_list)
    }

    // fn external_angle(&self, point: Cplx) -> Option<Real>
    // {
    //     let c = self.param_map(point);
    //     let z = self.start_point(c);
    //     if let EscapeState::Escaped { iters, final_value } =
    //         self.run_until_escape(z, c, 10., self.max_iter())
    //     {
    //         let error = 1e-12;
    //         let mut curr = c;
    //         let mut difference: Cplx;
    //         let mut target = final_value;
    //         let r = final_value.norm_sqr();
    //         while target.norm_sqr() <= r.powi(10)
    //         {
    //             target *= 1.01;
    //             dbg!(target);
    //             loop
    //             {
    //                 dbg!(curr);
    //                 // Newton's method to try to approximate
    //                 // outward points on external ray
    //                 let mut z_k = self.start_point(curr);
    //                 let mut d_k = ONE_COMPLEX;
    //                 for _ in 0..iters
    //                 {
    //                     d_k = d_k * self.dynamical_derivative(z_k, curr)
    //                         + self.parameter_derivative(z_k, curr);
    //                     z_k = self.map(z_k, curr);
    //                 }
    //
    //                 if z_k.is_nan()
    //                 {
    //                     println!("nan encountered!");
    //                     return Some(curr.arg() / TAU);
    //                     // break;
    //                 }
    //
    //                 difference = (target - z_k) / d_k;
    //                 curr += difference;
    //                 dbg!(z_k, d_k, difference);
    //
    //                 if difference.norm_sqr() < error
    //                 {
    //                     break;
    //                 }
    //             }
    //         }
    //         return Some(curr.arg() / TAU);
    //     }
    //     None
    // }

    fn run_point(&self, start: Self::Var, c: Self::Param) -> EscapeState<Self::Var, Self::Deriv>
    {
        let orbit_params = OrbitParams {
            max_iter: self.max_iter(),
            min_iter: self.min_iter(),
            periodicity_tolerance: self.periodicity_tolerance(),
            escape_radius: self.escape_radius(),
        };
        let orbit = CycleDetectedOrbitFloyd::new(
            |z, c| self.map(z, c),
            |z, c| self.map_and_multiplier(z, c),
            |z, c| self.early_bailout(z, c),
            start,
            c,
            &orbit_params,
        );
        if let Some((_, state)) = orbit.last()
        {
            state
        }
        else
        {
            EscapeState::Bounded
        }
    }

    fn run_and_encode_point(&self, start: Self::Var, c: Self::Param) -> PointInfo<Self::Deriv>
    {
        let orbit_params = OrbitParams {
            max_iter: self.max_iter(),
            min_iter: self.min_iter(),
            periodicity_tolerance: self.periodicity_tolerance(),
            escape_radius: self.escape_radius(),
        };
        let orbit = CycleDetectedOrbitFloyd::new(
            |z, c| self.map(z, c),
            |z, c| self.map_and_multiplier(z, c),
            |z, c| self.early_bailout(z, c),
            start,
            c,
            &orbit_params,
        );
        if let Some((_, state)) = orbit.last()
        {
            self.encode_escape_result(state, c)
        }
        else
        {
            PointInfo::Bounded
        }
    }

    fn get_orbit_vec(&self, point: Cplx) -> Vec<Self::Var>
    {
        let param = self.param_map(point);
        let start = self.start_point(point, param);
        let orbit = SimpleOrbit::new(
            |z, c| self.map(z, c),
            start,
            param,
            self.max_iter(),
            self.escape_radius(),
        );
        orbit.map(|(z, _s)| z).collect()
    }

    fn get_orbit_info(&self, point: Cplx) -> OrbitInfo<Self::Var, Self::Param, Self::Deriv>
    {
        let param = self.param_map(point);
        let start = self.start_point(point, param);
        let result = self.run_and_encode_point(start, param);
        OrbitInfo {
            start,
            param,
            result,
        }
    }

    fn get_orbit_and_info(
        &self,
        point: Cplx,
    ) -> (
        Vec<Self::Var>,
        OrbitInfo<Self::Var, Self::Param, Self::Deriv>,
    )
    {
        let param = self.param_map(point);
        let start = self.start_point(point, param);
        let orbit_params = OrbitParams {
            max_iter: self.max_iter(),
            min_iter: self.min_iter(),
            periodicity_tolerance: self.periodicity_tolerance(),
            escape_radius: self.escape_radius(),
        };
        let orbit = CycleDetectedOrbitFloyd::new(
            |c, z| self.map(c, z),
            |c, z| self.map_and_multiplier(c, z),
            |c, z| self.early_bailout(c, z),
            start,
            param,
            &orbit_params,
        );
        let mut final_state = EscapeState::Bounded;
        let trajectory: Vec<Self::Var> = orbit
            .map(|(z, s)| {
                final_state = s;
                z
            })
            .collect();
        let result = self.encode_escape_result(final_state, param);
        (
            trajectory,
            OrbitInfo {
                start,
                param,
                result,
            },
        )
    }

    fn default_bounds(&self) -> Bounds
    {
        Bounds::centered_square(2.2)
    }

    fn default_julia_bounds(&self, _point: Cplx, _c: Self::Param) -> Bounds
    {
        Bounds::centered_square(2.2)
    }

    fn default_selection(&self) -> Cplx
    {
        Cplx::default()
    }

    fn cycle_active_plane(&mut self) {}

    fn is_dynamical(&self) -> bool
    {
        false
    }

    fn degree(&self) -> f64
    {
        2.0f64
    }

    // fn julia_set(&self, point: Cplx) -> Option<Self::Child>
    // where
    //     Self: Clone,
    // {
    //     let param = self.param_map(point);
    //     let point_grid = self
    //         .point_grid()
    //         .with_same_height(self.default_julia_bounds(point, param));
    //
    //     Some(JuliaSet {
    //         point_grid,
    //         max_iter: self.max_iter(),
    //         min_iter: self.min_iter(),
    //         parent: self.clone(),
    //         param: (self.get_param(), param),
    //     })
    // }

    fn default_coloring(&self) -> Coloring
    {
        let mut coloring = Coloring::default();
        coloring.set_algorithm(ColoringAlgorithm::PeriodMultiplier);
        coloring
    }

    fn preperiod_smooth_coloring(&self) -> ColoringAlgorithm
    {
        ColoringAlgorithm::InternalPotential {
            periodicity_tolerance: self.periodicity_tolerance(),
        }
    }

    fn preperiod_period_smooth_coloring(&self) -> ColoringAlgorithm
    {
        ColoringAlgorithm::PreperiodPeriodSmooth {
            periodicity_tolerance: self.periodicity_tolerance(),
            fill_rate: 0.04,
        }
    }

    fn internal_potential(&self, point_info: PointInfo<Self::Deriv>) -> IterCount
    {
        match point_info
        {
            PointInfo::Bounded | PointInfo::Wandering => 0.,
            PointInfo::Escaping { potential } => potential,
            PointInfo::Periodic {
                preperiod,
                period,
                multiplier,
                final_error,
            } =>
            {
                let per = IterCount::from(period);

                let mult_norm = multiplier.norm();

                // Superattracting case
                if mult_norm <= 1e-10
                {
                    let potential =
                        2. * (final_error.log(self.periodicity_tolerance())).log2() as IterCount;
                    per.mul_add(-potential, IterCount::from(preperiod))
                }
                // Parabolic case
                else if 1. - mult_norm <= 1e-5
                {
                    let potential = final_error / self.periodicity_tolerance();
                    per.mul_add(-potential, IterCount::from(preperiod))
                }
                else
                {
                    let mut potential =
                        -(final_error / self.periodicity_tolerance()).log(mult_norm) as IterCount;
                    if potential.is_infinite() || potential.is_nan()
                    {
                        potential = -0.2;
                    }
                    per.mul_add(potential, f64::from(preperiod))
                }
            }
        }
    }
}
