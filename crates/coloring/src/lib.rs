#![allow(dead_code)]

use dynamo_common::prelude::*;

pub mod algorithms;
pub mod fractal_image;
pub mod palette;
pub mod prelude;
pub mod types;

pub use algorithms::IncoloringAlgorithm;
pub use palette::Palette;
use types::{FromColor, Hsv};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[macro_use]
extern crate log;

use self::palette::DiscretePalette;

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Coloring
{
    algorithm: IncoloringAlgorithm,
    palette: Palette,
    esc_period: Period,
    do_escape_phase_coloring: bool,
}
impl Coloring
{
    #[must_use]
    pub const fn new(algorithm: IncoloringAlgorithm, palette: Palette) -> Self
    {
        Self {
            algorithm,
            palette,
            esc_period: 1,
            do_escape_phase_coloring: false,
        }
    }

    #[must_use]
    pub fn map<D, T>(&self, point_info: &PointInfo<D>) -> T
    where
        D: Polar<Real>,
        T: FromColor,
    {
        use PointInfo::{
            Bounded, DistanceEstimate, Escaping, MarkedPoint, Periodic, PeriodicKnownPotential,
            Unknown, Wandering,
        };
        match point_info {
            Escaping {
                potential,
                phase: Some(phase),
            } if self.do_escape_phase_coloring => {
                self.palette
                    .map_phase(potential.ln(), *phase, self.esc_period)
            }
            Escaping { potential, .. } => self.palette.map(potential.ln()),
            Periodic(data) => self.algorithm.color_periodic(&self.palette, data),
            PeriodicKnownPotential(data) => {
                self.algorithm.color_known_potential(&self.palette, data)
            }
            Bounded => T::from_color32(self.palette.in_color),
            DistanceEstimate { distance, phase } if self.do_escape_phase_coloring => self
                .palette
                .map_phase(-distance.ln() / 2., *phase, self.esc_period),
            DistanceEstimate { distance, .. } => self.palette.map(-distance.ln() / 2.),
            Wandering => T::from_color32(self.palette.wandering_color),
            Unknown => T::from_color32(self.palette.unknown_color),
            MarkedPoint {
                class_id,
                num_point_classes,
                ..
            } => {
                let hue = (f32::from(*class_id) + 0.5) / (*num_point_classes as f32);
                Hsv {
                    hue,
                    saturation: 0.7,
                    intensity: 0.6,
                }
                .into()
            }
        }
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

    #[must_use]
    pub const fn with_escape_period(mut self, esc_period: Period) -> Self
    {
        self.esc_period = esc_period;
        self
    }

    pub fn toggle_escape_phase_coloring(&mut self)
    {
        self.do_escape_phase_coloring ^= true;
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

    #[test]
    fn cielch()
    {
        use crate::types::{Lchuv, Luv, Xyz};

        let lch = Lchuv {
            l: 0.1161,
            c: 1.,
            h: 0.625,
        };
        let luv = Luv::from(lch);
        dbg!(luv);
        let xyz = Xyz::from(luv);
        dbg!(xyz);
    }
}
