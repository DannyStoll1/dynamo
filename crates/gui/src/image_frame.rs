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
        // Skip an empty image: egui cannot create a zero-dimension texture.
        if self.image.width() == 0 || self.image.height() == 0 {
            return;
        }
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
    /// Ensure the backing image is exactly `[width, height]`, reallocating with
    /// a neutral fill if the size changed. A zero dimension is ignored, so a
    /// degenerate (for example NaN-bounded) grid never blanks a valid buffer.
    pub fn resize(&mut self, width: usize, height: usize)
    {
        if width == 0 || height == 0 {
            return;
        }
        if self.image.size != [width, height] {
            self.image = ColorImage::filled([width, height], egui::Color32::default());
        }
    }

    /// Blit a streamed compute tile into the backing image.
    ///
    /// The tile holds colors at its mip level (array order, rows bottom to top).
    /// The display buffer is sized to the tile's target (finest) resolution, so
    /// it stays constant across mip levels. Each level pixel expands to a
    /// `scale x scale` block, clamped to the buffer, and rows are flipped
    /// vertically so image row 0 is the top while array row 0 is the bottom.
    pub fn blit_tile(&mut self, tile: &crate::compute::Tile)
    {
        let (full_w, full_h) = tile.target_res;
        self.resize(full_w, full_h);
        if self.image.size != [full_w, full_h] {
            return;
        }

        let (_level_x, level_y) = tile.level_res;
        let tile_w = tile.x_range.1 - tile.x_range.0;
        for (idx, color) in tile.pixels.iter().enumerate() {
            let lx = tile.x_range.0 + idx % tile_w;
            let ly = tile.y_range.0 + idx / tile_w;
            // Flip vertically: level row `ly` maps to level row `level_y-1-ly`,
            // then scales into the target buffer.
            let flipped_y = level_y - 1 - ly;
            for dy in 0..tile.scale {
                let py = flipped_y * tile.scale + dy;
                if py >= full_h {
                    break;
                }
                let row = py * full_w;
                for dx in 0..tile.scale {
                    let px = lx * tile.scale + dx;
                    if px >= full_w {
                        break;
                    }
                    self.image.pixels[row + px] = *color;
                }
            }
        }
    }

    pub fn update_texture(&mut self)
    {
        // Never push a zero-dimension image: egui/wgpu reject it.
        if self.image.width() == 0 || self.image.height() == 0 {
            return;
        }
        if let Some(handle) = self.texture_id.as_mut() {
            handle.set(self.image.clone(), TextureOptions::default());
        }
    }
}
