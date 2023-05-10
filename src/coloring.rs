use std::ops::{Deref, DerefMut};

use crate::types::PointInfo;
pub mod color_types;
pub mod coloring_algorithm;
pub mod palette;

use color_types::Hsv;
use coloring_algorithm::ColoringAlgorithm;
use epaint::Color32;
use image::Rgb;
use palette::ColorPalette;

#[derive(Clone, Copy)]
pub struct Coloring
{
    algorithm: ColoringAlgorithm,
    palette: ColorPalette,
}
impl Coloring
{
    #[must_use] pub fn map_color32(&self, point_info: PointInfo) -> Color32
    {
        use PointInfo::{Bounded, Escaping, Periodic};
        match point_info
        {
            Escaping { potential } => self.palette.map_color32(potential),
            Periodic {
                period,
                preperiod,
                multiplier,
                final_error,
            } => self
                .algorithm
                .color_periodic(self.palette, period, preperiod, multiplier, final_error),
            Bounded => self.palette.in_color,
        }
    }

    #[must_use] pub fn map_rgb(&self, point_info: PointInfo) -> Rgb<u8>
    {
        use PointInfo::{Bounded, Escaping, Periodic};
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
        }
    }

    pub fn set_palette(&mut self, palette: ColorPalette)
    {
        self.palette = palette;
    }
    #[must_use] pub const fn get_algorithm(&self) -> &ColoringAlgorithm
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

impl Default for Coloring
{
    fn default() -> Self
    {
        Self {
            palette: ColorPalette::black(32.),
            algorithm: ColoringAlgorithm::PeriodMultiplier,
        }
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
