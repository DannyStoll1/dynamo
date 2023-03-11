use crate::macros::*;
use eframe::egui::Color32;
use image::Rgb;
use rand::prelude::*;
use rand_distr::{ChiSquared, Distribution};
use std::f64::consts::PI;

#[derive(Clone, Copy, Debug)]
pub struct Hsv {
    hue: f32,
    saturation: f32,
    luminosity: f32,
}
impl Hsv {
    pub fn new(hue: f32, saturation: f32, luminosity: f32) -> Self {
        Self {
            hue,
            saturation,
            luminosity,
        }
    }
}
impl Into<Color32> for Hsv {
    fn into(self) -> Color32 {
        let c = self.luminosity * self.saturation;
        let mode = self.hue * 6.;
        let x = c * (1. - (mode % 2. - 1.).abs());
        let m = self.luminosity - c;

        let (r_, g_, b_) = match (mode as i32) % 6 {
            0 => (c, x, 0.),
            1 => (x, c, 0.),
            2 => (0., c, x),
            3 => (0., x, c),
            4 => (x, 0., c),
            5 => (c, 0., x),
            _ => (0., 0., 0.),
        };
        let r = (256. * (r_ + m)) as u8;
        let g = (256. * (g_ + m)) as u8;
        let b = (256. * (b_ + m)) as u8;
        Color32::from_rgb(r, g, b)
    }
}
impl From<Color32> for Hsv {
    fn from(color32: Color32) -> Self {
        let r = color32.r();
        let g = color32.g();
        let b = color32.b();

        let c_max = max!(r, g, b);
        let c_min = min!(r, g, b);
        let range = c_max - c_min;

        if range == 0 {
            return Self {
                hue: 0.,
                saturation: 0.,
                luminosity: 0.,
            };
        }

        let range = range as f32;
        let c_max_f32 = c_max as f32;

        let normalization = 1. / (6. * range);
        let hue = {
            if c_max == r {
                ((g - b) as f32 * normalization) % 1.
            } else if c_max == g {
                (b - r) as f32 * normalization + 1. / 3.
            } else {
                (r - g) as f32 * normalization + 2. / 3.
            }
        };

        let saturation = range / c_max_f32;
        let luminosity = c_max_f32;

        Self {
            hue,
            saturation,
            luminosity,
        }
    }
}

#[derive(Clone, Copy)]
pub struct ColorPalette {
    period_r: f64,
    period_g: f64,
    period_b: f64,
    amplitude_r: f64,
    amplitude_g: f64,
    amplitude_b: f64,
    midline_r: f64,
    midline_g: f64,
    midline_b: f64,
    period_coloring: DiscretePalette,
}

impl ColorPalette {
    pub fn new(period_r: f64, period_g: f64, period_b: f64) -> Self {
        Self {
            period_r,
            period_g,
            period_b,
            amplitude_r: 0.5,
            amplitude_g: 0.5,
            amplitude_b: 0.5,
            midline_r: 0.5,
            midline_g: 0.5,
            midline_b: 0.5,
            period_coloring: DiscretePalette::default(),
        }
    }

    pub fn new_random(contrast: f64, brightness: f64) -> Self {
        let mut rng = thread_rng();
        let period_dist = ChiSquared::new(7.5).unwrap();

        let period_r: f64 = period_dist.sample(&mut rng);
        let period_g: f64 = period_dist.sample(&mut rng);
        let period_b: f64 = period_dist.sample(&mut rng);

        Self::new_with_contrast(period_r, period_g, period_b, contrast, brightness)
    }

    pub fn new_with_contrast(
        period_r: f64,
        period_g: f64,
        period_b: f64,
        contrast: f64,
        brightness: f64,
    ) -> Self {
        let _midline = brightness / 2.0;
        let mut amplitude = contrast / 2.0;
        if amplitude > brightness {
            amplitude = brightness;
        } else if amplitude > 1. - brightness {
            amplitude = 1. - brightness;
        }

        Self {
            period_r,
            period_g,
            period_b,
            amplitude_r: amplitude,
            amplitude_g: amplitude,
            amplitude_b: amplitude,
            midline_r: brightness,
            midline_g: brightness,
            midline_b: brightness,
            period_coloring: DiscretePalette::default(),
        }
    }

    pub fn map_rgb(&self, value: f64) -> Rgb<u8> {
        if value <= 0.0 {
            Rgb([0, 0, 0])
        } else {
            let potential = (value + 1.0_f64).log2();
            let r = Self::to_value(potential, self.period_r, self.amplitude_r, self.midline_r);
            let g = Self::to_value(potential, self.period_g, self.amplitude_g, self.midline_g);
            let b = Self::to_value(potential, self.period_b, self.amplitude_b, self.midline_b);
            Rgb([r, g, b])
        }
    }

    pub fn map_color32(&self, value: f64) -> Color32 {
        if value < 0.0 {
            self.period_coloring.map_color32(-value as f32)
        } else if value == 0.0 {
            Color32::from_rgb(0, 0, 0)
        } else {
            let potential = (value + 1.0_f64).log2();
            let r = Self::to_value(potential, self.period_r, self.amplitude_r, self.midline_r);
            let g = Self::to_value(potential, self.period_g, self.amplitude_g, self.midline_g);
            let b = Self::to_value(potential, self.period_b, self.amplitude_b, self.midline_b);
            Color32::from_rgb(r, g, b)
        }
    }

    pub fn to_value(potential: f64, period: f64, amplitude: f64, midline: f64) -> u8 {
        let theta = 2.0 * PI * (potential / period);
        let value = amplitude * theta.cos() + midline;
        (256.0 * value) as u8
    }
}

#[derive(Clone, Copy)]
pub struct DiscretePalette {
    num_colors: f32,
    base_hue: f32,
    saturation: f32,
    luminosity: f32,
}

impl DiscretePalette {
    pub fn map_hsv(&self, value: f32) -> Hsv {
        let hue = (value / self.num_colors + self.base_hue) % 1.;

        Hsv {
            hue,
            saturation: self.saturation,
            luminosity: self.luminosity,
        }
    }
    pub fn map_color32(&self, value: f32) -> Color32 {
        self.map_hsv(value).into()
    }
}

impl Default for DiscretePalette {
    fn default() -> Self {
        Self {
            num_colors: 8.,
            base_hue: 0.,
            saturation: 0.6,
            luminosity: 0.6,
        }
    }
}
