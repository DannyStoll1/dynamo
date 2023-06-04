use crate::{
    coloring::{algorithms::ColoringAlgorithm, Coloring},
    iter_plane::IterPlane,
    point_grid::{self, Bounds, PointGrid},
    types::param_stack::Summarize,
    types::*,
};
use ndarray::{Array2, Axis};
use num_cpus;
use rayon::iter::{ParallelBridge, ParallelIterator};
use std::cell::RefCell;
use std::fmt::Display;
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
// pub use simple_parameter_plane::SimpleParameterPlane;

pub trait ParameterPlane: Sync + Send + Clone
{
    type Var: Norm<RealNum>
        + Dist<RealNum>
        + Send
        + Default
        + From<ComplexNum>
        + Into<ComplexNum>
        + Display;
    type Param: Into<Self::Var>
        + From<ComplexNum>
        + Clone
        + Copy
        + Send
        + Sync
        + Default
        + PartialEq
        + Summarize;
    type MetaParam: ParamList + Clone + Copy + Send + Sync + Default + Summarize;
    type Deriv: Norm<RealNum> + Send + Default + From<f64> + MulAssign + Display;
    type Child: ParameterPlane + From<Self>;

    fn point_grid(&self) -> &PointGrid;
    fn point_grid_mut(&mut self) -> &mut PointGrid;
    fn with_point_grid(self, point_grid: PointGrid) -> Self;
    fn with_bounds(self, bounds: Bounds) -> Self
    {
        let point_grid = self.point_grid().new_with_same_height(bounds);
        self.with_point_grid(point_grid)
    }
    fn with_res_y(mut self, res_y: usize) -> Self
    {
        self.point_grid_mut().resize_y(res_y);
        self
    }

    fn max_iter(&self) -> Period;
    fn max_iter_mut(&mut self) -> &mut Period;
    fn set_max_iter(&mut self, new_max_iter: Period);
    fn with_max_iter(self, max_iter: Period) -> Self;

    fn name(&self) -> String;
    fn map(&self, z: Self::Var, c: Self::Param) -> Self::Var;
    fn dynamical_derivative(&self, z: Self::Var, c: Self::Param) -> Self::Deriv;
    fn parameter_derivative(&self, z: Self::Var, c: Self::Param) -> Self::Deriv;

