use super::pane::ColoredPoints;
use egui::Color32;
use fractal_common::coloring::palette::DiscretePalette;
use fractal_common::types::{Cplx, Period};
use fractal_core::dynamics::ParameterPlane;

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MarkingMode
{
    pub selection_enabled: bool,
    pub critical: bool,
    pub cycles: Vec<bool>,
    pub palette: DiscretePalette,
}

impl MarkingMode
{
    pub fn compute<P>(&self, plane: &P, selection: Cplx) -> ColoredPoints
    where
        P: ParameterPlane + 'static,
    {
        let mut points = vec![];

        if self.selection_enabled
        {
            let color = Color32::WHITE;
            points.push((selection, color));
        }

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
                    let color = self.palette.map_color32((1 + period) as f32, 1.);
                    points.extend(per_pts.iter().map(|z| ((*z).into(), color)));
                }
            });

        points
    }

    pub fn toggle_selection(&mut self)
    {
        self.selection_enabled ^= true;
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
