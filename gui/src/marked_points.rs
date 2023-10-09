use std::collections::{HashMap, VecDeque};

use egui::Color32;
use fractal_common::coloring::palette::DiscretePalette;
use fractal_common::types::{AngleNum, Cplx, Period, Real};
use fractal_core::dynamics::symbolic::{OrbitSchema, RationalAngle};
use fractal_core::dynamics::ParameterPlane;

type Curve = Vec<Cplx>;

pub struct ColoredPoint
{
    pub point: Cplx,
    pub color: Color32,
}

pub trait ObjectKey: Clone + std::hash::Hash + std::cmp::Eq + std::fmt::Debug
{
    type Object;
    fn color_with(&self, palette: &DiscretePalette, degree: AngleNum) -> Color32;
    fn compute<P: ParameterPlane>(&self, plane: &P, selection: Cplx) -> Self::Object;
}

/// Keys of point-set objects in the data store. Each key may be toggled by the API.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PointSetKey
{
    SelectedPoint,
    CriticalPoints,
    PeriodicPoints(Period),
    PreperiodicPoints(OrbitSchema),
}
impl ObjectKey for PointSetKey
{
    type Object = Vec<Cplx>;
    fn color_with(&self, palette: &DiscretePalette, _degree: AngleNum) -> Color32
    {
        match self
        {
            Self::SelectedPoint => Color32::WHITE,
            Self::CriticalPoints => Color32::RED,
            Self::PeriodicPoints(period) => palette.map_color32(*period as f32, 1.),
            Self::PreperiodicPoints(o) =>
            {
                palette.map_color32(o.period as f32, 1.0 - 0.5 * (o.preperiod as f32).tanh())
            }
        }
    }

    fn compute<P: ParameterPlane>(&self, plane: &P, selection: Cplx) -> Vec<Cplx>
    {
        match self
        {
            Self::SelectedPoint => vec![selection],
            Self::CriticalPoints => plane
                .critical_points()
                .into_iter()
                .map(|z| z.into())
                .collect(),
            Self::PeriodicPoints(period) => plane
                .cycles(*period)
                .into_iter()
                .map(|z| z.into())
                .collect(),
            Self::PreperiodicPoints(o) =>
            {
                plane.precycles(*o).into_iter().map(|z| z.into()).collect()
            }
        }
    }
}

/// Keys of curve objects in the data store. Each key may be toggled by the API.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum CurveKey
{
    Orbit,
    Ray(RationalAngle),
    Equipotential(hashing::HashedCplx),
}
impl ObjectKey for CurveKey
{
    type Object = Curve;
    fn color_with(&self, palette: &DiscretePalette, degree: AngleNum) -> Color32
    {
        match self
        {
            Self::Orbit => Color32::GREEN,
            Self::Ray(angle) =>
            {
                let o = angle.orbit_schema(degree);
                palette.map_color32(o.period as f32, 1.0 - 0.5 * (o.preperiod as f32).tanh())
            }
            Self::Equipotential(_) => Color32::YELLOW,
        }
    }

