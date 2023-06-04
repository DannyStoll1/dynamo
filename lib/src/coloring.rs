use std::ops::{Deref, DerefMut};

use crate::types::*;
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

#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Coloring
{
    algorithm: ColoringAlgorithm,
    palette: ColorPalette,
}
impl Coloring
{
    #[must_use]
    pub fn map_color32<D>(&self, point_info: PointInfo<D>) -> Color32
    where
        D: Norm<RealNum>,
    {
        use PointInfo::*;
        match point_info
        {
            Escaping { potential } => self.palette.map_color32(potential),
            Periodic {
                period,
                preperiod,
                multiplier,
                final_error,
            } => self.algorithm.color_periodic(
                self.palette,
                period,
                preperiod,
                multiplier,
                final_error,
            ),
            Bounded => self.palette.in_color,
            Wandering => self.palette.wandering_color,
        }
    }

    #[must_use]
    pub fn map_rgb<D>(&self, point_info: PointInfo<D>) -> Rgb<u8>
    where
        D: Norm<RealNum>,
    {
        use PointInfo::*;
        match point_info
        {
            Escaping { potential } => self.palette.map_rgb(potential),
            Periodic {
                period,
                preperiod,
                multiplier,
                final_error,
            } =>
            {
                let (r, g, b, _) = self
                    .algorithm
                    .color_periodic(self.palette, period, preperiod, multiplier, final_error)
                    .to_tuple();
                Rgb([r, g, b])
            }
            Bounded => self.palette.map_rgb(0.),
            Wandering =>
            {
                let (r, g, b, _a) = self.palette.wandering_color.to_tuple();
                Rgb([r, g, b])
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
