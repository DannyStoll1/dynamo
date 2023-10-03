use std::collections::HashMap;

use egui::Color32;
use fractal_common::coloring::palette::DiscretePalette;
use fractal_common::types::{Cplx, Period, Real};
use fractal_core::dynamics::symbolic::RationalAngle;
use fractal_core::dynamics::ParameterPlane;

#[derive(Clone, Debug)]
pub struct ColoredCurve
{
    pub curve: Vec<Cplx>,
    pub color: Color32,
}
#[derive(Clone, Debug)]
pub struct ColoredPoint
{
    pub point: Cplx,
    pub color: Color32,
}
pub type ColoredPoints = Vec<ColoredPoint>;

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MarkingMode
{
    pub selection_enabled: bool,
    pub critical: bool,
    pub cycles: Vec<bool>,
    pub precycles: Vec<Vec<bool>>,
    pub rays: HashMap<RationalAngle, Color32>,
    pub palette: DiscretePalette,
}

impl MarkingMode
{
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

    pub fn toggle_precycles(&mut self, preperiod: Period, period: Period)
    {
        let k = preperiod as usize;
        let p = period as usize;

        if self.precycles.len() < k
        {
            self.precycles.resize_with(k, || vec![]);
        }

        let k_precycles = &mut self.precycles[k - 1];
        if k_precycles.len() < k
        {
            k_precycles.resize(p, false);
        }
        k_precycles[p - 1] ^= true;
    }

    pub fn toggle_ray(&mut self, angle: RationalAngle)
    {
        if self.rays.contains_key(&angle)
        {
            self.rays.remove(&angle);
        }
        else
        {
            // TODO: don't hardcode degree=2 here
            let orbit_schema = angle.orbit_schema(2);
            let luminosity = 1.0 - 0.5 * (orbit_schema.preperiod as f32).tanh();
            let color = self
                .palette
                .map_color32(orbit_schema.period as f32, luminosity);
            self.rays.insert(angle, color);
        }
    }

    pub fn compute_points<P>(&self, plane: &P, selection: Cplx) -> ColoredPoints
    where
        P: ParameterPlane + 'static,
    {
        let mut points = vec![];

        if self.selection_enabled
        {
            let color = Color32::WHITE;
            points.push(ColoredPoint {
                point: selection,
                color,
            });
        }

        if self.critical
        {
            let crit_pts = plane.critical_points();
            let color = Color32::RED;
            points.extend(crit_pts.iter().map(|z| ColoredPoint {
                point: (*z).into(),
                color,
            }));
        }

        self.cycles.iter().enumerate().for_each(|(p, enabled)| {
            if *enabled
            {
                let per_pts = plane.cycles(1 + p as Period);
                // let per_pts = plane.precycles(2, 1 + period as Period);
                let color = self.palette.map_color32((1 + p) as f32, 1.);
                points.extend(per_pts.iter().map(|z| ColoredPoint {
                    point: (*z).into(),
                    color,
                }));
            }
        });

        self.precycles.iter().enumerate().for_each(|(k, p)| {
            p.iter().enumerate().for_each(|(period, enabled)| {
                if *enabled
                {
                    let per_pts = plane.precycles(1 + k as Period, 1 + period as Period);
                    // TODO: figure out a good way to color preperiodic points
                    let color = self.palette.map_color32((1 + period) as f32, 1.);
                    points.extend(per_pts.iter().map(|z| ColoredPoint {
                        point: (*z).into(),
                        color,
                    }));
                }
            });
        });

        points
    }

    pub fn compute_rays<P>(&self, plane: &P) -> Vec<ColoredCurve>
    where
        P: ParameterPlane + 'static,
    {
        self.rays
            .iter()
            .filter_map(|(angle, &color)| {
                plane
                    .external_ray(Real::from(*angle), 50, 120)
                    .map(|curve| ColoredCurve { curve, color })
            })
            .collect()
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
    pub fn clear_rays(&mut self)
    {
        self.rays.clear();
    }
    pub fn clear_all(&mut self)
    {
        self.clear_points();
        self.clear_orbits();
        self.clear_rays();
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