    fn compute<P: ParameterPlane>(&self, plane: &P, selection: Cplx) -> Curve
    {
        match self
        {
            Self::Orbit => plane.iter_orbit(selection).map(|z| z.into()).collect(),
            Self::Ray(angle) => plane
                .external_ray(Real::from(*angle))
                .unwrap_or_else(|| vec![]),
            Self::Equipotential(point) => plane
                .equipotential(Cplx::from(*point))
                .unwrap_or_else(|| vec![]),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Colored<O>
{
    pub object: O,
    pub color: Color32,
    pub visible: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum MarkingTask<K>
{
    Enable(K),
    Disable(K),
    Toggle(K),
    Recompute(K),
    Recolor(K),
    RecomputeAll,
    RecolorAll,
}

#[derive(Clone, Copy, Debug)]
struct EnvironmentInfo<'plane, 'palette, P: ParameterPlane>
{
    plane: &'plane P,
    selection: Cplx,
    palette: &'palette DiscretePalette,
}

#[derive(Clone, Debug)]
pub struct MarkedObjectStore<K, O>
where
    K: ObjectKey<Object = O>,
{
    pub objects: HashMap<K, Colored<O>>,
    tasks: VecDeque<MarkingTask<K>>,
    pub degree: AngleNum,
}
impl<K, O> Default for MarkedObjectStore<K, O>
where
    K: ObjectKey<Object = O>,
{
    fn default() -> Self
    {
        Self {
            objects: HashMap::new(),
            tasks: VecDeque::new(),
            degree: 0,
        }
    }
}

// #[derive(Default, Clone)]
// pub struct Marking
// {
//     pub point_sets: HashMap<PointSetKey, ColoredPointSet>,
//     pub curves: HashMap<CurveKey, ColoredCurve>,
//     tasks: VecDeque<MarkingTask>,
//     degree: AngleNum,
// }
//
// #[must_use]
// pub fn with_degree(mut self, degree: AngleNum) -> Self
// {
//     self.degree = degree;
//     self
// }
//
// pub fn toggle_selection(&mut self)
// {
//     self.sched_toggle(K::SelectedPoint);
// }
//
// pub fn toggle_critical(&mut self)
// {
//     self.sched_toggle(K::SelectedPoint);
// }
//
// pub fn toggle_cycles_of_period(&mut self, period: Period)
// {
//     self.sched_toggle(K::PeriodicPoints(period));
// }
//
// pub fn toggle_ray(&mut self, angle: RationalAngle)
// {
//     self.sched_toggle_curve(CurveKey::Ray(angle));
// }
impl<K, O> MarkedObjectStore<K, O>
where
    K: ObjectKey<Object = O>,
{
    pub fn sched_toggle(&mut self, key: K)
    {
        self.tasks.push_back(MarkingTask::Toggle(key));
    }
    pub fn sched_enable(&mut self, key: K)
    {
        self.tasks.push_back(MarkingTask::Enable(key));
    }
    pub fn sched_disable(&mut self, key: K)
    {
        self.tasks.push_back(MarkingTask::Disable(key));
    }
    pub fn sched_recompute(&mut self, key: K)
    {
        self.tasks.push_back(MarkingTask::Recompute(key));
    }
    pub fn sched_recolor(&mut self, key: K)
    {
        self.tasks.push_back(MarkingTask::Recolor(key));
    }

    pub fn sched_recompute_all(&mut self)
    {
        self.tasks.push_back(MarkingTask::RecomputeAll);
    }
    pub fn sched_recolor_all(&mut self)
    {
        self.tasks.push_back(MarkingTask::RecolorAll);
    }

    fn process_task<P: ParameterPlane>(&mut self, task: MarkingTask<K>, e: &EnvironmentInfo<P>)
    {
        match task
        {
            MarkingTask::Enable(key) =>
            {
                self.enable(key, e);
            }
            MarkingTask::Disable(key) =>
            {
                self.disable(key);
            }
            MarkingTask::Toggle(key) =>
            {
                if self.objects.contains_key(&key)
                {
                    self.disable(key);
                }
                else
                {
                    self.enable(key, e);
                }
            }
            MarkingTask::Recompute(key) =>
            {
                if let Some(col_obj) = self.objects.get_mut(&key)
                {
                    col_obj.object = key.compute(e.plane, e.selection);
                    col_obj.color = key.color_with(e.palette, self.degree);
                }
            }
            MarkingTask::Recolor(key) =>
            {
                if let Some(col_obj) = self.objects.get_mut(&key)
                {
                    col_obj.color = key.color_with(e.palette, self.degree);
                }
            }
            MarkingTask::RecomputeAll =>
            {
                self.recompute_all(e.plane, e.selection);
            }
            MarkingTask::RecolorAll =>
            {
                self.recolor_all(e.palette);
            }
        }
    }

    fn enable<P: ParameterPlane>(&mut self, key: K, e: &EnvironmentInfo<P>)
    {
        let col_obj = Colored {
            object: key.compute(e.plane, e.selection),
            color: key.color_with(e.palette, self.degree),
            visible: true,
        };
        self.objects.insert(key, col_obj);
    }

    fn disable(&mut self, key: K)
    {
        self.objects.remove(&key);
    }

    pub fn recolor_all(&mut self, palette: &DiscretePalette)
    {
        self.objects.iter_mut().for_each(|(key, col_obj)| {
            col_obj.color = key.color_with(palette, self.degree);
        });
    }

    pub fn recompute_all<P: ParameterPlane>(&mut self, plane: &P, selection: Cplx)
    {
        self.objects.iter_mut().for_each(|(key, col_obj)| {
            col_obj.object = key.compute(plane, selection);
        });
    }

    fn process_all_tasks<P: ParameterPlane>(&mut self, env: &EnvironmentInfo<P>)
    {
        let tasks: Vec<_> = self.tasks.drain(..).collect();
        tasks.iter().for_each(|task| {
            self.process_task(task.clone(), env);
        });
    }

    pub fn clear_all_tasks(&mut self)
    {
        self.tasks.clear();
    }

    pub fn disable_all(&mut self)
    {
        self.objects.clear();
    }
}

#[derive(Default, Clone)]
pub struct Marking
{
    point_sets: MarkedObjectStore<PointSetKey, Vec<Cplx>>,
    curves: MarkedObjectStore<CurveKey, Curve>,
}
impl Marking
{
    #[must_use]
    pub fn with_degree(mut self, degree: AngleNum) -> Self
    {
        self.point_sets.degree = degree;
        self.curves.degree = degree;
        self
    }

    pub fn toggle_selection(&mut self)
    {
        self.point_sets.sched_toggle(PointSetKey::SelectedPoint);
    }

    pub fn select_point(&mut self, point: Cplx)
    {
        if let Some(selection) = self.point_sets.objects.get_mut(&PointSetKey::SelectedPoint)
        {
            selection.object = vec![point];
        }
    }

    pub fn toggle_critical(&mut self)
    {
        self.point_sets.sched_toggle(PointSetKey::CriticalPoints);
    }

    pub fn toggle_cycles_of_period(&mut self, period: Period)
    {
        self.point_sets
            .sched_toggle(PointSetKey::PeriodicPoints(period));
    }

    pub fn toggle_ray(&mut self, angle: RationalAngle)
    {
        self.curves.sched_toggle(CurveKey::Ray(angle));
    }

    pub fn toggle_equipotential(&mut self, base_point: Cplx)
    {
        self.curves
            .sched_toggle(CurveKey::Equipotential(base_point.into()));
    }

    pub fn sched_recompute_all(&mut self)
    {
        self.point_sets.sched_recompute_all();
        self.curves.sched_recompute_all();
    }
    pub fn sched_recolor_all(&mut self)
    {
        self.point_sets.sched_recolor_all();
        self.curves.sched_recolor_all();
    }

    pub fn remove_all_annotations(&mut self)
    {
        self.point_sets.disable_all();
        self.curves.disable_all();
    }

    pub fn process_all_tasks<P: ParameterPlane>(
        &mut self,
        plane: &P,
        selection: Cplx,
        palette: &DiscretePalette,
    )
    {
        let env = EnvironmentInfo {
            plane,
            selection,
            palette,
        };
        self.point_sets.process_all_tasks(&env);
        self.curves.process_all_tasks(&env);
    }

    pub fn mark_orbit_manually(&mut self, orbit: Curve, color: Color32)
    {
        let col_obj = Colored {
            object: orbit,
            color,
            visible: true,
        };
        self.curves.objects.insert(CurveKey::Orbit, col_obj);
    }

    pub fn disable_orbit(&mut self)
    {
        self.curves.sched_disable(CurveKey::Orbit);
    }

    pub fn disable_all_equipotentials(&mut self)
    {
        let to_remove: Vec<_> = self
            .curves
            .objects
            .keys()
            .filter(|k| matches!(k, CurveKey::Equipotential(_)))
            .cloned()
            .collect();
        to_remove.iter().for_each(|k| {
            self.curves.objects.remove(k);
        });
    }

    pub fn disable_all_rays(&mut self)
    {
        let to_remove: Vec<_> = self
            .curves
            .objects
            .keys()
            .filter(|k| matches!(k, CurveKey::Ray(_)))
            .cloned()
            .collect();
        to_remove.iter().for_each(|k| {
            self.curves.objects.remove(k);
        });
    }

    pub fn disable_all_points(&mut self)
    {
        self.point_sets.disable_all();
    }

    pub fn disable_all_curves(&mut self)
    {
        self.curves.disable_all();
    }

    pub fn iter_visible_points(&self) -> impl Iterator<Item = ColoredPoint> + '_
    {
        self.point_sets
            .objects
            .values()
            .filter(|o| o.visible)
            .flat_map(
                |Colored {
                     object: point_set,
                     color,
                     ..
                 }| {
                    point_set.iter().map(|&point| ColoredPoint {
                        point,
                        color: *color,
                    })
                },
            )
    }

    pub fn iter_visible_curves(&self) -> impl Iterator<Item = Colored<Curve>> + '_
    {
        self.curves.objects.values().filter(|o| o.visible).cloned()
    }

    pub fn ray_landing_point(&self, angle: RationalAngle) -> Option<Cplx>
    {
        let col_ray = self.curves.objects.get(&CurveKey::Ray(angle))?;
        col_ray.object.last().copied()
    }
}

mod hashing
{

    use fractal_common::types::{Cplx, Real};
    use std::mem;

    fn integer_decode(val: f64) -> (u64, i16, i8)
    {
        let bits: u64 = unsafe { mem::transmute(val) };
        let sign: i8 = if bits >> 63 == 0 { 1 } else { -1 };
        let mut exponent: i16 = ((bits >> 52) & 0x7ff) as i16;
        let mantissa = if exponent == 0
        {
            (bits & 0xfffffffffffff) << 1
        }
        else
        {
            (bits & 0xfffffffffffff) | 0x10000000000000
        };

        exponent -= 1023 + 52;
        (mantissa, exponent, sign)
    }

    #[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
    pub(super) struct HashedReal(u64);

    impl From<Real> for HashedReal
    {
        fn from(real: Real) -> Self
        {
            Self(real.to_bits())
        }
    }

    impl From<HashedReal> for Real
    {
        fn from(encoded: HashedReal) -> Self
        {
            Real::from_bits(encoded.0)
        }
    }

    #[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
    pub(super) struct HashedCplx
    {
        re: HashedReal,
        im: HashedReal,
    }

    impl From<Cplx> for HashedCplx
    {
        fn from(value: Cplx) -> Self
        {
            Self {
                re: value.re.into(),
                im: value.im.into(),
            }
        }
    }

    impl From<HashedCplx> for Cplx
    {
        fn from(value: HashedCplx) -> Self
        {
            Self {
                re: value.re.into(),
                im: value.im.into(),
            }
        }
    }
}
