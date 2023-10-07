use std::ops::{Deref, DerefMut};

use crate::traits::Polar;
use crate::types::{PointInfo, Real};

pub mod algorithms;
pub mod palette;
pub mod types;

use algorithms::InteriorColoringAlgorithm;
use egui::Color32;
use image::Rgb;
use palette::ColorPalette;
use types::Hsv;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use self::palette::DiscretePalette;

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Coloring
{
    algorithm: InteriorColoringAlgorithm,
    palette: ColorPalette,
}
impl Coloring
{
    #[must_use]
    pub fn map_color32<V, D>(&self, point_info: PointInfo<V, D>) -> Color32
    where
        D: Polar<Real>,
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
                class_id,
                num_point_classes,
                ..
            } =>
            {
                let hue = (f32::from(class_id)) / (num_point_classes as f32);
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
        D: Polar<Real>,
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
                class_id,
                num_point_classes: num_points,
                ..
            } =>
            {
                let hue = (f32::from(class_id)) / (num_points as f32);
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
    pub fn get_palette(&self) -> &ColorPalette
    {
        &self.palette
    }
    pub fn get_period_coloring(&self) -> &DiscretePalette
    {
        &self.palette.period_coloring
    }
    #[must_use]
    pub const fn get_algorithm(&self) -> &InteriorColoringAlgorithm
    {
        &self.algorithm
    }
    pub fn get_algorithm_mut(&mut self) -> &mut InteriorColoringAlgorithm
    {
        &mut self.algorithm
    }
    pub fn set_interior_algorithm(&mut self, algorithm: InteriorColoringAlgorithm)
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
