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

    #[must_use]
    pub const fn with_hue(mut self, hue: f32) -> Self
    {
        self.hue = hue;
        self
    }

    #[must_use]
    pub const fn with_saturation(mut self, saturation: f32) -> Self
    {
        self.saturation = saturation;
        self
    }

    #[must_use]
    pub const fn with_intensity(mut self, intensity: f32) -> Self
    {
        self.intensity = intensity;
        self
    }

    #[allow(clippy::cast_sign_loss)]
    fn as_rgb_tuple_round(&self) -> (u8, u8, u8)
    {
        let chr = self.saturation;
        let hue = self.hue * TAU;
        let itn = self.intensity;

        let red = 127.5 * itn * chr.mul_add(hue.cos(), 1.);
        let grn = 127.5 * itn * chr.mul_add((hue - TAU_3).cos(), 1.);
        let blu = 127.5 * itn * chr.mul_add((hue + TAU_3).cos(), 1.);

        (red as u8, grn as u8, blu as u8)
    }

    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::many_single_char_names)]
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
        let r = f32::from(rgb.0) / f32::from(u8::MAX - 1);
        let g = f32::from(rgb.1) / f32::from(u8::MAX - 1);
        let b = f32::from(rgb.2) / f32::from(u8::MAX - 1);

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

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Lchuv
{
    pub l: f32,
    pub c: f32,
    pub h: f32,
}

