use std::f32::consts::TAU;

use egui::Color32;
use image::{Pixel, Rgb};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

const SIN_60: f32 = 0.866_025_4;
const TAU_3: f32 = std::f32::consts::TAU / 3.;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Hsv
{
    pub hue: f32,
    pub saturation: f32,
    pub intensity: f32,
}
impl Hsv
{
    const WHITE: Self = Self {
        hue: 0.,
        saturation: 0.,
        intensity: 1.,
    };

    #[must_use]
    pub const fn new(hue: f32, saturation: f32, intensity: f32) -> Self
    {
        Self {
            hue,
            saturation,
            intensity,
        }
    }

    pub const fn with_hue(mut self, hue: f32) -> Self {
        self.hue = hue;
        self
    }

    pub const fn with_saturation(mut self, saturation: f32) -> Self {
        self.saturation = saturation;
        self
    }

    pub const fn with_intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity;
        self
    }

    fn as_rgb_tuple_round(&self) -> (u8, u8, u8)
    {
        let c = self.saturation;
        let h = self.hue * TAU;
        let i = self.intensity;

        let r = 127.5 * i * c.mul_add(h.cos(), 1.);
        let g = 127.5 * i * c.mul_add((h - TAU_3).cos(), 1.);
        let b = 127.5 * i * c.mul_add((h + TAU_3).cos(), 1.);

        (r as u8, g as u8, b as u8)
    }

    fn as_rgb_tuple(&self) -> (u8, u8, u8)
    {
        let c = self.intensity * self.saturation;
        let mode = self.hue * 6.;
        let x = c * (1. - (mode % 2. - 1.).abs());
        let m = self.intensity - c;

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
        (r, g, b)
    }

    fn from_rgb_tuple_round(rgb: (u8, u8, u8)) -> Self
    {
        let r = rgb.0 as f32 / f32::from(u8::MAX - 1);
        let g = rgb.1 as f32 / f32::from(u8::MAX - 1);
        let b = rgb.2 as f32 / f32::from(u8::MAX - 1);

        let alpha = r - (g + b) / 2.;
        let beta = SIN_60 * (g - b);

        let hue = beta.atan2(alpha) / TAU;
        let chroma = alpha.hypot(beta);
        // let intensity = (r.powi(2) + g.powi(2) + b.powi(2)).sqrt();
        let intensity = Rgb([r, g, b]).to_luma().0[0];

        Self {
            hue,
            saturation: chroma / intensity,
            intensity,
        }
    }

    fn from_rgb_tuple((r, g, b): (u8, u8, u8)) -> Self
    {
        let c_max = r.max(g).max(b);
        let c_min = r.min(g).min(b);
        let range = c_max - c_min;

        if range == 0 {
            return Self {
                hue: 0.,
                saturation: 0.,
                intensity: 0.,
            };
        }

        let range = f32::from(range);
        let c_max_f32 = f32::from(c_max);

        let normalization = 1. / (6. * range);
        let hue = {
            if c_max == r {
                (f32::from(g - b) * normalization) % 1.
            } else if c_max == g {
                f32::from(b - r).mul_add(normalization, 1. / 3.)
            } else {
                f32::from(r - g).mul_add(normalization, 2. / 3.)
            }
        };

        let saturation = range / c_max_f32;
        let luminosity = c_max_f32;

        Self {
            hue,
            saturation,
            intensity: luminosity,
        }
    }
}
impl From<Hsv> for Color32
{
    fn from(val: Hsv) -> Self
    {
        let (r, g, b) = val.as_rgb_tuple();
        Self::from_rgb(r, g, b)
    }
}
impl From<Hsv> for Rgb<u8>
{
    fn from(val: Hsv) -> Self
    {
        let rgb = val.as_rgb_tuple();
        Self(rgb.into())
    }
}
impl From<Color32> for Hsv
{
    fn from(color32: Color32) -> Self
    {
        let r = color32.r();
        let g = color32.g();
        let b = color32.b();

        Self::from_rgb_tuple((r, g, b))
    }
}
impl From<(u8, u8, u8)> for Hsv
{
    fn from(rgb: (u8, u8, u8)) -> Self
    {
        Self::from_rgb_tuple(rgb)
    }
}
impl From<Rgb<u8>> for Hsv
{
    fn from(rgb: Rgb<u8>) -> Self
    {
        Self::from_rgb_tuple(rgb.0.into())
    }
}
