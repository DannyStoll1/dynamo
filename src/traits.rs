use crate::covering_maps::CoveringMap;
use crate::julia::JuliaSet;
use crate::palette::ColorPalette;
use crate::point_grid::PointGrid;
use crate::primitive_types::{ComplexNum, EscapeState, Period};
use eframe::egui::{Color32, ColorImage, Ui};
use image::ImageBuffer;
use ndarray::Array2;
use show_image::create_window;

pub trait FractalImage {
    fn point_grid(&self) -> PointGrid;
    fn render(&self, palette: ColorPalette) -> ColorImage;
    fn draw(&self, palette: ColorPalette, filename: String) {
        let image = self.render(palette);
        // image.save(filename).unwrap();
    }
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

pub trait ParameterPlane {
    fn point_grid(&self) -> PointGrid;

    fn point_grid_julia(&self) -> PointGrid {
        self.point_grid().with_new_bounds(-2.5, 2.5, -2.5, 2.5)
    }

    fn stop_condition(&self, iter: i32, z: ComplexNum) -> EscapeState;

    fn start_point(&self, c: ComplexNum) -> ComplexNum {
        c
    }

    fn param_map(&self, c: ComplexNum) -> ComplexNum {
        c
    }

    fn map(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum;

    fn encode_escape_result(&self, iter: i32, state: EscapeState, base_param: ComplexNum) -> f64;

    fn compute_escape_times(&self) -> Array2<f64> {
        let mut iter_counts = Array2::zeros((self.point_grid().res_x, self.point_grid().res_y));
        for ((x, y), point) in self.point_grid().iter() {
            let c = self.param_map(point);
            let mut z = self.start_point(c);
            let mut iter = 0;
            while let EscapeState::NotYetEscaped = self.stop_condition(iter, z) {
                z = self.map(z, c);
                iter += 1;
            }

            let result = self.stop_condition(iter, z);

            let iter_count = self.encode_escape_result(iter, result, c);
            iter_counts[(x, y)] = iter_count;
        }
        iter_counts
    }

    fn compute_escape_times_julia(&self, param: ComplexNum) -> Array2<f64> {
        let mut iter_counts =
            Array2::zeros((self.point_grid_julia().res_x, self.point_grid_julia().res_y));
        let c = self.param_map(param);
        for ((x, y), point) in self.point_grid_julia().iter() {
            let mut z = point;
            let mut iter = 0;
            while let EscapeState::NotYetEscaped = self.stop_condition(iter, z) {
                z = self.map(z, c);
                iter += 1;
            }

            let result = self.stop_condition(iter, z);

            let iter_count = self.encode_escape_result(iter, result, c);
            iter_counts[(x, y)] = iter_count;
        }
        iter_counts
    }

    fn compute(&self) -> IterPlane {
        let iter_counts = self.compute_escape_times();
        IterPlane {
            iter_counts,
            point_grid: self.point_grid(),
        }
    }

    fn compute_julia(&self, param: ComplexNum) -> IterPlane {
        let iter_counts = self.compute_escape_times_julia(param);
        IterPlane {
            iter_counts,
            point_grid: self.point_grid_julia(),
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

    fn stop_condition(&self, iter: i32, z: ComplexNum) -> EscapeState;

    fn param_map(&self, z: ComplexNum) -> ComplexNum {
        z
    }

    fn map(&self, z: ComplexNum) -> ComplexNum;

    fn encode_escape_result(&self, iter: i32, state: EscapeState) -> f64;

    fn compute_escape_times(&self) -> Array2<f64> {
        let mut iter_counts = Array2::zeros((self.point_grid().res_x, self.point_grid().res_y));
        for ((x, y), point) in self.point_grid().iter() {
            let mut z = self.param_map(point);
            let mut iter = 0;
            while let EscapeState::NotYetEscaped = self.stop_condition(iter, z) {
                z = self.map(z);
                iter += 1;
            }

            let result = self.stop_condition(iter, z);

            let iter_count = self.encode_escape_result(iter, result);
            iter_counts[(x, y)] = iter_count;
        }
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
    iter_counts: Array2<f64>,
    point_grid: PointGrid,
}

impl FractalImage for IterPlane {
    fn point_grid(&self) -> PointGrid {
        self.point_grid
    }
    // fn render(&self, palette: ColorPalette) -> ImageBuffer<image::Rgb<u8>, Vec<u8>> {
    //     let mut img = ImageBuffer::new(
    //         self.point_grid().res_x as u32,
    //         self.point_grid().res_y as u32,
    //     );
    //
    //     for (x, y, pixel) in img.enumerate_pixels_mut() {
    //         let iter_count = self.iter_counts[(x as usize, y as usize)];
    //         *pixel = palette.color_map(iter_count);
    //     }
    //     img
    // }
    fn render(&self, palette: ColorPalette) -> ColorImage {
        let width = self.point_grid().res_x;
        let mut img = ColorImage::new([width, self.point_grid().res_y], Color32::default());

        for ((x, y), value) in self.iter_counts.indexed_iter() {
            img.pixels[x + y * width] = palette.map_color32(*value);
        }
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
