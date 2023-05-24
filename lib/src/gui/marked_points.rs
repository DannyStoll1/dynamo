use super::pane::ColoredPoints;
use crate::coloring::palette::DiscretePalette;
use crate::dynamics::ParameterPlane;
use crate::types::*;
use eframe::egui::Color32;

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MarkingMode
{
    pub critical: bool,
    pub cycles: Vec<bool>,
    pub palette: DiscretePalette,
}

impl MarkingMode
{
    pub fn compute<P>(&self, plane: &P) -> ColoredPoints
    where
        P: ParameterPlane + 'static,
    {
        let mut points = vec![];
        if self.critical
        {
            let crit_pts = plane.critical_points();
            let color = Color32::RED;
            points.extend(crit_pts.iter().map(|z| ((*z).into(), color)));
        }

        self.cycles
            .iter()
            .enumerate()
            .for_each(|(period, enabled)| {
                if *enabled
                {
                    let per_pts = plane.cycles(1 + period as Period);
                    let color = self.palette.map_color32(period as f32, 1.);
                    points.extend(per_pts.iter().map(|z| ((*z).into(), color)));
                }
            });

        points
    }

    pub fn toggle_critical(&mut self)
    {
        self.critical ^= true;
    }

    pub fn toggle_cycles(&mut self, period: Period)
    {
        let p = period as usize;
        if self.cycles.len() < p
        {
            self.cycles.resize(p, false);
        }
        self.cycles[p - 1] ^= true;
    }
}
