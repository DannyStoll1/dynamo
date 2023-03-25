use crate::covering_maps::CoveringMap;
use crate::orbit_info::*;
use crate::palette::ColorPalette;
use crate::point_grid::*;
use crate::primitive_types::*;
use dyn_clone::DynClone;
use eframe::egui::{Color32, ColorImage};
use ndarray::Array2;
use rayon::iter::{ParallelBridge, ParallelIterator};

pub mod julia;
use julia::JuliaSet;

pub trait FractalImage {
    fn point_grid(&self) -> PointGrid;
    fn render(&self, palette: ColorPalette) -> ColorImage;
}

pub trait ParameterPlane: Sync + Send + DynClone {
    fn point_grid(&self) -> PointGrid;

    fn point_grid_mut(&mut self) -> &mut PointGrid;

    fn stop_condition(&self, iter: Period, z: ComplexNum) -> EscapeState {
        if iter > self.max_iter() {
            return EscapeState::Bounded;
        }

        let r = z.norm_sqr();
        if r > self.escape_radius() || z.is_nan() {
            EscapeState::Escaped {
                iters: iter,
                final_value: z,
            }
        } else {
            EscapeState::NotYetEscaped
        }
    }

    fn check_periodicity(
        &self,
        iter: Period,
        z0: ComplexNum,
        z1: ComplexNum,
        base_param: ComplexNum,
    ) -> EscapeState {
        if iter > self.max_iter() {
            return EscapeState::Bounded;
        }

        let r = z1.norm_sqr();
        if r > self.escape_radius() || z1.is_nan() {
            EscapeState::Escaped {
                iters: 2 * iter,
                final_value: z1,
            }
        } else if (z1 - z0).norm_sqr() < self.periodicity_tolerance() {
            if let Some(period) = self.compute_period(
                z1,
                base_param,
                self.periodicity_tolerance() * 10.,
                iter as usize,
            ) {
                EscapeState::Periodic {
                    preperiod: iter,
                    period,
                }
            } else {
                EscapeState::NotYetEscaped
            }
        } else {
            EscapeState::NotYetEscaped
        }
    }

    fn max_iter(&self) -> Period;

    fn max_iter_mut(&mut self) -> &mut Period;

    fn set_max_iter(&mut self, new_max_iter: Period);

    #[inline]
    fn escape_radius(&self) -> RealNum {
        1e16
    }

    #[inline]
    fn periodicity_tolerance(&self) -> RealNum {
        self.point_grid().bounds.area() * 1e-12
    }

    fn compute_period(
        &self,
        z0: ComplexNum,
        c: ComplexNum,
        tolerance: RealNum,
        patience: usize,
    ) -> Option<Period> {
        let mut z = z0;
        for i in 1..=patience {
            z = self.map(z, c);
            if (z - z0).norm_sqr() <= tolerance {
                if let Ok(period) = Period::try_from(i) {
                    return Some(period);
                }
                return None;
            }
        }
        None
    }

    #[inline]
    fn start_point(&self, param: ComplexNum) -> ComplexNum {
        self.param_map(param)
    }

    #[inline]
    fn param_map(&self, point: ComplexNum) -> ComplexNum {
        point
    }

