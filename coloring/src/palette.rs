use super::Hsv;
use dynamo_common::symbolic_dynamics::OrbitSchema;
use dynamo_common::types::IterCount;
use egui::Color32;
use image::Rgb;
use rand::prelude::*;
use rand_distr::{ChiSquared, Distribution, Uniform};
use std::f64::consts::PI;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Sinusoid
{
    period: f64,
    phase: f64,
    amplitude: f64,
    midline: f64,
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
        }
    }
    fn get_value_f64(&self, potential: IterCount) -> f64
    {
        let theta = 2.0 * PI * (potential / self.period - self.phase);
        self.amplitude.mul_add(theta.cos(), self.midline)
    }
    fn get_value_u8(&self, potential: IterCount) -> u8
    {
        let gamma = self.get_value_f64(potential);
        (256. * gamma) as u8
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
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Palette
{
    pub color_map_r: Sinusoid,
    pub color_map_g: Sinusoid,
    pub color_map_b: Sinusoid,
    pub period_coloring: DiscretePalette,
    pub in_color: Color32,
    pub wandering_color: Color32,
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
        };
        Self {
            color_map_r: color_map,
            color_map_g: color_map,
            color_map_b: color_map,
            period_coloring: DiscretePalette::standard(),
            in_color: Color32::WHITE,
            wandering_color: Color32::BROWN,
        }
    }

    #[must_use]
    pub fn new_random(contrast: f64, brightness: f64) -> Self
    {
        let mut rng = thread_rng();

        let phase_r = Uniform::new(0., 1.).sample(&mut rng);
        let phase_g = Uniform::new(0., 1.).sample(&mut rng);
        let phase_b = Uniform::new(0., 1.).sample(&mut rng);

        ChiSquared::new(7.5).map_or(Self::black(8.), |period_dist| {
            let period_r: f64 = period_dist.sample(&mut rng);
            let period_g: f64 = period_dist.sample(&mut rng);
            let period_b: f64 = period_dist.sample(&mut rng);

            Self::new(period_r, period_g, period_b)
                .with_phases(phase_r, phase_g, phase_b)
                .with_contrast(contrast, brightness)
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
    pub fn map_rgb(&self, value: IterCount) -> Rgb<u8>
    {
        if value <= 0.0 {
            let (r, g, b, _) = self.in_color.to_tuple();
            Rgb([r, g, b])
        } else {
            let potential = (value + 1.0 as IterCount).log2();
            let r = self.color_map_r.get_value_u8(potential);
            let g = self.color_map_g.get_value_u8(potential);
            let b = self.color_map_b.get_value_u8(potential);
            Rgb([r, g, b])
        }
    }

    #[must_use]
    pub fn map_color32(&self, value: IterCount) -> Color32
    {
        let potential = (value + 1.0 as IterCount).log2();

        let r = self.color_map_r.get_value_u8(potential);
        let g = self.color_map_g.get_value_u8(potential);
        let b = self.color_map_b.get_value_u8(potential);
        Color32::from_rgb(r, g, b)
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
        // Self::new_with_contrast(3.0, 8.0, 5.0, 0.45, 0.38)
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
    fn map_preperiodic_hsv(&self, o: OrbitSchema) -> Hsv
    {
        self.map_hsv(
            o.period as f32,
            0.5f32.mul_add((o.preperiod as f32).tanh(), 1.),
        )
    }

    #[must_use]
    pub fn map<T: From<Hsv>>(&self, period: f32, luminosity_modifier: f32) -> T
    {
        self.map_hsv(period, luminosity_modifier).into()
    }

    #[must_use]
    pub fn map_preperiodic<T: From<Hsv>>(&self, orbit_schema: OrbitSchema) -> T
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
