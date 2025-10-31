use egui::containers::Frame;
use egui::{Pos2, Rect, TextureOptions, Ui, Vec2};
use epaint::{ColorImage, Stroke, TextureHandle};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BorderMode
{
    selected: bool,
    live:     bool,
}
impl BorderMode
{
    pub(super) const fn stroke(&self) -> Stroke
    {
        use crate::colors::{INACTIVE, LIVE, SELECTED};
        if self.live {
            return Stroke {
                color: LIVE,
                width: 2.,
            };
        }
        if self.selected {
            Stroke {
                color: SELECTED,
                width: 2.,
            }
        } else {
            Stroke {
                color: INACTIVE,
                width: 2.,
            }
        }
    }
}

pub struct ImageFrame
{
    pub image:  ColorImage,
    pub region: Rect,
    texture_id: Option<TextureHandle>,
    border:     BorderMode,
}
impl Default for ImageFrame
{
    fn default() -> Self
    {
        Self {
            image:      ColorImage::default(),
            region:     Rect::NOTHING,
            texture_id: None,
            border:     BorderMode::default(),
        }
    }
}
impl ImageFrame
{
    #[must_use]
    pub fn new(image: ColorImage) -> Self
    {
        Self {
            image,
            region: Rect::NOTHING,
            texture_id: None,
            border: BorderMode::default(),
        }
    }
    fn show(&mut self, ui: &mut Ui)
    {
        let texture_id = self.texture_id.get_or_insert_with(|| {
            ui.ctx()
                .load_texture("fractal", self.image.clone(), TextureOptions::default())
        });

        Frame::new().stroke(self.border.stroke()).show(ui, |ui| {
            ui.image(&*texture_id);
        });
    }
    pub const fn select(&mut self)
    {
        self.border.selected = true;
    }
    pub const fn deselect(&mut self)
    {
        self.border.selected = false;
    }
    pub const fn set_live(&mut self)
    {
        self.border.live = true;
    }
    pub const fn unset_live(&mut self)
    {
        self.border.live = false;
    }
    #[must_use]
    pub fn height(&self) -> usize
    {
        self.image.height()
    }
    #[must_use]
    pub fn width(&self) -> usize
    {
        self.image.width()
    }
    #[must_use]
    pub fn image_dims(&self) -> Vec2
    {
        let [x, y] = self.image.size;
        Vec2::from([x as f32, y as f32])
    }
    pub fn set_position(&mut self, anchor: Pos2)
    {
        self.region.min = anchor;
        self.region.max = anchor + self.image_dims();
    }
    pub fn put(&mut self, ui: &mut Ui)
    {
        let anchor = ui.cursor().min;
        self.set_position(anchor);
        self.show(ui);
    }
    #[must_use]
    pub fn to_local_coords(&self, absolute_pos: Pos2) -> Vec2
    {
        absolute_pos - self.region.min
    }
    #[must_use]
    pub fn to_global_coords(&self, local_pos: Vec2) -> Pos2
    {
        self.region.min + local_pos
    }
    pub fn update_texture(&mut self)
    {
        if let Some(handle) = self.texture_id.as_mut() {
            handle.set(self.image.clone(), TextureOptions::default());
        }
    }
}
