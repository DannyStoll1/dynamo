use crate::types::{FromCartesian, FromPolar, Lchab, RgbLinear, Xyz};

use super::Hsv;
use dynamo_common::types::IterCount;
use dynamo_common::{symbolic_dynamics::OrbitSchema, types::Period};
use egui::Color32;
use rand::prelude::*;
use rand_distr::{ChiSquared, Distribution, Uniform};
use std::f64::consts::PI;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CartesianColorSpace
{
    #[default]
    Rgb,
    Xyz,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Sinusoid
{
    period: f64,
    phase: f64,
    amplitude: f64,
    midline: f64,
    #[serde(default)]
    degree: i32,
}
impl Sinusoid
{
    const fn new(period: f64) -> Self
    {
        Self {
            period,
            phase: 0.,
            amplitude: 0.5,
            midline: 0.5,
            degree: 2,
        }
    }
    fn get_value_f64(&self, potential: IterCount) -> f64
    {
        let theta = 2.0 * PI * (potential / self.period - self.phase);
        let mut val = theta.cos();
        if self.degree > 1 {
            val = val.powi(self.degree).abs() * val.signum();
        }
        self.amplitude.mul_add(val, self.midline)
    }
    fn get_period_mut(&mut self) -> &mut f64
    {
        &mut self.period
    }
    fn get_amplitude_mut(&mut self) -> &mut f64
    {
        &mut self.amplitude
    }
    fn get_midline_mut(&mut self) -> &mut f64
    {
        &mut self.midline
    }
    fn get_phase_mut(&mut self) -> &mut f64
    {
        &mut self.phase
    }
    fn set_period(&mut self, period: f64)
    {
        self.period = period;
    }
    fn set_amplitude(&mut self, amplitude: f64)
    {
        self.amplitude = amplitude;
    }
    fn set_midline(&mut self, midline: f64)
    {
        self.midline = midline;
    }
    fn set_phase(&mut self, phase: f64)
    {
        self.phase = phase;
    }
}
impl Default for Sinusoid
{
    fn default() -> Self
    {
        Self {
            period: 8.,
            amplitude: 0.5,
            midline: 0.5,
            phase: 0.,
            degree: 1,
        }
    }
}

mod defaults
{
    use egui::Color32;
    pub(super) const fn white() -> Color32
    {
        Color32::WHITE
    }

    pub(super) const fn black() -> Color32
    {
        Color32::BLACK
    }

    pub(super) const fn gray() -> Color32
    {
        Color32::GRAY
    }

    pub(super) const fn brown() -> Color32
    {
        Color32::BROWN
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Palette
{
    pub color_map_r: Sinusoid,
    pub color_map_g: Sinusoid,
    pub color_map_b: Sinusoid,
    #[cfg_attr(feature = "serde", serde(default = "DiscretePalette::default"))]
    pub period_coloring: DiscretePalette,
    #[cfg_attr(feature = "serde", serde(default = "defaults::black"))]
    pub in_color: Color32,
    #[cfg_attr(feature = "serde", serde(default = "defaults::brown"))]
    pub wandering_color: Color32,
    #[cfg_attr(feature = "serde", serde(default = "defaults::gray"))]
    pub unknown_color: Color32,
    #[cfg_attr(feature = "serde", serde(default = "CartesianColorSpace::default"))]
    pub color_space: CartesianColorSpace,
}

impl Palette
{
    #[must_use]
    pub const fn new(period_r: f64, period_g: f64, period_b: f64) -> Self
    {
        Self {
            color_map_r: Sinusoid::new(period_r),
            color_map_g: Sinusoid::new(period_g),
            color_map_b: Sinusoid::new(period_b),
            period_coloring: DiscretePalette::standard(),
            in_color: Color32::BLACK,
            wandering_color: Color32::BROWN,
            unknown_color: Color32::GRAY,
            color_space: CartesianColorSpace::Xyz,
        }
    }

    #[must_use]
    pub const fn white(period: f64) -> Self
    {
        let color_map = Sinusoid::new(period);
        Self {
            color_map_r: color_map,
            color_map_g: color_map,
            color_map_b: color_map,
            period_coloring: DiscretePalette::standard(),
            in_color: Color32::BLACK,
            wandering_color: Color32::BROWN,
            unknown_color: Color32::GRAY,
            color_space: CartesianColorSpace::Rgb,
        }
    }

    #[must_use]
    pub const fn black(period: f64) -> Self
    {
        let color_map = Sinusoid {
            period,
            amplitude: 0.5,
            midline: 0.5,
            phase: 0.5,
            degree: 1,
        };
        Self {
            color_map_r: color_map,
            color_map_g: color_map,
            color_map_b: color_map,
            period_coloring: DiscretePalette::standard(),
            in_color: Color32::WHITE,
            wandering_color: Color32::BROWN,
            unknown_color: Color32::GRAY,
            color_space: CartesianColorSpace::Xyz,
        }
    }

    #[must_use]
    pub fn new_random(contrast: f64, brightness: f64) -> Self
    {
        let mut rng = thread_rng();

        let phase_r = Uniform::new(0., 1.).sample(&mut rng);
        let phase_g = Uniform::new(0., 1.).sample(&mut rng);
        let phase_b = Uniform::new(0., 1.).sample(&mut rng);

        ChiSquared::new(7.5).map_or(Self::black(32.), |period_dist| {
            let period_r: f64 = period_dist.sample(&mut rng);
            let period_g: f64 = period_dist.sample(&mut rng);
            let period_b: f64 = period_dist.sample(&mut rng);

            Self::new(period_r, period_g, period_b)
                .with_phases(phase_r, phase_g, phase_b)
                .with_contrast(contrast, brightness)
                .with_degree(2)
        })
    }

    #[must_use]
    pub const fn with_phases(mut self, phase_r: f64, phase_g: f64, phase_b: f64) -> Self
    {
        self.color_map_r.phase = phase_r;
        self.color_map_g.phase = phase_g;
        self.color_map_b.phase = phase_b;
        self
    }

    #[must_use]
    pub const fn with_degree(mut self, degree: i32) -> Self
    {
        self.color_map_r.degree = degree;
        self.color_map_g.degree = degree;
        self.color_map_b.degree = degree;
        self
    }

    #[must_use]
    pub fn with_contrast(mut self, contrast: f64, brightness: f64) -> Self
    {
        let mut amplitude = contrast / 2.0;
        if amplitude > brightness {
            amplitude = brightness;
        } else if amplitude > 1. - brightness {
            amplitude = 1. - brightness;
        }

        self.color_map_r.amplitude = amplitude;
        self.color_map_g.amplitude = amplitude;
        self.color_map_b.amplitude = amplitude;

        self.color_map_r.midline = brightness;
        self.color_map_g.midline = brightness;
        self.color_map_b.midline = brightness;

        self
    }

    #[must_use]
    fn ext_potential_to_sinusoid_input(potential: IterCount) -> IterCount
    {
        (potential - 1.0).log2()
    }

    #[must_use]
    pub fn map<T: FromCartesian>(&self, potential: IterCount) -> T
    {
        let t = Self::ext_potential_to_sinusoid_input(potential);

        let v0 = self.color_map_r.get_value_f64(t) as f32;
        let v1 = self.color_map_g.get_value_f64(t) as f32;
        let v2 = self.color_map_b.get_value_f64(t) as f32;

        match self.color_space {
            CartesianColorSpace::Rgb => RgbLinear {
                r: v0,
                g: v1,
                b: v2,
            }
            .into(),
            CartesianColorSpace::Xyz => Xyz {
                x: v0,
                y: v1,
                z: v2,
            }
            .into(),
        }
    }

    #[must_use]
    pub fn map_color32_phase(
        &self,
        potential: IterCount,
        phase: Period,
        esc_period: Period,
    ) -> Color32
    {
        if esc_period > 1 {
            let t = Self::ext_potential_to_sinusoid_input(potential);

            let r = self.color_map_r.get_value_f64(t) as f32;
            let b = self.color_map_b.get_value_f64(t) as f32;
            DiscretePalette::default()
                .with_num_colors(esc_period as f32)
                // .map_hsv(phase as f32, 1.0)
                // .with_intensity(r)
                // .with_saturation(b)
                .map_lch(phase as f32, 1.0)
                .with_l(r)
                .with_c(b)
                .into()
        } else {
            self.map(potential)
        }
    }

    pub fn scale_period(&mut self, scale_factor: f64)
    {
        *self.color_map_r.get_period_mut() *= scale_factor;
        *self.color_map_g.get_period_mut() *= scale_factor;
        *self.color_map_b.get_period_mut() *= scale_factor;
    }

    pub fn adjust_phase(&mut self, shift: f64)
    {
        *self.color_map_r.get_phase_mut() += shift;
        *self.color_map_g.get_phase_mut() += shift;
        *self.color_map_b.get_phase_mut() += shift;
    }
}

impl Default for Palette
{
    fn default() -> Self
    {
        Self::black(32.)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DiscretePalette
{
    pub num_colors: f32,
    pub base_hue: f32,
    pub saturation: f32,
    pub luminosity: f32,
}

impl DiscretePalette
{
    const DEFAULT_BASE_HUE: f32 = 0.47;
    const DEFAULT_SATURATION: f32 = 0.7;
    const DEFAULT_LUMINOSITY: f32 = 1.;
    const DEFAULT_NUM_COLORS: f32 = 7.;

    #[must_use]
    pub const fn standard() -> Self
    {
        Self {
            num_colors: Self::DEFAULT_NUM_COLORS,
            base_hue: Self::DEFAULT_BASE_HUE,
            saturation: Self::DEFAULT_SATURATION,
            luminosity: Self::DEFAULT_LUMINOSITY,
        }
    }

    #[must_use]
    pub const fn with_num_colors(mut self, num_colors: f32) -> Self
    {
        self.num_colors = num_colors;
        self
    }

    #[must_use]
    fn map_hsv(&self, period: f32, luminosity_modifier: f32) -> Hsv
    {
        let hue = (period / self.num_colors + self.base_hue) % 1.;

        Hsv {
            hue,
            saturation: self.saturation,
            intensity: self.luminosity * luminosity_modifier,
        }
    }

    #[must_use]
    fn map_lch(&self, period: f32, luminosity_modifier: f32) -> Lchab
    {
        let h = (period / self.num_colors + self.base_hue + 0.15) % 1.;

        Lchab {
            h,
            c: self.saturation,
            l: self.luminosity * luminosity_modifier,
        }
    }

    #[must_use]
    fn map_preperiodic_hsv(&self, o: OrbitSchema) -> Hsv
    {
        self.map_hsv(
            o.period as f32,
            0.5f32.mul_add((o.preperiod as f32).tanh(), 1.),
        )
    }

    #[must_use]
    fn map_preperiodic_lch(&self, o: OrbitSchema) -> Lchab
    {
        self.map_lch(
            o.period as f32,
            0.5f32.mul_add((o.preperiod as f32).tanh(), 1.),
        )
    }

    #[must_use]
    pub fn map<T: FromPolar>(&self, period: f32, luminosity_modifier: f32) -> T
    {
        self.map_hsv(period, luminosity_modifier).into()
    }

    #[must_use]
    pub fn map_preperiodic<T: FromPolar>(&self, orbit_schema: OrbitSchema) -> T
    {
        self.map_preperiodic_hsv(orbit_schema).into()
    }

    #[must_use]
    pub const fn black() -> Self
    {
        Self {
            num_colors: 1.,
            base_hue: Self::DEFAULT_BASE_HUE,
            saturation: 1.,
            luminosity: 0.,
        }
    }

    #[must_use]
    pub const fn white() -> Self
    {
        Self {
            num_colors: 1.,
            base_hue: Self::DEFAULT_BASE_HUE,
            saturation: 0.,
            luminosity: 1.,
        }
    }
}

impl Default for DiscretePalette
{
    fn default() -> Self
    {
        Self::standard()
    }
}
