#![allow(dead_code)]

use dynamo_common::prelude::*;
use egui::Color32;
use image::Rgb;

pub mod algorithms;
pub mod fractal_image;
pub mod palette;
pub mod prelude;
pub mod types;

pub use algorithms::IncoloringAlgorithm;
pub use palette::Palette;
use types::Hsv;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use self::palette::DiscretePalette;

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Coloring
{
    algorithm: IncoloringAlgorithm,
    palette: Palette,
}
impl Coloring
{
    #[must_use]
    pub const fn new(algorithm: IncoloringAlgorithm, palette: Palette) -> Self
    {
        Self { algorithm, palette }
    }

    #[must_use]
    pub fn map_color32<D>(&self, point_info: &PointInfo<D>) -> Color32
    where
        D: Polar<Real>,
    {
        use PointInfo::*;
        match point_info {
            Escaping { potential } => self.palette.map_color32(*potential),
            Periodic(data) => self.algorithm.color_periodic(&self.palette, data),
            PeriodicKnownPotential(data) => {
                self.algorithm.color_known_potential(&self.palette, data)
            }
            Bounded => self.palette.in_color,
            Wandering => self.palette.wandering_color,
            Unknown => self.palette.unknown_color,
            MarkedPoint {
                class_id,
                num_point_classes,
                ..
            } => {
                let hue = (f32::from(*class_id)) / (*num_point_classes as f32);
                Hsv {
                    hue,
                    saturation: 0.8,
                    intensity: 1.0,
                }
                .into()
            }
        }
    }

    #[must_use]
    pub fn map_rgb<D>(&self, point_info: &PointInfo<D>) -> Rgb<u8>
    where
        D: Polar<Real>,
    {
        let (r, g, b, _a) = self.map_color32(point_info).to_tuple();
        Rgb([r, g, b])
    }

    pub fn set_palette(&mut self, palette: Palette)
    {
        self.palette = palette;
    }

    #[must_use]
    pub const fn get_palette(&self) -> &Palette
    {
        &self.palette
    }
    #[must_use]
    pub fn get_palette_mut(&mut self) -> &mut Palette
    {
        &mut self.palette
    }
    #[must_use]
    pub const fn get_period_coloring(&self) -> &DiscretePalette
    {
        &self.palette.period_coloring
    }
    #[must_use]
    pub fn get_period_coloring_mut(&mut self) -> &mut DiscretePalette
    {
        &mut self.palette.period_coloring
    }
    #[must_use]
    pub const fn get_algorithm(&self) -> &IncoloringAlgorithm
    {
        &self.algorithm
    }
    pub fn get_algorithm_mut(&mut self) -> &mut IncoloringAlgorithm
    {
        &mut self.algorithm
    }
    pub fn set_interior_algorithm(&mut self, algorithm: IncoloringAlgorithm)
    {
        self.algorithm = algorithm;
    }
    #[must_use]
    pub const fn with_interior_algorithm(mut self, algorithm: IncoloringAlgorithm) -> Self
    {
        self.algorithm = algorithm;
        self
    }

    #[cfg(feature = "serde")]
    pub fn save_to_file<P>(&self, filename: P) -> std::io::Result<()>
    where
        P: AsRef<std::path::Path>,
    {
        use std::io::Write;

        let toml_string =
            toml::to_string(self.get_palette()).expect("Failed to serialize palette.");
        let mut file = std::fs::File::create(filename)?;
        file.write_all(toml_string.as_bytes())?;

        Ok(())
    }

    #[cfg(feature = "serde")]
    pub fn load_palette<P>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>>
    where
        P: AsRef<std::path::Path>,
    {
        let content = std::fs::read_to_string(path)?;
        let palette: Palette = toml::from_str(&content)?;
        self.palette = palette;
        Ok(())
    }
}

impl std::ops::Deref for Coloring
{
    type Target = Palette;

    fn deref(&self) -> &Self::Target
    {
        &self.palette
    }
}

impl std::ops::DerefMut for Coloring
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.palette
    }
}

#[cfg(test)]
mod tests
{
    #[test]
    fn hsv()
    {
        use crate::types::Hsv;
        use image::Rgb;

        let hsv = Hsv::new(0., 1., 0.4);
        let rgb = Rgb::from(hsv);
        let hsv1 = Hsv::from(rgb);

        dbg!(hsv, rgb, hsv1);

        assert!(hsv.hue - hsv1.hue < 1e-2);
        assert!(hsv.saturation - hsv1.saturation < 1e-2);
        assert!(hsv.intensity - hsv1.intensity < 1e-2);
    }
}
