use crate::macros::{max, min};
use egui::Color32;
use image::Rgb;

#[derive(Clone, Copy, Debug)]
pub struct Hsv
{
    pub hue: f32,
    pub saturation: f32,
    pub luminosity: f32,
}
impl Hsv
{
    #[must_use]
    pub const fn new(hue: f32, saturation: f32, luminosity: f32) -> Self
    {
        Self {
            hue,
            saturation,
            luminosity,
        }
    }

    fn as_rgb_tuple(&self) -> (u8, u8, u8)
    {
        let c = self.luminosity * self.saturation;
        let mode = self.hue * 6.;
        let x = c * (1. - (mode % 2. - 1.).abs());
        let m = self.luminosity - c;

        let (r_, g_, b_) = match (mode as i32) % 6
        {
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

    fn from_rgb_tuple(r: u8, g: u8, b: u8) -> Self
    {
        let c_max = max!(r, g, b);
        let c_min = min!(r, g, b);
        let range = c_max - c_min;

        if range == 0
        {
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
            if c_max == r
            {
                (f32::from(g - b) * normalization) % 1.
            }
            else if c_max == g
            {
                f32::from(b - r).mul_add(normalization, 1. / 3.)
            }
            else
            {
                f32::from(r - g).mul_add(normalization, 2. / 3.)
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
        let (r, g, b) = val.as_rgb_tuple();
        Self([r, g, b])
    }
}
impl From<Color32> for Hsv
{
    fn from(color32: Color32) -> Self
    {
        let r = color32.r();
        let g = color32.g();
        let b = color32.b();

        Self::from_rgb_tuple(r, g, b)
    }
}
impl From<(u8, u8, u8)> for Hsv
{
    fn from(rgb: (u8, u8, u8)) -> Self
    {
        let (r, g, b) = rgb;
        Self::from_rgb_tuple(r, g, b)
    }
}
