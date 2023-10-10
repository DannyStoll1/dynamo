use egui::{Pos2, Rect, Ui, Vec2};
use epaint::{ColorImage, TextureHandle};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub struct ImageFrame
{
    pub image: ColorImage,
    pub region: Rect,
    texture_id: Option<TextureHandle>,
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
        }
    }
    fn show(&mut self, ui: &mut Ui)
    {
        let texture_id = self.texture_id.get_or_insert_with(|| {
            ui.ctx()
                .load_texture("fractal", self.image.clone(), Default::default())
        });
        ui.image(&*texture_id);
    }
    pub fn height(&self) -> usize
    {
        self.image.height()
    }
    pub fn width(&self) -> usize
    {
        self.image.width()
    }
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
    pub fn to_local_coords(&self, absolute_pos: Pos2) -> Vec2
    {
        absolute_pos - self.region.min
    }
    pub fn to_global_coords(&self, local_pos: Vec2) -> Pos2
    {
        self.region.min + local_pos
    }
    pub fn update_texture(&mut self) {
        if let Some(handle) = self.texture_id.as_mut() {
            handle.set(self.image.clone(), Default::default());
        }
    }
}