    fn map(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum;
    fn dynamical_derivative(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum;
    fn parameter_derivative(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum;
    fn gradient(&self, z: ComplexNum, c: ComplexNum) -> (ComplexNum, ComplexNum) {
        (self.dynamical_derivative(z,c), self.parameter_derivative(z,c))
    }

    fn encode_escape_result(&self, state: EscapeState, base_param: ComplexNum) -> IterCount;

    fn run_point(&self, start: ComplexNum, param: ComplexNum) -> EscapeState {
        let mut tortoise = start;
        let mut hare = start;

        let mut iter = 0;
        while let EscapeState::NotYetEscaped = self.check_periodicity(iter, tortoise, hare, param) {
            tortoise = self.map(tortoise, param);
            hare = self.map(hare, param);

            // Check if we have escaped halfway through
            let mid_result = self.stop_condition(iter, hare);
            if let EscapeState::Escaped { iters, final_value } = mid_result {
                return EscapeState::Escaped {
                    iters: 2 * iters + 1,
                    final_value,
                };
            }

            hare = self.map(hare, param);
            iter += 1;
        }

        self.check_periodicity(iter, tortoise, hare, param)
    }

    fn compute_escape_times(&self) -> Array2<IterCount> {
        let mut iter_counts = Array2::zeros((self.point_grid().res_x, self.point_grid().res_y));
        iter_counts
            .indexed_iter_mut()
            .par_bridge()
            .for_each(|((x, y), count)| {
                let point = self.point_grid().map_pixel(x, y);
                let param = self.param_map(point);
                let start = self.start_point(param);

                let result = self.run_point(start, param);

                *count = self.encode_escape_result(result, point);
            });
        iter_counts
    }

    fn compute(&self) -> IterPlane {
        let iter_counts = self.compute_escape_times();
        IterPlane {
            iter_counts,
            point_grid: self.point_grid(),
        }
    }

    #[inline]
    fn get_param(&self) -> ComplexNum {
        (0.).into()
    }

    #[inline]
    fn set_param(&mut self, _value: ComplexNum) {}

    fn external_angle(&self, point: ComplexNum) -> Option<RealNum> {
        let c = self.param_map(point);
        let z = self.start_point(c);
        if let EscapeState::Escaped {
            iters: _,
            final_value,
        } = self.run_point(z, c)
        {
            return Some(final_value.arg() / TAU);
        }
        None
    }

    fn external_ray(
        &self,
        theta: RealNum,
        depth: u32,
        sharpness: u32,
        pixel_count: u32,
    ) -> Option<Vec<ComplexNum>> {
        let escape_radius = 100.;
        let pixel_width = self.point_grid().pixel_width();
        let mut m: u32;
        let mut r_m: RealNum;
        let mut t_m: ComplexNum;
        let mut temp_c: ComplexNum;
        let mut old_c: ComplexNum;
        let mut c_list = vec![escape_radius * TAUI * theta.exp()];
        let error = pixel_count as RealNum * 0.0001;
        let mut difference: RealNum;
        let mut dist: RealNum;
        let mut c_k: ComplexNum;
        let mut d_k: ComplexNum;

        for k in 0..depth {
            for j in 0..sharpness {
                m = k * sharpness + j - 1;
                r_m = escape_radius.powf(TWO.powf(-(m as RealNum) / sharpness as RealNum));
                t_m = r_m.powf(TWO.powi(k as i32)) * (TAUI * theta * TWO.powi(k as i32)).exp();
                temp_c = *c_list.last()?;

                // Repeat Newton's method until points are close together.
                loop {
                    old_c = temp_c;
                    // Recursive formula for iterates of q(z) = z^2 + c
                    c_k = old_c;
                    d_k = ComplexNum::new(1., 0.);
                    for _ in 0..k {
                        d_k = 2. * d_k * c_k + 1.;
                        c_k = self.map(c_k, old_c);
                    }
                    temp_c = old_c - (c_k - t_m) / d_k; // Newton map
                    difference = (old_c - temp_c).norm();

                    if error > difference {
                        break;
                    }
                }

                dist = (2. * c_k.norm() * (c_k.norm()).log2()) / d_k.norm();
                if dist < pixel_width {
                    break;
                }
                c_list.push(temp_c);
                if dist < pixel_width {
                    break;
                }
            }
        }
        Some(c_list)
    }

    fn get_orbit_info(&self, point: ComplexNum) -> OrbitInfo {
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


#[derive(Clone)]
pub struct IterPlane {
    iter_counts: Array2<IterCount>,
    point_grid: PointGrid,
}

impl FractalImage for IterPlane {
    fn point_grid(&self) -> PointGrid {
        self.point_grid
    }
    fn render(&self, palette: ColorPalette) -> ColorImage {
        let width = self.point_grid().res_x;
        let height = self.point_grid().res_y;
        let mut img = ColorImage::new([width, self.point_grid().res_y], Color32::default());

        self.iter_counts.indexed_iter().for_each(|((x, y), value)| {
            img.pixels[x + (height - y - 1) * width] = palette.map_color32(*value);
        });
        img
    }
}

pub trait HasDynamicalCovers: ParameterPlane + Clone {
    fn marked_cycle_curve(self, _period: Period) -> CoveringMap<Self> {
        let param_map = |c| c;
        let bounds = self.point_grid();

        println!("Marked cycle has not been implemented; falling back to base curve!");
        CoveringMap::new(self, param_map, bounds)
    }
    fn dynatomic_curve(self, _period: Period) -> CoveringMap<Self> {
        let param_map = |c| c;
        let bounds = self.point_grid();

        println!("Dynatomic curve has not been implemented; falling back to base curve!");
        CoveringMap::new(self, param_map, bounds)
    }
    fn misiurewicz_curve(self, _preperiod: Period, _period: Period) -> CoveringMap<Self> {
        let param_map = |c| c;
        let bounds = self.point_grid();

        println!("Misiurewicz curve has not been implemented; falling back to base curve!");
        CoveringMap::new(self, param_map, bounds)
    }
}
