use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};

use egui::{Color32, Painter};
use epaint::{CircleShape, PathShape, Pos2, Stroke};
use image::{ImageBuffer, Rgb};
use imageproc::drawing::{
    draw_antialiased_line_segment_mut, draw_filled_circle_mut, draw_polygon_mut,
};
use imageproc::pixelops::interpolate;
use itertools::Itertools;

use dynamo_color::palette::DiscretePalette;
use dynamo_common::prelude::*;
use dynamo_core::dynamics::Displayable;

use crate::image_frame::ImageFrame;

use self::hashing::HashedReal;

const POINT_RADIUS: f32 = 3.5;
const CURVE_THICKNESS: f32 = 1.4;

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
    fn compute<P: Displayable>(&self, plane: &P, selection: Cplx) -> Self::Object;
}

/// Keys of point-set objects in the data store. Each key may be toggled by the API.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PointSetKey
{
    SelectedPoint,
    CriticalPoints,
    MiscMarkedPoints,
    PeriodicPoints(Period),
    PreperiodicPoints(OrbitSchema),
}
impl ObjectKey for PointSetKey
{
    type Object = Vec<Cplx>;
    fn color_with(&self, palette: &DiscretePalette, _degree: AngleNum) -> Color32
    {
        match self {
            Self::SelectedPoint => Color32::WHITE,
            Self::CriticalPoints => Color32::RED,
            Self::MiscMarkedPoints => Color32::from_rgb(255, 0, 64),
            Self::PeriodicPoints(period) => palette.map(*period as f32, 1.),
            Self::PreperiodicPoints(o) => palette.map_preperiodic(*o),
        }
    }

