use crate::coloring::Coloring;
use crate::point_grid::PointGrid;

use crate::types::*;
use egui::{Color32, ColorImage};
use image::ImageBuffer;
use ndarray::Array2;

pub trait FractalImage
{
    fn point_grid(&self) -> &PointGrid;
    fn render(&self, coloring: &Coloring) -> ColorImage;
    fn save(&self, coloring: &Coloring, filename: String);
}

#[derive(Clone)]
pub struct IterPlane<D>
{
    pub iter_counts: Array2<PointInfo<D>>,
    pub point_grid: PointGrid,
}

impl<D> FractalImage for IterPlane<D>
where
    D: Norm<Real>,
{
    fn point_grid(&self) -> &PointGrid
    {
        &self.point_grid
    }
    fn render(&self, coloring: &Coloring) -> ColorImage
    {
        let width = self.point_grid().res_x;
        let height = self.point_grid().res_y;
        let mut img = ColorImage::new([width, self.point_grid().res_y], Color32::default());

        self.iter_counts
            .indexed_iter()
            .for_each(|((x, y), point_info)| {
                img.pixels[x + (height - y - 1) * width] = coloring.map_color32(*point_info);
            });
        img
    }
    fn save(&self, coloring: &Coloring, filename: String)
    {
        let res_x = self.point_grid().res_x as u32;
        let res_y = self.point_grid().res_y as u32;
        let mut image = ImageBuffer::new(res_x, res_y);

        for (x, y, pixel) in image.enumerate_pixels_mut()
        {
            let iter_count = self.iter_counts[(x as usize, (res_y - y - 1) as usize)];
            *pixel = coloring.map_rgb(iter_count);
        }
        if let Err(e) = image.save(filename.clone())
        {
            println!("Error encountered saving file: {e:?}")
        }
        else
        {
            println!("Image saved to {}", filename);
        }
    }
}
