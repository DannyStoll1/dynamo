use crate::covering_maps::CoveringMap;
use crate::palette::ColorPalette;
use crate::point_grid::PointGrid;
use crate::primitive_types::*;
use eframe::egui::{Color32, ColorImage};
use ndarray::Array2;
use rayon::iter::{ParallelBridge, ParallelIterator};

pub trait FractalImage {
    fn point_grid(&self) -> PointGrid;
    fn render(&self, palette: ColorPalette) -> ColorImage;
    // fn save(&self, palette: ColorPalette, filename: String) {
    //     let image = self.render(palette);
    //     image.save(filename).unwrap();
    // }
    // fn show(&self, ui: &mut Ui, palette: ColorPalette) -> Result<(), Box<dyn std::error::Error>> {
    //     let image = self.render(palette);
    //     image.show(ui);
    //
    //     // let window = create_window("image", Default::default())?;
    //     // window.set_image("image0", image)?;
    //
    //     Ok(())
    // }
}

pub trait ParameterPlane: Sync {
    fn point_grid(&self) -> PointGrid;

    fn point_grid_mut(&mut self) -> &mut PointGrid;

    fn point_grid_child(&self) -> PointGrid;

    fn point_grid_child_mut(&mut self) -> &mut PointGrid;

    fn stop_condition(&self, iter: Period, z: ComplexNum) -> EscapeState {
        if iter > self.max_iter() {
            return EscapeState::Bounded;
        }

        let r = z.norm_sqr();
        if r > self.escape_radius() {
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
        if r > self.escape_radius() {
            EscapeState::Escaped {
                iters: 2 * iter,
                final_value: z1,
            }
        } else if (z1 - z0).norm_sqr() < self.periodicity_tolerance() {
            if let Some(period) = self.compute_period(z1, base_param, self.periodicity_tolerance(), iter as usize)
            {
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

    #[inline(always)]
    fn escape_radius(&self) -> Float {
        1e16
    }

    #[inline(always)]
    fn periodicity_tolerance(&self) -> Float {
        self.point_grid().bounds.area() * 1e-12
    }

    fn compute_period(
        &self,
        z0: ComplexNum,
        c: ComplexNum,
        tolerance: Float,
        patience: usize,
    ) -> Option<Period> {
        let mut z = z0;
        for i in 1..=patience {
            z = self.map(z, c);
            if (z - z0).norm_sqr() <= tolerance {
                return Some(i as Period);
            }
        }
        None
    }

    fn start_point(&self, c: ComplexNum) -> ComplexNum {
        c
    }

    fn param_map(&self, c: ComplexNum) -> ComplexNum {
        c
    }

    fn map(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum;

    fn encode_escape_result(&self, state: EscapeState, base_param: ComplexNum) -> IterCount;

    fn compute_escape_times(&self) -> Array2<IterCount> {
        let mut iter_counts = Array2::zeros((self.point_grid().res_x, self.point_grid().res_y));
        iter_counts
            .indexed_iter_mut()
            .par_bridge()
            .for_each(|((x, y), count)| {
                let point = self.point_grid().map_pixel(x, y);
                let c = self.param_map(point);
                let mut z0 = self.start_point(c);
                let mut z1 = z0;

                let mut iter = 0;
                while let EscapeState::NotYetEscaped = self.check_periodicity(iter, z0, z1, c) {
                    z0 = self.map(z0, c);
                    z1 = self.map(z1, c);

                    // Check if we have escaped halfway through
                    let mid_result = self.stop_condition(iter, z1);
                    if let EscapeState::Escaped { iters, final_value } = mid_result {
                        let result = EscapeState::Escaped {
                            iters: 2 * iters + 1,
                            final_value,
                        };
                        *count = self.encode_escape_result(result, c);
                        return;
                    }

                    z1 = self.map(z1, c);
                    iter += 1;
                }

                let result = self.check_periodicity(iter, z0, z1, c);

                *count = self.encode_escape_result(result, c);
            });
        iter_counts
    }

    fn compute_escape_times_child(&self, param: ComplexNum) -> Array2<IterCount> {
        let mut iter_counts =
            Array2::zeros((self.point_grid_child().res_x, self.point_grid_child().res_y));
        let c = self.param_map(param);
        iter_counts
            .indexed_iter_mut()
            .par_bridge()
            .for_each(|((x, y), count)| {
                let mut z0 = self.point_grid_child().map_pixel(x, y);
                let mut z1 = z0;

                let mut iter = 0;
                while let EscapeState::NotYetEscaped = self.check_periodicity(iter, z0, z1, c) {
                    z0 = self.map(z0, c);
                    z1 = self.map(z1, c);

                    // Check if we have escaped halfway through
                    let mid_result = self.stop_condition(iter, z1);
                    if let EscapeState::Escaped { iters, final_value } = mid_result {
                        let result = EscapeState::Escaped {
                            iters: 2 * iters + 1,
                            final_value,
                        };
                        *count = self.encode_escape_result(result, c);
                        return;
                    }

                    z1 = self.map(z1, c);
                    iter += 1;
                }

                let result = self.check_periodicity(iter, z0, z1, c);

                *count = self.encode_escape_result(result, c);
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

    fn compute_child(&self, param: ComplexNum) -> IterPlane {
        let iter_counts = self.compute_escape_times_child(param);
        IterPlane {
            iter_counts,
            point_grid: self.point_grid_child(),
        }
    }

    fn to_cover(self, covering_map: fn(ComplexNum) -> ComplexNum) -> CoveringMap<Self>
    where
        Self: Copy,
    {
        CoveringMap::new(self, covering_map, self.point_grid())
    }

    // fn spawn_julia(&self, point: ComplexNum) -> JuliaSet {
    //     let c = self.param_map(point);
    //     let map = |z|self.map(z, c);
    //     let stop_condition = |iter, z|{
    //         self.stop_condition(iter, z)
    //     };
    //     let escape_encoding = |iter, state|{
    //         self.encode_escape_result(iter, state, c)
    //     };
    //
    //     JuliaSet::new(
    //         self.point_grid(),
    //         Box::from(map),
    //         Box::from(stop_condition),
    //         Box::from(escape_encoding),
    //     )
    // }

    fn name(&self) -> String;
}

pub trait DynamicalPlane {
    fn point_grid(&self) -> PointGrid;

    fn stop_condition(&self, iter: Period, z: ComplexNum) -> EscapeState;

    fn check_periodicity(&self, iter: Period, z0: ComplexNum, z1: ComplexNum) -> EscapeState;

    fn param_map(&self, z: ComplexNum) -> ComplexNum {
        z
    }

    fn map(&self, z: ComplexNum) -> ComplexNum;

    fn encode_escape_result(&self, state: EscapeState) -> IterCount;

    fn compute_escape_times(&self) -> Array2<IterCount> {
        let mut iter_counts = Array2::zeros((self.point_grid().res_x, self.point_grid().res_y));
        iter_counts.indexed_iter_mut().for_each(|((x, y), count)| {
            let mut tortoise = self.point_grid().map_pixel(x, y);
            let mut hare = tortoise;
            let mut iter = 0;
            while let EscapeState::NotYetEscaped = self.check_periodicity(iter, tortoise, hare) {
                tortoise = self.map(tortoise);
                hare = self.map(hare);

                // Check if we have escaped halfway through
                let mid_result = self.stop_condition(iter, hare);
                if let EscapeState::Escaped { iters, final_value } = mid_result {
                    let result = EscapeState::Escaped {
                        iters: 2 * iters + 1,
                        final_value,
                    };
                    *count = self.encode_escape_result(result);
                    return;
                }

                hare = self.map(hare);
                iter += 1;
            }

            let result = self.check_periodicity(iter, tortoise, hare);

            *count = self.encode_escape_result(result);
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

pub trait HasDynamicalCovers: ParameterPlane + Copy {
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
