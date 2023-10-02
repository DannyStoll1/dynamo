use std::collections::HashSet;

use egui::Color32;
use fractal_common::coloring::palette::DiscretePalette;
use fractal_common::types::{Cplx, Period, Rational};
use fractal_core::dynamics::ParameterPlane;

pub type ColoredCurve = (Vec<Cplx>, Color32);
pub type ColoredPoint = (Cplx, Color32);
pub type ColoredPoints = Vec<ColoredPoint>;

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MarkingMode
{
    pub selection_enabled: bool,
    pub critical: bool,
    pub cycles: Vec<bool>,
    pub rays: HashSet<Rational>,
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

    pub fn toggle_ray(&mut self, angle: Rational)
    {
        if self.rays.contains(&angle)
        {
            self.rays.remove(&angle);
        }
        else
        {
            self.rays.insert(angle);
        }
    }
}

#[derive(Default, Clone)]
pub struct MarkedData
{
    pub orbits: Vec<ColoredCurve>,
    pub rays: Vec<ColoredCurve>,
    pub points: Vec<ColoredPoint>,
}

impl MarkedData
{
    pub fn clear_points(&mut self)
    {
        self.points.clear();
    }
    pub fn clear_orbits(&mut self)
    {
        self.orbits.clear();
    }
    pub fn clear_all(&mut self)
    {
        self.clear_points();
        self.clear_orbits();
    }
    pub fn iter_curves<'a>(&'a self) -> Box<dyn Iterator<Item = &ColoredCurve> + 'a>
    {
        Box::new(self.orbits.iter().chain(self.rays.iter()))
    }
    pub fn iter_points<'a>(&'a self) -> Box<dyn Iterator<Item = &ColoredPoint> + 'a>
    {
        Box::new(self.points.iter())
    }
}