    fn compute<P: Displayable>(&self, plane: &P, selection: Cplx) -> Vec<Cplx>
    {
        match self {
            Self::SelectedPoint => vec![selection],
            Self::CriticalPoints => plane
                .critical_points()
                .into_iter()
                .map(Into::into)
                .collect(),
            Self::MiscMarkedPoints => plane.other_marked_points(),
            Self::PeriodicPoints(period) => {
                plane.cycles(*period).into_iter().map(Into::into).collect()
            }
            Self::PreperiodicPoints(o) => plane.precycles(*o).into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ContourType
{
    Equipotential,
    Multiplier(Option<HashedReal>),
    ExtendRay,
}
impl ContourType
{
    #[must_use]
    pub fn multiplier(target: Real) -> Self
    {
        Self::Multiplier(Some(target.into()))
    }

    #[must_use]
    pub const fn multiplier_auto() -> Self
    {
        Self::Multiplier(None)
    }

    const fn color(self) -> Color32
    {
        match self {
            Self::Equipotential => Color32::YELLOW,
            Self::Multiplier(_) => Color32::from_rgb(255, 160, 122),
            Self::ExtendRay => Color32::RED, //Color32::from_rgb(127, 127, 127),
        }
    }
}

/// Keys of curve objects in the data store. Each key may be toggled by the API.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum CurveKey
{
    Orbit,
    Ray(RationalAngle),
    Contour(ContourType, hashing::HashedCplx),
}
impl ObjectKey for CurveKey
{
    type Object = Curve;
    fn color_with(&self, palette: &DiscretePalette, degree: AngleNum) -> Color32
    {
        match self {
            Self::Orbit => Color32::GREEN,
            Self::Ray(angle) => {
                let o = angle.with_degree(degree).orbit_schema();
                palette.map_preperiodic(o)
            }
            Self::Contour(ctype, _) => ctype.color(),
        }
    }

    fn compute<P: Displayable>(&self, plane: &P, selection: Cplx) -> Curve
    {
        match self {
            Self::Orbit => plane.iter_orbit(selection).map(Into::into).collect(),
            Self::Ray(angle) => plane.external_ray(*angle).unwrap_or_default(),

            Self::Contour(ctype, point) => match ctype {
                ContourType::Equipotential => plane.equipotential(Cplx::from(*point)).compute(),
                ContourType::Multiplier(Some(target)) => {
                    let mut contour = plane.aux_contour(Cplx::from(*point));
                    contour.set_target((*target).into());
                    contour.compute()
                }
                ContourType::Multiplier(_) => plane.aux_contour(Cplx::from(*point)).compute(),
                ContourType::ExtendRay => plane.extend_ray(Cplx::from(*point)).compute(),
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct ColoredMaybeHidden<O>
{
    pub object: O,
    pub color: Color32,
    pub visible: bool,
}

#[derive(Clone, Debug)]
pub struct Colored<O>
{
    pub object: O,
    pub color: Color32,
}
impl<O> From<ColoredMaybeHidden<O>> for Colored<O>
{
    fn from(value: ColoredMaybeHidden<O>) -> Self
    {
        Self {
            object: value.object,
            color: value.color,
        }
    }
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
struct EnvironmentInfo<'plane, 'palette, P: Displayable>
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
    pub objects: HashMap<K, ColoredMaybeHidden<O>>,
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

    fn process_task<P: Displayable>(&mut self, task: MarkingTask<K>, e: &EnvironmentInfo<P>)
    {
        match task {
            MarkingTask::Enable(key) => {
                self.enable(key, e);
            }
            MarkingTask::Disable(key) => {
                self.disable(&key);
            }
            MarkingTask::Toggle(key) => {
                if self.objects.contains_key(&key) {
                    self.disable(&key);
                } else {
                    self.enable(key, e);
                }
            }
            MarkingTask::Recompute(key) => {
                if let Some(col_obj) = self.objects.get_mut(&key) {
                    col_obj.object = key.compute(e.plane, e.selection);
                    col_obj.color = key.color_with(e.palette, self.degree);
                }
            }
            MarkingTask::Recolor(key) => {
                if let Some(col_obj) = self.objects.get_mut(&key) {
                    col_obj.color = key.color_with(e.palette, self.degree);
                }
            }
            MarkingTask::RecomputeAll => {
                self.recompute_all(e.plane, e.selection);
            }
            MarkingTask::RecolorAll => {
                self.recolor_all(e.palette);
            }
        }
    }

    fn enable<P: Displayable>(&mut self, key: K, e: &EnvironmentInfo<P>)
    {
        let col_obj = ColoredMaybeHidden {
            object: key.compute(e.plane, e.selection),
            color: key.color_with(e.palette, self.degree),
            visible: true,
        };
        self.objects.insert(key, col_obj);
    }

    fn disable(&mut self, key: &K)
    {
        self.objects.remove(key);
    }

    pub fn recolor_all(&mut self, palette: &DiscretePalette)
    {
        self.objects.iter_mut().for_each(|(key, col_obj)| {
            col_obj.color = key.color_with(palette, self.degree);
        });
    }

    pub fn recompute_all<P: Displayable>(&mut self, plane: &P, selection: Cplx)
    {
        self.objects.iter_mut().for_each(|(key, col_obj)| {
            col_obj.object = key.compute(plane, selection);
        });
    }

    fn process_all_tasks<P: Displayable>(&mut self, env: &EnvironmentInfo<P>)
    {
        let tasks: Vec<_> = self.tasks.drain(..).collect();
        for task in &tasks {
            self.process_task(task.clone(), env);
        }
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
    path_cache: RefCell<PathCache>,
}
impl Marking
{
    #[must_use]
    pub const fn with_degree(mut self, degree: AngleNum) -> Self
    {
        self.point_sets.degree = degree;
        self.curves.degree = degree;
        self
    }

    pub fn toggle_selection(&mut self)
    {
        self.point_sets.sched_toggle(PointSetKey::SelectedPoint);
    }

    pub fn enable_selection(&mut self)
    {
        self.point_sets.sched_enable(PointSetKey::SelectedPoint);
    }

    pub fn disable_selection(&mut self)
    {
        self.point_sets.sched_disable(PointSetKey::SelectedPoint);
    }

    pub fn select_point(&mut self, point: Cplx)
    {
        if let Some(selection) = self.point_sets.objects.get_mut(&PointSetKey::SelectedPoint) {
            selection.object = vec![point];
        }
    }

    pub fn toggle_critical(&mut self)
    {
        self.point_sets.sched_toggle(PointSetKey::CriticalPoints);
    }

    pub fn toggle_misc_marked(&mut self)
    {
        self.point_sets.sched_toggle(PointSetKey::MiscMarkedPoints);
    }

    pub fn toggle_cycles_of_period(&mut self, period: Period)
    {
        self.point_sets
            .sched_toggle(PointSetKey::PeriodicPoints(period));
    }

    pub fn toggle_ray(&mut self, angle: RationalAngle)
    {
        self.curves.sched_toggle(CurveKey::Ray(angle));
        self.path_cache.borrow_mut().set_stale();
    }

    pub fn enable_ray(&mut self, angle: RationalAngle)
    {
        self.curves.sched_enable(CurveKey::Ray(angle));
        self.path_cache.borrow_mut().set_stale();
    }

    pub fn toggle_contour(&mut self, contour_type: ContourType, base_point: Cplx)
    {
        self.curves
            .sched_toggle(CurveKey::Contour(contour_type, base_point.into()));
        self.path_cache.borrow_mut().set_stale();
    }

    pub fn sched_recompute_all(&mut self)
    {
        self.point_sets.sched_recompute_all();
        self.curves.sched_recompute_all();
        self.path_cache.borrow_mut().set_stale();
    }
    pub fn sched_recolor_all(&mut self)
    {
        self.point_sets.sched_recolor_all();
        self.curves.sched_recolor_all();
    }

    pub fn flush_path_cache(&mut self)
    {
        self.path_cache.borrow_mut().set_stale();
    }

    pub fn remove_all_annotations(&mut self)
    {
        self.point_sets.disable_all();
        self.curves.disable_all();
        self.path_cache.borrow_mut().set_stale();
    }

    pub fn process_all_tasks<P: Displayable>(
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
        let col_obj = ColoredMaybeHidden {
            object: orbit,
            color,
            visible: true,
        };
        self.curves.objects.insert(CurveKey::Orbit, col_obj);
        self.path_cache.borrow_mut().set_stale();
    }

    pub fn disable_orbit(&mut self)
    {
        self.curves.disable(&CurveKey::Orbit);
        self.path_cache.borrow_mut().set_stale();
    }

    pub fn disable_all_contours(&mut self)
    {
        let to_remove: Vec<_> = self
            .curves
            .objects
            .keys()
            .filter(|k| matches!(k, CurveKey::Contour(..)))
            .copied()
            .collect();
        for key in &to_remove {
            self.curves.objects.remove(key);
        }
        self.path_cache.borrow_mut().set_stale();
    }

    pub fn disable_all_rays(&mut self)
    {
        let to_remove: Vec<_> = self
            .curves
            .objects
            .keys()
            .filter(|k| matches!(k, CurveKey::Ray(_)))
            .copied()
            .collect();
        for key in &to_remove {
            self.curves.objects.remove(key);
        }
        self.path_cache.borrow_mut().set_stale();
    }

    pub fn disable_all_points(&mut self)
    {
        self.point_sets.disable_all();
    }

    pub fn disable_all_curves(&mut self)
    {
        self.curves.disable_all();
        self.path_cache.borrow_mut().set_stale();
    }

    pub fn iter_points(&self) -> impl Iterator<Item = ColoredPoint> + '_
    {
        self.point_sets
            .objects
            .values()
            .filter(|o| o.visible)
            .flat_map(
                |ColoredMaybeHidden {
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

    fn iter_visible_curves(&self) -> impl Iterator<Item = ColoredMaybeHidden<Curve>> + '_
    {
        self.curves.objects.values().filter(|o| o.visible).cloned()
    }

    pub fn ray_landing_point(&self, angle: RationalAngle) -> Option<Cplx>
    {
        let col_ray = self.curves.objects.get(&CurveKey::Ray(angle))?;
        col_ray.object.last().copied()
    }

    fn update_cache(&self, grid: &PointGrid, frame: &ImageFrame)
    {
        self.path_cache.borrow_mut().paths.clear();
        self.path_cache
            .borrow_mut()
            .paths
            .extend(self.iter_visible_curves().map(
                |ColoredMaybeHidden {
                     object: zs, color, ..
                 }| {
                    let points = zs
                        .iter()
                        .map(|z| {
                            let pt = grid.locate_point(*z);
                            frame.to_global_coords(pt.into())
                        })
                        .collect();
                    Colored {
                        object: points,
                        color,
                    }
                },
            ));

        self.path_cache.borrow_mut().set_fresh();
    }

    pub fn draw_points(&self, painter: &Painter, grid: &PointGrid, frame: &ImageFrame)
    {
        for ColoredPoint { point: z, color } in self.iter_points() {
            let point = frame.to_global_coords(grid.locate_point(z).into());
            let patch = CircleShape::filled(point, POINT_RADIUS, color);
            painter.add(patch);
        }
    }

    pub fn draw_curves(&self, painter: &Painter, grid: &PointGrid, frame: &ImageFrame)
    {
        if self.path_cache.borrow().is_stale() {
            self.update_cache(grid, frame);
        }
        self.path_cache.borrow().paths.iter().for_each(
            |Colored {
                 object: path,
                 color,
             }| {
                let stroke = Stroke::new(1.0, *color);
                let path = PathShape::line(path.clone(), stroke);
                painter.add(path);
            },
        );
    }

    fn draw_curves_to_image(&self, grid: &PointGrid, image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>)
    {
        let thickness = CURVE_THICKNESS * (image.width() as f32) / 768.;

        self.iter_visible_curves().for_each(
            |ColoredMaybeHidden {
                 object: curve,
                 color,
                 ..
             }| {
                let (r, g, b, _a) = color.to_tuple();
                let color = Rgb([r, g, b]);
                CurveDrawJob {
                    curve: &curve,
                    color,
                    thickness,
                    grid,
                }
                .draw_to(image);
            },
        );
    }
    fn draw_points_to_image(&self, grid: &PointGrid, image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>)
    {
        let radius = POINT_RADIUS * (image.width() as f32) / 768.;
        self.iter_points()
            .for_each(|ColoredPoint { point, color }| {
                let (r, g, b, _a) = color.to_tuple();
                let color = Rgb([r, g, b]);
                let [x, y] = grid.locate_point(point);
                let center = (x as i32, y as i32);
                draw_filled_circle_mut(image, center, radius as i32, color);
            });
    }
    pub fn mark_image(&self, grid: &PointGrid, image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>)
    {
        self.draw_curves_to_image(grid, image);
        self.draw_points_to_image(grid, image);
    }
}

mod hashing
{

    use dynamo_common::types::{Cplx, Real};

    #[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
    pub struct HashedReal(u64);

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
            Self::from_bits(encoded.0)
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

#[derive(Clone)]
pub struct PathCache
{
    paths: Vec<Colored<Vec<Pos2>>>,
    needs_refresh: bool,
}
impl Default for PathCache
{
    fn default() -> Self
    {
        Self {
            paths: Vec::new(),
            needs_refresh: true,
        }
    }
}

impl PathCache
{
    pub fn set_fresh(&mut self)
    {
        self.needs_refresh = false;
    }
    #[must_use]
    pub const fn is_fresh(&self) -> bool
    {
        !self.needs_refresh
    }
    pub fn set_stale(&mut self)
    {
        self.needs_refresh = true;
    }
    #[must_use]
    pub const fn is_stale(&self) -> bool
    {
        self.needs_refresh
    }
}

struct CurveDrawJob<'a>
{
    curve: &'a Curve,
    color: Rgb<u8>,
    thickness: f32,
    grid: &'a PointGrid,
}
impl<'a> CurveDrawJob<'a>
{
    pub fn draw_thick(self, image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>)
    {
        self.curve
            .iter()
            .copied()
            .map(|z| self.grid.locate_point(z))
            .tuple_windows()
            .for_each(|([x0, y0], [x1, y1])| {
                let normal_x = y1 - y0;
                let normal_y = x0 - x1;
                let n_length = normal_x.hypot(normal_y);

                let nx = 0.5 * self.thickness * normal_x / n_length;
                let ny = 0.5 * self.thickness * normal_y / n_length;

                let corners = [
                    (x0 - nx, y0 - ny),
                    (
                        0.866f32.mul_add(ny, 0.5f32.mul_add(-nx, x0)),
                        0.866f32.mul_add(-nx, 0.5f32.mul_add(-ny, y0)),
                    ), // hexagonal corner
                    (
                        0.866f32.mul_add(ny, 0.5f32.mul_add(nx, x0)),
                        0.866f32.mul_add(-nx, 0.5f32.mul_add(ny, y0)),
                    ),
                    (x0 + nx, y0 + ny),
                    (x1 + nx, y1 + ny),
                    (
                        0.866f32.mul_add(-ny, 0.5f32.mul_add(nx, x1)),
                        0.866f32.mul_add(nx, 0.5f32.mul_add(ny, y1)),
                    ),
                    (
                        0.866f32.mul_add(-ny, 0.5f32.mul_add(-nx, x1)),
                        0.866f32.mul_add(nx, 0.5f32.mul_add(-ny, y1)),
                    ),
                    (x1 - nx, y1 - ny),
                ]
                .map(|(x, y)| imageproc::point::Point::new(x as i32, y as i32));

                if corners[0] != corners[7] {
                    draw_polygon_mut(image, &corners, self.color);
                }
            });
    }

    fn draw_thin(self, image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>)
    {
        self.curve
            .iter()
            .copied()
            .map(|z| self.grid.locate_point(z))
            .map(|[x, y]| (x as i32, y as i32))
            .tuple_windows()
            .for_each(|(p0, p1)| {
                draw_antialiased_line_segment_mut(image, p0, p1, self.color, interpolate);
            });
    }

    pub fn draw_to(self, image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>)
    {
        if self.thickness <= 1.0 {
            self.draw_thin(image);
        } else {
            self.draw_thick(image);
        }
    }
}