    fn early_bailout(
        &self,
        _start: Self::Var,
        _param: Self::Param,
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

    fn start_point(&self, _point: ComplexNum, param: Self::Param) -> Self::Var
    {
        param.into()
    }

    fn param_map(&self, point: ComplexNum) -> Self::Param
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
        base_param: Self::Param,
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
                self.encode_escaping_point(iters, final_value, base_param)
            }
        }
    }

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
        let residual = (v / u).log2();
        PointInfo::Escaping {
            potential: f64::from(iters) - (residual as IterCount),
        }
    }

    fn compute_escape_times(&self) -> Array2<PointInfo<Self::Deriv>>
    {
        let mut iter_counts = Array2::from_elem(self.point_grid().shape(), PointInfo::Bounded);
        self.compute_escape_times_into(&mut iter_counts);
        iter_counts
    }

    fn compute_escape_times_into(&self, iter_counts: &mut Array2<PointInfo<Self::Deriv>>)
    {
        let orbits = ThreadLocal::new();

        let chunk_size = self.point_grid().res_y / num_cpus::get(); // or another value that gives optimal performance

        iter_counts
            .axis_chunks_iter_mut(Axis(1), chunk_size)
            .enumerate()
            .par_bridge()
            .for_each(|(chunk_idx, mut chunk)| {
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
                                self.max_iter(),
                                self.min_iter(),
                                self.periodicity_tolerance(),
                                self.escape_radius(),
                            ))
                        })
                        .borrow_mut();

                    orbit.reset(param, start);
                    let result = orbit.run_until_complete();
                    *count = self.encode_escape_result(result, param);
                });
            });
    }

    fn compute(&self) -> IterPlane<Self::Deriv>
    {
        let iter_counts = self.compute_escape_times();
        IterPlane {
            iter_counts,
            point_grid: self.point_grid().clone(),
        }
    }

    fn compute_into(&self, iter_plane: &mut IterPlane<Self::Deriv>)
    {
        self.compute_escape_times_into(&mut iter_plane.iter_counts);
    }

    #[inline]
    fn get_param(&self) -> Self::MetaParam
    {
        Self::MetaParam::default()
    }

    #[inline]
    fn get_local_param(&self) -> <Self::MetaParam as ParamList>::Param
    {
        <Self::MetaParam as ParamList>::Param::default()
    }

    #[inline]
    fn set_meta_param(&mut self, _value: Self::MetaParam) {}

    #[inline]
    fn set_param(&mut self, _value: <Self::MetaParam as ParamList>::Param) {}

    #[inline]
    fn with_param(mut self, param: <Self::MetaParam as ParamList>::Param) -> Self
    {
        self.set_param(param);
        self
    }

    #[inline]
    fn critical_points_child(&self, _param: Self::Param) -> Vec<Self::Var>
    {
        vec![]
    }

    #[inline]
    fn critical_points(&self) -> Vec<Self::Var>
    {
        vec![]
    }

    fn cycles_child(&self, _param: Self::Param, _period: Period) -> Vec<Self::Var>
    {
        vec![]
    }

    fn cycles(&self, _period: Period) -> Vec<Self::Var>
    {
        vec![]
    }

    // fn external_angle(&self, point: ComplexNum) -> Option<RealNum>
    // {
    //     let c = self.param_map(point);
    //     let z = self.start_point(c);
    //     if let EscapeState::Escaped { iters, final_value } =
    //         self.run_until_escape(z, c, 10., self.max_iter())
    //     {
    //         let error = 1e-12;
    //         let mut curr = c;
    //         let mut difference: ComplexNum;
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

    fn run_point(&self, start: Self::Var, param: Self::Param) -> PointInfo<Self::Deriv>
    {
        let orbit = CycleDetectedOrbitFloyd::new(
            |z, c| self.map(z, c),
            |z, c| self.map_and_multiplier(z, c),
            |z, c| self.early_bailout(z, c),
            start,
            param,
            self.max_iter(),
            self.min_iter(),
            self.periodicity_tolerance(),
            self.escape_radius(),
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

    fn get_orbit_vec(&self, point: ComplexNum) -> Vec<Self::Var>
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

    fn get_orbit_info(&self, point: ComplexNum) -> OrbitInfo<Self::Var, Self::Param, Self::Deriv>
    {
        let param = self.param_map(point);
        let start = self.start_point(point, param);
        let result = self.run_point(start, param);
        OrbitInfo {
            start,
            param,
            result,
        }
    }

    fn get_orbit_and_info(
        &self,
        point: ComplexNum,
    ) -> (
        Vec<Self::Var>,
        OrbitInfo<Self::Var, Self::Param, Self::Deriv>,
    )
    {
        let param = self.param_map(point);
        let start = self.start_point(point, param);
        let orbit = CycleDetectedOrbitFloyd::new(
            |c, z| self.map(c, z),
            |c, z| self.map_and_multiplier(c, z),
            |c, z| self.early_bailout(c, z),
            start,
            param,
            self.max_iter(),
            self.min_iter(),
            self.periodicity_tolerance(),
            self.escape_radius(),
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

    fn default_julia_bounds(&self, _point: ComplexNum, _param: Self::Param) -> Bounds
    {
        Bounds::centered_square(2.2)
    }

    fn default_selection(&self) -> ComplexNum
    {
        ComplexNum::default()
    }

    fn cycle_active_plane(&mut self) {}

    fn is_dynamical(&self) -> bool
    {
        false
    }

    // fn julia_set(&self, point: ComplexNum) -> Option<Self::Child>
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
            PointInfo::Bounded => 0.,
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
            PointInfo::Wandering => 0.,
        }
    }
}