impl Lchuv
{
    #[must_use]
    pub const fn with_l(mut self, l: f32) -> Self
    {
        self.l = l;
        self
    }
    #[must_use]
    pub const fn with_c(mut self, c: f32) -> Self
    {
        self.c = c;
        self
    }
    #[must_use]
    pub const fn with_h(mut self, h: f32) -> Self
    {
        self.h = h;
        self
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Lchab
{
    pub l: f32,
    pub c: f32,
    pub h: f32,
}

impl Lchab
{
    #[must_use]
    pub const fn with_l(mut self, l: f32) -> Self
    {
        self.l = l;
        self
    }
    #[must_use]
    pub const fn with_c(mut self, c: f32) -> Self
    {
        self.c = c;
        self
    }
    #[must_use]
    pub const fn with_h(mut self, h: f32) -> Self
    {
        self.h = h;
        self
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Luv
{
    pub l: f32,
    pub u: f32,
    pub v: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Lab
{
    pub l: f32,
    pub a: f32,
    pub b: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Xyz
{
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Xyz
{
    const REF_WHITE: Self = Self {
        x: 0.95047,
        y: 1.,
        z: 1.08883,
    };
    const WHITE_U0: f32 = 0.197_833_03;
    const WHITE_V0: f32 = 0.468_330_47;
    const KAPPA: f32 = 903.3;
    const EPS: f32 = 0.008_856;

    const FROM_RGB_MATRIX: [[f32; 3]; 3] = [
        [0.49, 0.31, 0.2],
        [0.17697, 0.81240, 0.01063],
        [0.0, 0.01, 0.99],
    ];

    const TO_RGB_MATRIX: [[f32; 3]; 3] = [
        [2.364_613_8, -0.896_540_6, -0.468_076_48],
        [-0.515_166_2, 1.426_408, 0.088_758_1],
        [0.005_203_7, -0.014_408_16, 1.009_204_5],
    ];
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RgbLinear
{
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl RgbLinear
{
    const GAMMA: f32 = 1.;

    #[allow(clippy::cast_sign_loss)]
    fn gamma_map(val: f32) -> u8
    {
        (val * 256.) as u8
    }
    fn to_u8(self) -> [u8; 3]
    {
        [
            Self::gamma_map(self.r),
            Self::gamma_map(self.g),
            Self::gamma_map(self.b),
        ]
    }
}

impl From<Lchuv> for Luv
{
    fn from(Lchuv { l, c, h }: Lchuv) -> Self
    {
        Self {
            l,
            u: c * (h * TAU).cos(),
            v: c * (h * TAU).sin(),
        }
    }
}

impl From<Lchab> for Lab
{
    fn from(Lchab { l, c, h }: Lchab) -> Self
    {
        Self {
            l,
            a: c * (h * TAU).cos(),
            b: c * (h * TAU).sin(),
        }
    }
}

impl From<Luv> for Xyz
{
    #[allow(clippy::many_single_char_names)]
    fn from(Luv { l, u, v }: Luv) -> Self
    {
        let Self { y: yr, .. } = Self::REF_WHITE;

        let l = l * 100.;
        let u = u * 100.;
        let v = v * 100.;

        let y = if l > 8. {
            ((l + 16.) / 116.).powi(3) * yr
        } else {
            l / Self::KAPPA * yr
        };

        let u0 = u / (13. * l) + Self::WHITE_U0;
        let v0 = v / (13. * l) + Self::WHITE_V0;

        let x = y * 2.25 * u0 / v0;
        let z = y * (u0.mul_add(-3.0, 12.0) / (4. * v0) - 5.);
        Self { x, y, z }
    }
}

impl From<Lab> for Xyz
{
    #[allow(clippy::many_single_char_names)]
    fn from(Lab { l, a, b }: Lab) -> Self
    {
        let Self {
            x: xr,
            y: yr,
            z: zr,
        } = Self::REF_WHITE;

        let l = l * 100.;

        let fy = (l + 16.) / 116.;
        let fx = a / 5. + fy;
        let fz = fy - b / 2.;

        let mut x0 = fx.powi(3);
        if x0 <= Self::EPS {
            x0 = fx.mul_add(116., -16.) / Self::KAPPA;
        }

        let mut z0 = fz.powi(3);
        if z0 <= Self::EPS {
            z0 = fz.mul_add(116., -16.) / Self::KAPPA;
        }

        let y0 = if l > 8. {
            ((l + 16.) / 116.).powi(3)
        } else {
            l / Self::KAPPA
        };

        Self {
            x: x0 * xr,
            y: y0 * yr,
            z: z0 * zr,
        }
    }
}

fn dot<const N: usize>(v: [f32; N], w: [f32; N]) -> f32
{
    v.into_iter().zip(w).map(|(v, w)| v * w).sum()
}

impl From<Xyz> for RgbLinear
{
    #[allow(clippy::many_single_char_names)]
    fn from(Xyz { x, y, z }: Xyz) -> Self
    {
        Self {
            r: dot(Xyz::TO_RGB_MATRIX[0], [x, y, z]),
            g: dot(Xyz::TO_RGB_MATRIX[1], [x, y, z]),
            b: dot(Xyz::TO_RGB_MATRIX[2], [x, y, z]),
        }
    }
}

impl From<RgbLinear> for Color32
{
    fn from(rgb: RgbLinear) -> Self
    {
        let [r, g, b] = rgb.to_u8();
        Self::from_rgb(r, g, b)
    }
}

impl From<Xyz> for Color32
{
    fn from(xyz: Xyz) -> Self
    {
        Self::from(RgbLinear::from(xyz))
    }
}

impl From<RgbLinear> for Rgb<u8>
{
    fn from(rgb: RgbLinear) -> Self
    {
        Self(rgb.to_u8())
    }
}

impl From<Xyz> for Rgb<u8>
{
    fn from(xyz: Xyz) -> Self
    {
        Self::from(RgbLinear::from(xyz))
    }
}

impl From<Lchuv> for Xyz
{
    fn from(lch: Lchuv) -> Self
    {
        Self::from(Luv::from(lch))
    }
}

impl From<Lchab> for Xyz
{
    fn from(lch: Lchab) -> Self
    {
        Self::from(Lab::from(lch))
    }
}

impl From<Lchuv> for Rgb<u8>
{
    fn from(lch: Lchuv) -> Self
    {
        Self::from(Xyz::from(lch))
    }
}

impl From<Lchab> for Rgb<u8>
{
    fn from(lch: Lchab) -> Self
    {
        Self::from(Xyz::from(Lab::from(lch)))
    }
}

impl From<Lchuv> for Color32
{
    fn from(lch: Lchuv) -> Self
    {
        Self::from(Xyz::from(lch))
    }
}

impl From<Lchab> for Color32
{
    fn from(lch: Lchab) -> Self
    {
        Self::from(Xyz::from(lch))
    }
}

pub trait FromColor32
{
    fn from_color32(color32: Color32) -> Self;
}
impl FromColor32 for Color32
{
    #[inline]
    fn from_color32(color32: Color32) -> Self
    {
        color32
    }
}
impl FromColor32 for Rgb<u8>
{
    #[inline]
    fn from_color32(color32: Color32) -> Self
    {
        let [r, g, b, _a] = color32.to_array();
        Self([r, g, b])
    }
}
impl FromColor32 for Hsv
{
    #[inline]
    fn from_color32(color32: Color32) -> Self
    {
        Self::from(color32)
    }
}

pub trait FromCartesian: From<RgbLinear> + From<Xyz> {}
pub trait FromPolar: From<Hsv> + From<Lchab> + From<Lchuv> {}
pub trait FromColor: FromPolar + FromCartesian + FromColor32 {}

impl<T> FromCartesian for T where T: From<RgbLinear> + From<Xyz> {}
impl<T> FromPolar for T where T: From<Hsv> + From<Lchuv> + From<Lchab> {}
impl<T> FromColor for T where T: FromCartesian + FromPolar + FromColor32 {}
