use crate::coloring::Coloring;
use crate::point_grid::PointGrid;

use crate::types::{Norm, PointInfo, Real};
use egui::{Color32, ColorImage};
use image::{ImageBuffer, Rgb};
use ndarray::Array2;

fn blend_rgb_images(
    imgs: Vec<ImageBuffer<Rgb<u8>, Vec<u8>>>,
) -> Option<ImageBuffer<Rgb<u8>, Vec<u8>>>
{
    let width = imgs.last()?.width();
    let height = imgs.last()?.height();
    let n = imgs.len() as f64;

    let mut img = ImageBuffer::new(width, height);

    for y in 0..height
    {
        for x in 0..width
        {
            let r: f64 = imgs.iter().map(|im| (im[(x, y)]).0[0] as f64).sum();
            let g: f64 = imgs.iter().map(|im| (im[(x, y)]).0[1] as f64).sum();
            let b: f64 = imgs.iter().map(|im| (im[(x, y)]).0[2] as f64).sum();
            img[(x, y)] = Rgb([(r / n) as u8, (g / n) as u8, (b / n) as u8]);
        }
    }

    Some(img)
}
fn blend_images(imgs: Vec<ColorImage>) -> Option<ColorImage>
{
    let [width, height] = imgs.last()?.size;
    let n = imgs.len() as f64;

    let mut img = ColorImage::new([width, height], Color32::default());

    for y in 0..height
    {
        for x in 0..width
        {
            let r: f64 = imgs.iter().map(|im| (im[(x, y)]).r() as f64).sum();
            let g: f64 = imgs.iter().map(|im| (im[(x, y)]).g() as f64).sum();
            let b: f64 = imgs.iter().map(|im| (im[(x, y)]).b() as f64).sum();
            img[(x, y)] = Color32::from_rgb((r / n) as u8, (g / n) as u8, (b / n) as u8);
        }
    }

    Some(img)
}

pub trait FractalImage
{
    fn point_grid(&self) -> &PointGrid;
    fn render(&self, coloring: &Coloring) -> ColorImage;
    fn save(&self, coloring: &Coloring, filename: String);
}

#[derive(Clone)]
pub struct IterPlane<D, const N: usize>
{
    pub iter_counts: [Array2<PointInfo<D>>; N],
    pub point_grid: PointGrid,
}

impl<D, const N: usize> IterPlane<D, N>
where
    D: Norm<Real> + std::fmt::Debug,
{
    #[must_use]
    pub fn new_default(point_grid: PointGrid) -> Self
    {
        let shape = point_grid.shape();
        let value = PointInfo::Bounded;
        let iter_counts_vec: Vec<Array2<PointInfo<D>>> =
            (0..N).map(|_| Array2::from_elem(shape, value)).collect();
        let iter_counts = iter_counts_vec.try_into().expect("Failed to convert vec to array");
        Self {
            iter_counts,
            point_grid,
        }
    }
}

impl<D, const N: usize> FractalImage for IterPlane<D, N>
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

        let images = self
            .iter_counts
            .iter()
            .map(|counts| {
                let mut img = ColorImage::new([width, self.point_grid().res_y], Color32::default());

                counts.indexed_iter().for_each(|((x, y), point_info)| {
                    img.pixels[x + (height - y - 1) * width] = coloring.map_color32(*point_info);
                });
                img
            })
            .collect();
        blend_images(images).unwrap()
    }
    fn save(&self, coloring: &Coloring, filename: String)
    {
        let res_x = u32::try_from(self.point_grid().res_x).unwrap_or(u32::MAX);
        let res_y = u32::try_from(self.point_grid().res_y).unwrap_or(u32::MAX);

        let images = self
            .iter_counts
            .iter()
            .map(|counts| {
                let mut image = ImageBuffer::new(res_x, res_y);

                for (x, y, pixel) in image.enumerate_pixels_mut()
                {
                    let iter_count = counts[(x as usize, (res_y - y - 1) as usize)];
                    *pixel = coloring.map_rgb(iter_count);
                }
                image
            })
            .collect();

        let image = blend_rgb_images(images).unwrap();

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
