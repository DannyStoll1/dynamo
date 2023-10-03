use std::ops::{Deref, DerefMut};

use crate::types::{Norm, PointInfo, Real};
pub mod algorithms;
pub mod palette;
pub mod types;

use algorithms::ColoringAlgorithm;
use egui::Color32;
use image::Rgb;
use palette::ColorPalette;
use types::Hsv;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Coloring
{
    algorithm: ColoringAlgorithm,
    palette: ColorPalette,
}
impl Coloring
{
    #[must_use]
    pub fn map_color32<V, D>(&self, point_info: PointInfo<V, D>) -> Color32
    where
        D: Norm<Real>,
        V: Clone,
    {
        use PointInfo::*;
        match point_info
        {
            Escaping { potential } => self.palette.map_color32(potential),
            Periodic { data } => self.algorithm.color_periodic(self.palette, data),
            Bounded => self.palette.in_color,
            Wandering => self.palette.wandering_color,
            MarkedPoint {
                point_id,
                num_points,
                ..
            } =>
            {
                let hue = (point_id as f32) / (num_points as f32);
                Hsv {
                    hue,
                    saturation: 0.8,
                    luminosity: 1.0,
                }
                .into()
            }
        }
    }

    #[must_use]
    pub fn map_rgb<V, D>(&self, point_info: PointInfo<V, D>) -> Rgb<u8>
    where
        D: Norm<Real>,
        V: Clone,
    {
        use PointInfo::*;
        match point_info
        {
            Escaping { potential } => self.palette.map_rgb(potential),
            Periodic { data } =>
            {
                let (r, g, b, _) = self.algorithm.color_periodic(self.palette, data).to_tuple();
                Rgb([r, g, b])
            }
            Bounded => self.palette.map_rgb(0.),
            Wandering =>
            {
                let (r, g, b, _a) = self.palette.wandering_color.to_tuple();
                Rgb([r, g, b])
            }
            MarkedPoint {
                point_id,
                num_points,
                ..
            } =>
            {
                let hue = (point_id as f32) / (num_points as f32);
                Hsv {
                    hue,
                    saturation: 0.8,
                    luminosity: 1.0,
                }
                .into()
            }
        }
    }

    pub fn set_palette(&mut self, palette: ColorPalette)
    {
        self.palette = palette;
    }
    #[must_use]
    pub const fn get_algorithm(&self) -> &ColoringAlgorithm
    {
        &self.algorithm
    }
    pub fn get_algorithm_mut(&mut self) -> &mut ColoringAlgorithm
    {
        &mut self.algorithm
    }
    pub fn set_algorithm(&mut self, algorithm: ColoringAlgorithm)
    {
        self.algorithm = algorithm;
    }
}

impl Deref for Coloring
{
    type Target = ColorPalette;

    fn deref(&self) -> &Self::Target
    {
        &self.palette
    }
}

impl DerefMut for Coloring
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.palette
    }
}
