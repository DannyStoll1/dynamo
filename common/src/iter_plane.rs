use crate::coloring::Coloring;
use crate::point_grid::PointGrid;

use crate::orbit_info::PointInfo;
use crate::traits::Polar;
use crate::types::Real;
use egui::{Color32, ColorImage};
use image::ImageBuffer;
use ndarray::Array2;

pub trait FractalImage
{
    fn point_grid(&self) -> &PointGrid;
    fn render(&self, coloring: &Coloring) -> ColorImage;
    fn render_into(&self, image: &mut ColorImage, coloring: &Coloring);
    fn save(&self, coloring: &Coloring, filename: String);
}

#[derive(Clone)]
pub struct IterPlane<V, D>
{
    pub iter_counts: Array2<PointInfo<V, D>>,
    pub point_grid: PointGrid,
}

impl<V, D> IterPlane<V, D>
where
    D: Clone,
    V: Clone,
{
    #[must_use]
    pub fn create(point_grid: PointGrid) -> Self
    {
        let iter_counts = Array2::from_elem(point_grid.shape(), PointInfo::Bounded);
        Self {
            iter_counts,
            point_grid,
        }
    }
}

impl<V, D> FractalImage for IterPlane<V, D>
where
    D: Polar<Real>,
    V: Clone,
{
    fn point_grid(&self) -> &PointGrid
    {
        &self.point_grid
    }
    fn render(&self, coloring: &Coloring) -> ColorImage
    {
        let width = self.point_grid().res_x;
        let height = self.point_grid().res_y;
        let mut img = ColorImage::new([width, height], Color32::default());

        self.iter_counts
            .indexed_iter()
            .for_each(|((x, y), point_info)| {
                img.pixels[x + (height - y - 1) * width] = coloring.map_color32(point_info.clone());
            });
        img
    }
    fn render_into(&self, image: &mut ColorImage, coloring: &Coloring)
    {
        let width = self.point_grid().res_x;
        let height = self.point_grid().res_y;

        self.iter_counts
            .indexed_iter()
            .for_each(|((x, y), point_info)| {
                image.pixels[x + (height - y - 1) * width] = coloring.map_color32(point_info.clone());
            });
    }
    fn save(&self, coloring: &Coloring, filename: String)
    {
        let res_x = u32::try_from(self.point_grid().res_x).unwrap_or(u32::MAX);
        let res_y = u32::try_from(self.point_grid().res_y).unwrap_or(u32::MAX);
        let mut image = ImageBuffer::new(res_x, res_y);

        for (x, y, pixel) in image.enumerate_pixels_mut()
        {
            let iter_count = self.iter_counts[(x as usize, (res_y - y - 1) as usize)].clone();
            *pixel = coloring.map_rgb(iter_count);
        }
        if let Err(e) = image.save(filename.clone())
        {
            println!("Error encountered saving file: {e:?}");
        }
        else
        {
            println!("Image saved to {filename}");
        }
    }
}
