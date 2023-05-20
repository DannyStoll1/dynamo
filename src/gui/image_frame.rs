use eframe::egui::{Pos2, Rect, Ui, Vec2};
use egui_extras::RetainedImage;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub struct ImageFrame
{
    pub image: RetainedImage,
    pub region: Rect,
}
impl ImageFrame
{
    pub fn show(&self, ui: &mut Ui)
    {
        self.image.show(ui);
    }
    pub fn height(&self) -> usize
    {
        self.image.height()
    }
    pub fn width(&self) -> usize
    {
        self.image.height()
    }
    pub fn image_dims(&self) -> Vec2
    {
        Vec2 {
            x: self.image.width() as f32,
            y: self.image.height() as f32,
        }
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
    pub fn to_local_coords(&self, absolute_pos: Pos2) -> Vec2
    {
        absolute_pos - self.region.min
    }
    pub fn to_global_coords(&self, local_pos: Vec2) -> Pos2
    {
        self.region.min + local_pos
    }
}
