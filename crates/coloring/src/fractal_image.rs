use crate::Coloring;
use dynamo_common::prelude::*;
use egui::{Color32, ColorImage};
use image::{ImageBuffer, Rgb};

pub trait FractalImage
{
    type Image;
    fn point_grid(&self) -> &PointGrid;
    fn render(&self, coloring: &Coloring) -> ColorImage;
    fn render_into(&self, image: &mut ColorImage, coloring: &Coloring);
    fn save(&self, coloring: &Coloring, filename: String);
    fn write_image(&self, coloring: &Coloring) -> Self::Image;
}

impl<D> FractalImage for IterPlane<D>
where
    D: Polar<Real>,
{
    type Image = ImageBuffer<Rgb<u8>, Vec<u8>>;
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
                img.pixels[x + (height - y - 1) * width] = coloring.map_color32(point_info);
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
                image.pixels[x + (height - y - 1) * width] = coloring.map_color32(point_info);
            });
    }
    fn save(&self, coloring: &Coloring, filename: String)
    {
        let res_x = u32::try_from(self.point_grid().res_x).unwrap_or(u32::MAX);
        let res_y = u32::try_from(self.point_grid().res_y).unwrap_or(u32::MAX);
        let mut image = ImageBuffer::new(res_x, res_y);

        for (x, y, pixel) in image.enumerate_pixels_mut() {
            let iter_count = &self.iter_counts[(x as usize, (res_y - y - 1) as usize)];
            *pixel = coloring.map_rgb(iter_count);
        }
        if let Err(e) = image.save(filename.clone()) {
            println!("Error saving file: {e:?}");
        } else {
            println!("Image saved to {filename}");
        }
    }
    fn write_image(&self, coloring: &Coloring) -> Self::Image
    {
        let res_x = u32::try_from(self.point_grid().res_x).unwrap_or(u32::MAX);
        let res_y = u32::try_from(self.point_grid().res_y).unwrap_or(u32::MAX);
        let mut image = ImageBuffer::new(res_x, res_y);

        for (x, y, pixel) in image.enumerate_pixels_mut() {
            let iter_count = &self.iter_counts[(x as usize, (res_y - y - 1) as usize)];
            *pixel = coloring.map_rgb(iter_count);
        }
        image
    }
}
