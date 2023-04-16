use crate::macros::{max, min};
use crate::primitive_types::IterCount;
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
    #[must_use]
    pub fn new(hue: f32, saturation: f32, luminosity: f32) -> Self {
        Self {
            hue,
            saturation,
            luminosity,
        }
    }
}
impl From<Hsv> for Color32 {
    fn from(val: Hsv) -> Self {
        let c = val.luminosity * val.saturation;
        let mode = val.hue * 6.;
        let x = c * (1. - (mode % 2. - 1.).abs());
        let m = val.luminosity - c;

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

        let range = f32::from(range);
        let c_max_f32 = f32::from(c_max);

        let normalization = 1. / (6. * range);
        let hue = {
            if c_max == r {
                (f32::from(g - b) * normalization) % 1.
            } else if c_max == g {
                f32::from(b - r) * normalization + 1. / 3.
            } else {
                f32::from(r - g) * normalization + 2. / 3.
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

#[derive(Clone, Copy, Debug)]
pub struct Sinusoid {
    period: f64,
    amplitude: f64,
    midline: f64,
    phase: f64,
}
impl Sinusoid {
    fn new(period: f64) -> Self {
        Self {
            period,
            amplitude: 0.5,
            midline: 0.5,
            phase: 0.,
        }
    }
    fn get_value_f64(&self, potential: IterCount) -> f64 {
        let theta = 2.0 * PI * (f64::from(potential) / self.period - self.phase);
        self.amplitude * theta.cos() + self.midline
    }
    fn get_value_u8(&self, potential: IterCount) -> u8 {
        let gamma = self.get_value_f64(potential);
        (256. * gamma) as u8
    }
    fn set_period(&mut self, period: f64) {
        self.period = period
    }
    fn set_amplitude(&mut self, amplitude: f64) {
        self.amplitude = amplitude
    }
    fn set_midline(&mut self, midline: f64) {
        self.midline = midline
    }
    fn set_phase(&mut self, phase: f64) {
        self.phase = phase
    }
}
impl Default for Sinusoid {
    fn default() -> Self {
        Self {
            period: 8.,
            amplitude: 0.5,
            midline: 0.5,
            phase: 0.,
        }
    }
}

#[derive(Clone, Copy)]
pub struct ColorPalette {
    pub color_map_r: Sinusoid,
    pub color_map_g: Sinusoid,
    pub color_map_b: Sinusoid,
    pub period_coloring: DiscretePalette,
    pub in_color: (u8, u8, u8),
}

impl ColorPalette {
    #[must_use]
    pub fn new(period_r: f64, period_g: f64, period_b: f64) -> Self {
        Self {
            color_map_r: Sinusoid::new(period_r),
            color_map_g: Sinusoid::new(period_g),
            color_map_b: Sinusoid::new(period_b),
            period_coloring: DiscretePalette::default(),
            in_color: (0, 0, 0),
        }
    }

    #[must_use]
    pub fn white(period: f64) -> Self {
        let color_map = Sinusoid::new(period);
        Self {
            color_map_r: color_map,
            color_map_g: color_map,
            color_map_b: color_map,
            period_coloring: DiscretePalette::black(),
            in_color: (0, 0, 0),
        }
    }

    #[must_use]
    pub fn black(period: f64) -> Self {
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
            in_color: (255, 255, 255),
        }
    }

    #[must_use]
    pub fn new_random(contrast: f64, brightness: f64) -> Self {
        let mut rng = thread_rng();
        let period_dist = ChiSquared::new(7.5).unwrap();

        let period_r: f64 = period_dist.sample(&mut rng);
        let period_g: f64 = period_dist.sample(&mut rng);
        let period_b: f64 = period_dist.sample(&mut rng);

        Self::new_with_contrast(period_r, period_g, period_b, contrast, brightness)
    }

    #[must_use]
    pub fn new_with_contrast(
        period_r: f64,
        period_g: f64,
        period_b: f64,
        contrast: f64,
        brightness: f64,
    ) -> Self {
        let mut amplitude = contrast / 2.0;
        if amplitude > brightness {
            amplitude = brightness;
        } else if amplitude > 1. - brightness {
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
            in_color: (0, 0, 0),
        }
    }

    #[must_use]
    pub fn map_rgb(&self, value: IterCount) -> Rgb<u8> {
        if value <= 0.0 {
            let (r, g, b) = self.in_color;
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
    pub fn map_color32(&self, value: IterCount) -> Color32 {
        if value < 0.0 {
            let preperiod = -value % 1.;
            let period = -(value + preperiod);
            self.period_coloring.map_color32(period as f32, preperiod as f32)
        } else if value == 0.0 {
            let (r, g, b) = self.in_color;
            Color32::from_rgb(r, g, b)
        } else {
            let potential = (value + 1.0 as IterCount).log2();
            let r = self.color_map_r.get_value_u8(potential);
            let g = self.color_map_g.get_value_u8(potential);
            let b = self.color_map_b.get_value_u8(potential);
            Color32::from_rgb(r, g, b)
        }
    }
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self::new_with_contrast(3.0, 8.0, 5.0, 0.45, 0.38)
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
    const DEFAULT_BASE_HUE: f32 = 0.47;
    const DEFAULT_SATURATION: f32 = 0.7;
    const DEFAULT_LUMINOSITY: f32 = 0.75;
    const DEFAULT_NUM_COLORS: f32 = 7.;

    #[must_use]
    pub fn map_hsv(&self, period: f32, preperiod: f32) -> Hsv {
        let hue = (period / self.num_colors + self.base_hue) % 1.;

        Hsv {
            hue,
            saturation: self.saturation,
            luminosity: self.luminosity * preperiod,
        }
    }
    #[must_use]
    pub fn map_color32(&self, period: f32, preperiod: f32) -> Color32 {
        self.map_hsv(period, preperiod).into()
    }
    #[must_use]
    pub fn black() -> Self {
        Self {
            num_colors: 1.,
            base_hue: Self::DEFAULT_BASE_HUE,
            saturation: 1.,
            luminosity: 0.,
        }
    }
    #[must_use]
    pub fn white() -> Self {
        Self {
            num_colors: 1.,
            base_hue: Self::DEFAULT_BASE_HUE,
            saturation: 0.,
            luminosity: 1.,
        }
    }
}

impl Default for DiscretePalette {
    fn default() -> Self {
        Self {
            num_colors: Self::DEFAULT_NUM_COLORS,
            base_hue: Self::DEFAULT_BASE_HUE,
            saturation: Self::DEFAULT_SATURATION,
            luminosity: Self::DEFAULT_LUMINOSITY,
        }
    }
}
