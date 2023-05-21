use crate::point_grid::*;
use crate::primitive_types::*;
use eframe::egui::{Color32, ColorImage};

#[derive(Clone)]
pub struct ImageAnnotation {
    pub point_grid: PointGrid,
    pub marked_points: Vec<(usize, usize, Color32)>,
}
impl ImageAnnotation {
    #[inline]
    pub fn mark_pixel(&mut self, x: usize, y: usize, color: Color32) {
        self.marked_points.push((x, y, color));
    }

    #[inline]
    pub fn clear(&mut self) {
        self.marked_points = vec![];
    }

    #[inline]
    pub fn mark_value(&mut self, v: ComplexNum, color: Color32) {
        if let Some((x, y)) = self.point_grid.locate_point(v) {
            self.mark_pixel(x, y, color);
        }
    }
    pub fn apply_annotations(&self, img: &mut ColorImage) {
        self.marked_points.iter().for_each(|(x, y, color)| {
            img.pixels[x + y * self.point_grid.res_y] = *color;
        });
    }
}
