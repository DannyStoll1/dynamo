use image::Rgb;
use std::f64::consts::PI;

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
}

impl ColorPalette {
    pub fn new(period_r: f64, period_g: f64, period_b: f64) -> Self {
        Self {
            period_r: period_r,
            period_g: period_g,
            period_b: period_b,
            amplitude_r: 0.5,
            amplitude_g: 0.5,
            amplitude_b: 0.5,
            midline_r: 0.5,
            midline_g: 0.5,
            midline_b: 0.5,
        }
    }

    pub fn color_map(&self, value: f64) -> Rgb<u16> {
        if value < 0.0 {
            return Rgb([0, 0, 0]);
        } else {
            let potential = (value + 1.0_f64).log2();
            let r = Self::to_value(potential, self.period_r, self.amplitude_r, self.midline_r);
            let g = Self::to_value(potential, self.period_g, self.amplitude_g, self.midline_g);
            let b = Self::to_value(potential, self.period_b, self.amplitude_b, self.midline_b);
            return Rgb([r, g, b]);
        }
    }

    pub fn to_value(potential: f64, period: f64, amplitude: f64, midline: f64) -> u16 {
        let theta = 2.0 * PI * (potential / period);
        let value = amplitude * theta.cos() + midline;
        (65536.0 * value) as u16
    }
}
