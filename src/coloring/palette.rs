use crate::types::IterCount;
use super::Hsv;
use eframe::egui::Color32;
use image::Rgb;
use rand::prelude::*;
use rand_distr::{ChiSquared, Distribution};
use std::f64::consts::PI;


#[derive(Clone, Copy, Debug)]
pub struct Sinusoid
{
    period: f64,
    phase: f64,
    amplitude: f64,
    midline: f64,
}
impl Sinusoid
{
    fn new(period: f64) -> Self
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
        self.amplitude * theta.cos() + self.midline
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
        self.period = period
    }
    fn set_amplitude(&mut self, amplitude: f64)
    {
        self.amplitude = amplitude
    }
    fn set_midline(&mut self, midline: f64)
    {
        self.midline = midline
    }
    fn set_phase(&mut self, phase: f64)
    {
        self.phase = phase
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

#[derive(Clone, Copy)]
pub struct ColorPalette
{
    pub color_map_r: Sinusoid,
    pub color_map_g: Sinusoid,
    pub color_map_b: Sinusoid,
    pub period_coloring: DiscretePalette,
    pub in_color: Color32,
}

impl ColorPalette
{
    #[must_use]
    pub fn new(period_r: f64, period_g: f64, period_b: f64) -> Self
    {
        Self {
            color_map_r: Sinusoid::new(period_r),
            color_map_g: Sinusoid::new(period_g),
            color_map_b: Sinusoid::new(period_b),
            period_coloring: DiscretePalette::default(),
            in_color: Color32::BLACK,
        }
    }

    #[must_use]
    pub fn white(period: f64) -> Self
    {
        let color_map = Sinusoid::new(period);
        Self {
            color_map_r: color_map,
            color_map_g: color_map,
            color_map_b: color_map,
            period_coloring: DiscretePalette::black(),
            in_color: Color32::BLACK,
        }
    }

    #[must_use]
    pub fn black(period: f64) -> Self
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
            period_coloring: DiscretePalette::white(),
            in_color: Color32::WHITE,
        }
    }

    #[must_use]
    pub fn new_random(contrast: f64, brightness: f64) -> Self
    {
        let mut rng = thread_rng();
        if let Ok(period_dist) = ChiSquared::new(7.5)
        {
            let period_r: f64 = period_dist.sample(&mut rng);
            let period_g: f64 = period_dist.sample(&mut rng);
            let period_b: f64 = period_dist.sample(&mut rng);

            Self::new_with_contrast(period_r, period_g, period_b, contrast, brightness)
        }
        else
        {
            Self::black(8.)
        }
    }

    #[must_use]
    pub fn new_with_contrast(
        period_r: f64,
        period_g: f64,
        period_b: f64,
        contrast: f64,
        brightness: f64,
    ) -> Self
    {
        let mut amplitude = contrast / 2.0;
        if amplitude > brightness
        {
            amplitude = brightness;
        }
        else if amplitude > 1. - brightness
        {
            amplitude = 1. - brightness;
        }

        let palette_r = Sinusoid {
            period: period_r,
            amplitude,
            midline: brightness,
            phase: 0.,
        };
        let palette_g = Sinusoid {
            period: period_g,
            amplitude,
            midline: brightness,
            phase: 0.,
        };
        let palette_b = Sinusoid {
            period: period_b,
            amplitude,
            midline: brightness,
            phase: 0.,
        };

        Self {
            color_map_r: palette_r,
            color_map_g: palette_g,
            color_map_b: palette_b,
            period_coloring: DiscretePalette::default(),
            in_color: Color32::BLACK,
        }
    }

    #[must_use]
    pub fn map_rgb(&self, value: IterCount) -> Rgb<u8>
    {
        if value <= 0.0
        {
            let (r, g, b, _) = self.in_color.to_tuple();
            Rgb([r, g, b])
        }
        else
        {
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
        if value <= 0.0
        {
            self.in_color
        }
        else
        {
            let potential = (value + 1.0 as IterCount).log2();
            let r = self.color_map_r.get_value_u8(potential);
            let g = self.color_map_g.get_value_u8(potential);
            let b = self.color_map_b.get_value_u8(potential);
            Color32::from_rgb(r, g, b)
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

impl Default for ColorPalette
{
    fn default() -> Self
    {
        Self::new_with_contrast(3.0, 8.0, 5.0, 0.45, 0.38)
    }
}

#[derive(Clone, Copy)]
pub struct DiscretePalette
{
    num_colors: f32,
    base_hue: f32,
    saturation: f32,
    luminosity: f32,
}

impl DiscretePalette
{
    const DEFAULT_BASE_HUE: f32 = 0.47;
    const DEFAULT_SATURATION: f32 = 0.7;
    const DEFAULT_LUMINOSITY: f32 = 0.75;
    const DEFAULT_NUM_COLORS: f32 = 7.;

    #[must_use]
    pub fn map_hsv(&self, period: f32, preperiod: f32) -> Hsv
    {
        let hue = (period / self.num_colors + self.base_hue) % 1.;

        Hsv {
            hue,
            saturation: self.saturation,
            luminosity: self.luminosity * preperiod,
        }
    }
    #[must_use]
    pub fn map_rgb(&self, period: f32, preperiod: f32) -> Rgb<u8>
    {
        self.map_hsv(period, preperiod).into()
    }
    #[must_use]
    pub fn map_color32(&self, period: f32, preperiod: f32) -> Color32
    {
        self.map_hsv(period, preperiod).into()
    }
    #[must_use]
    pub fn black() -> Self
    {
        Self {
            num_colors: 1.,
            base_hue: Self::DEFAULT_BASE_HUE,
            saturation: 1.,
            luminosity: 0.,
        }
    }
    #[must_use]
    pub fn white() -> Self
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
        Self {
            num_colors: Self::DEFAULT_NUM_COLORS,
            base_hue: Self::DEFAULT_BASE_HUE,
            saturation: Self::DEFAULT_SATURATION,
            luminosity: Self::DEFAULT_LUMINOSITY,
        }
    }
}
