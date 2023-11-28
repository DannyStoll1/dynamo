use egui::{Color32, Pos2, Ui};
use std::path::Path;

use crate::actions::ChangeBoolean;
use crate::marked_points::ContourType;

use super::image_frame::ImageFrame;
use super::marked_points::Marking;
use dynamo_color::prelude::*;
use dynamo_common::prelude::*;
use dynamo_core::error::FindPointResult;
use dynamo_core::prelude::*;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub mod id;
pub mod tasks;
use tasks::*;

pub trait Pane
{
    fn tasks(&self) -> &PaneTasks;
    fn tasks_mut(&mut self) -> &mut PaneTasks;
    fn frame(&self) -> &ImageFrame;
    fn frame_mut(&mut self) -> &mut ImageFrame;

    fn get_coloring(&self) -> &Coloring;
    fn get_coloring_mut(&mut self) -> &mut Coloring;

    fn select_point(&mut self, point: Cplx);
    fn get_selection(&self) -> Cplx;
    fn reset_selection(&mut self);
    fn reset(&mut self);
    fn select_nearby_point(&mut self, orbit_schema: OrbitSchema) -> FindPointResult<Cplx>;
    fn map_selection(&mut self);
    fn stop_following(&mut self);
    fn set_follow_state(&mut self, follow_state: FollowState);
    fn degree(&self) -> AngleNum;

    fn draw_contour(&mut self, contour_type: ContourType);
    fn draw_aux_contours(&mut self);

    fn get_image_frame(&self) -> &ImageFrame;
    fn get_image_frame_mut(&mut self) -> &mut ImageFrame;

    fn clear_marked_points(&mut self);
    fn clear_marked_orbit(&mut self);
    fn clear_marked_rays(&mut self);
    fn clear_equipotentials(&mut self);
    fn clear_curves(&mut self);
    fn put_marked_points(&self, ui: &mut Ui);
    fn put_marked_curves(&self, ui: &mut Ui);

    fn plane_type(&self) -> PlaneType;
    fn name(&self) -> String;
    fn long_name(&self) -> String;

    fn grid(&self) -> &PointGrid;

    fn grid_mut(&mut self) -> &mut PointGrid;

    fn schedule_compute(&mut self)
    {
        self.tasks_mut().compute.schedule_init_run();
        self.tasks_mut().draw.schedule_init_run();
        self.marking_mut().sched_recompute_all();
    }

    fn schedule_recompute(&mut self)
    {
        self.tasks_mut().compute.schedule_rerun();
        self.tasks_mut().draw.schedule_rerun();
        self.marking_mut().sched_recompute_all();
    }

    fn schedule_draw(&mut self)
    {
        self.tasks_mut().draw.schedule_init_run();
    }

    fn schedule_redraw(&mut self)
    {
        self.tasks_mut().draw.schedule_rerun();
    }

    fn resize_x(&mut self, width: usize)
    {
        self.grid_mut().resize_x(width);
        self.schedule_compute();
    }

    fn resize_y(&mut self, height: usize)
    {
        self.grid_mut().resize_y(height);
        self.schedule_compute();
    }

    fn change_palette(&mut self, palette: Palette)
    {
        self.get_coloring_mut().set_palette(palette);
        self.marking_mut().sched_recolor_all();
        self.schedule_redraw();
    }

    fn scale_palette(&mut self, scale_factor: f64)
    {
        self.get_coloring_mut().scale_period(scale_factor);
        self.schedule_redraw();
    }

    fn set_coloring_algorithm(&mut self, coloring_algorithm: IncoloringAlgorithm)
    {
        self.get_coloring_mut()
            .set_interior_algorithm(coloring_algorithm);
        self.schedule_redraw();
    }

    fn shift_palette(&mut self, shift: f64)
    {
        self.get_coloring_mut().adjust_phase(shift);
        self.schedule_redraw();
    }

    fn zoom(&mut self, scale: Real, base_point: Cplx);

    #[inline]
    fn pan(&mut self, offset_vector: Cplx)
    {
        self.grid_mut().translate(offset_vector);
        self.schedule_recompute();
        self.schedule_redraw();
    }

    #[inline]
    fn nudge_horizontal(&mut self, relative_offset: f64)
    {
        let grid_width = self.grid().range_x();
        let translation_vector = Cplx::from(grid_width * relative_offset);
        self.pan(translation_vector);
    }

    #[inline]
    fn nudge_vertical(&mut self, relative_offset: f64)
    {
        let grid_height = self.grid().range_y();
        let translation_vector = Cplx::new(0., grid_height * relative_offset);
        self.pan(translation_vector);
    }

    #[inline]
    fn pan_relative(&mut self, x: f64, y: f64)
    {
        let grid_width = self.grid().range_x();
        let grid_height = self.grid().range_y();
        let translation_vector = Cplx::new(grid_width * x, grid_height * y);
        self.pan(translation_vector);
    }

    fn process_tasks(&mut self);

    fn frame_contains_pixel(&self, pointer_pos: Pos2) -> bool
    {
        self.frame().region.contains(pointer_pos)
    }

    fn map_pixel(&self, pointer_pos: Pos2) -> Cplx
    {
        let relative_pos = self.frame().to_local_coords(pointer_pos);
        self.grid().map_pos(relative_pos.into())
    }

    fn process_mouse_input(&mut self, pointer_value: Cplx, zoom_factor: f32, reselect_point: bool)
    {
        if (zoom_factor - 1.0).abs() > f32::EPSILON {
            self.zoom((1. / zoom_factor).into(), pointer_value);
        }

        if reselect_point {
            self.select_point(pointer_value);
        }
    }

    fn select_preperiod_smooth_coloring(&mut self);
    fn select_preperiod_period_smooth_coloring(&mut self);
    fn select_preperiod_coloring(&mut self);

    fn marking(&self) -> &Marking;
    fn marking_mut(&mut self) -> &mut Marking;

    fn cycle_active_plane(&mut self);
    fn change_compute_mode(&mut self, change: ChangeBoolean);

    fn scale_max_iter(&mut self, factor: f64);

    fn save_image(&mut self, img_width: usize, filename: &Path);
    fn save_palette(&mut self, filename: &Path);
    fn load_palette(&mut self, filename: &Path);

    fn change_height(&mut self, new_height: usize);

    fn state_info(&self) -> String;
    fn pop_child_task(&mut self) -> ChildTask;
}

/// `WindowPane` is a struct that represents a window pane in the GUI.
/// It holds the plane being displayed, the coloring information, the image frame,
/// tasks for computation and drawing, and other state related to the dynamical system.
///
/// # Type Parameters
///
/// * `P`: The type of the plane being displayed, which must implement the `dynamo_core::dynamics::Displayable` trait.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub(super) struct WindowPane<P>
where
    P: Displayable,
{
    pub plane: P,
    pub coloring: Coloring,
    iter_plane: IterPlane<P::Deriv>,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub image_frame: ImageFrame,
    tasks: PaneTasks,
    selection: Cplx,
    #[cfg_attr(feature = "serde", serde(skip))]
    orbit_info: Option<orbit::Info<P::Param, P::Var, P::Deriv>>,
    pub marking: Marking,
    pub zoom_factor: Real,
    pub child_task: ChildTask,
}
impl<P> WindowPane<P>
where
    P: Displayable + 'static,
{
    /// Change the meta-parameter for the plane. Returns true if the new value is distinct from the
    /// old one.
    /// Sets a new parameter for the plane and updates the state accordingly.
    /// Clears marked orbits and equipotentials if the ray state is idle.
    /// Returns true if the new parameter is different from the current one.
    ///
    /// # Arguments
    ///
    /// * `new_param`: The new parameter to set for the plane.
    ///
    /// # Returns
    ///
    /// `bool`: Indicates whether the parameter was updated (true) or not (false).
    pub fn set_param(&mut self, new_param: <P::MetaParam as ParamList>::Param) -> bool
    {
        let old_param = self.plane.get_param();

        let update: bool = if old_param == new_param {
            false
        } else {
            self.plane.set_param(new_param);
            if matches!(self.tasks().follow, FollowState::Idle) {
                self.select_point(self.plane.default_selection());
            }
            self.schedule_recompute();
            true
        };

        self.clear_equipotentials();

        update
    }

    /// Creates a new `WindowPane` with the given plane and coloring.
    /// Initializes the image frame, tasks, selection, and marking based on the plane type.
    ///
    /// # Arguments
    ///
    /// * `plane`: The plane to be displayed in the window pane.
    /// * `coloring`: The coloring information for rendering the plane.
    ///
    /// # Returns
    ///
    /// `WindowPane<P>`: A new instance of `WindowPane` with the specified plane and coloring.
    #[must_use]
    pub fn new(plane: P, coloring: Coloring) -> Self
    {
        let iter_plane = IterPlane::create(plane.point_grid().clone());
        let selection = plane.default_selection();
        let frame = ImageFrame::default();

        let degree = plane.degree_real().try_round().unwrap_or(2);
        let mut marking = Marking::default().with_degree(degree);

        if plane.plane_type().is_dynamical() {
            marking.toggle_critical();
        }

        Self {
            plane,
            coloring,
            iter_plane,
            image_frame: frame,
            tasks: PaneTasks::init_tasks(),
            selection,
            orbit_info: None,
            marking,
            zoom_factor: 1.,
            child_task: ChildTask::Idle,
        }
    }

    #[inline]
    const fn plane(&self) -> &P
    {
        &self.plane
    }
    #[inline]
    fn plane_mut(&mut self) -> &mut P
    {
        &mut self.plane
    }

    #[inline]
    const fn get_orbit_info(&self) -> &Option<orbit::Info<P::Param, P::Var, P::Deriv>>
    {
        &self.orbit_info
    }
    #[inline]
    fn get_orbit_info_mut(&mut self) -> &mut Option<orbit::Info<P::Param, P::Var, P::Deriv>>
    {
        &mut self.orbit_info
    }
    #[inline]
    fn set_orbit_info(&mut self, info: orbit::Info<P::Param, P::Var, P::Deriv>)
    {
        self.orbit_info = Some(info);
    }
    #[inline]
    fn del_orbit_info(&mut self)
    {
        self.orbit_info = None;
    }

    #[inline]
    fn select_ray_landing_point_now(&mut self, angle: RationalAngle)
    {
        if let Some(approx_landing_point) = self.marking().ray_landing_point(angle) {
            self.select_point(approx_landing_point);
        }
    }

    fn describe_max_iter(&self) -> String
    {
        format!("Max iterations: {n}", n = self.plane.max_iter())
    }

    fn describe_selection(&self) -> String
    {
        let conf = self.plane.orbit_summary_conf();
        self.selection
            .describe(&conf.selection_conf())
            .map_or_else(String::new, |description| {
                format!("Selection: {description}")
            })
    }

    fn describe_orbit_info(&self) -> String
    {
        let conf = self.plane.orbit_summary_conf();
        self.get_orbit_info()
            .as_ref()
            .map_or_else(String::new, |info| info.summary(&conf))
    }

    #[inline]
    fn process_marking_tasks(&mut self)
    {
        let period_coloring = self.coloring.get_period_coloring();
        self.marking
            .process_all_tasks(&self.plane, self.selection, period_coloring);
    }

    fn draw(&mut self)
    {
        let image = self.iter_plane.render(self.get_coloring());
        let image_frame = self.frame_mut();
        image_frame.image = image;
        image_frame.update_texture();
    }

    fn redraw(&mut self)
    {
        let coloring = self.coloring.clone();
        self.iter_plane
            .render_into(&mut self.image_frame.image, &coloring);
        self.image_frame.update_texture();
    }

    #[inline]
    fn compute(&mut self)
    {
        self.iter_plane = self.plane.compute();
    }

    #[inline]
    fn recompute(&mut self)
    {
        self.plane.compute_into(&mut self.iter_plane);
    }

    fn mark_orbit_and_info(&mut self, pointer_value: Cplx)
    {
        let orbit::OrbitAndInfo { orbit, info } = self.plane.get_orbit_and_info(pointer_value);
        let orbit_pts = orbit.iter().map(|x| (*x).into()).collect();
        self.mark_orbit(orbit_pts, Color32::GREEN);
        self.set_orbit_info(info);
    }

    fn mark_orbit(&mut self, zs: ComplexVec, color: Color32)
    {
        self.marking.mark_orbit_manually(zs, color);
    }

    fn schedule_recompute_keep_old_annotations(&mut self)
    {
        self.tasks_mut().compute.schedule_rerun();
        self.tasks_mut().draw.schedule_rerun();
        self.marking_mut().flush_path_cache();
    }
}

impl<P> From<P> for WindowPane<P>
where
    P: Displayable + 'static,
{
    fn from(plane: P) -> Self
    {
        let coloring = plane.default_coloring();
        Self::new(plane, coloring)
    }
}

/// Implementation of the `Pane` trait for `WindowPane`.
/// Provides methods for interacting with the pane, such as computing and drawing the fractal,
/// handling tasks, zooming, panning, and managing selections and markings.
impl<P> Pane for WindowPane<P>
where
    P: Displayable + 'static,
{
    #[inline]
    fn tasks(&self) -> &PaneTasks
    {
        &self.tasks
    }
    #[inline]
    fn tasks_mut(&mut self) -> &mut PaneTasks
    {
        &mut self.tasks
    }
    #[inline]
    fn grid(&self) -> &PointGrid
    {
        self.plane.point_grid()
    }
    #[inline]
    fn grid_mut(&mut self) -> &mut PointGrid
    {
        self.plane.point_grid_mut()
    }
    #[inline]
    fn frame(&self) -> &ImageFrame
    {
        &self.image_frame
    }
    #[inline]
    fn frame_mut(&mut self) -> &mut ImageFrame
    {
        &mut self.image_frame
    }
    #[inline]
    fn get_coloring(&self) -> &Coloring
    {
        &self.coloring
    }
    #[inline]
    fn get_coloring_mut(&mut self) -> &mut Coloring
    {
        &mut self.coloring
    }
    #[inline]
    fn get_image_frame(&self) -> &ImageFrame
    {
        &self.image_frame
    }
    #[inline]
    fn get_image_frame_mut(&mut self) -> &mut ImageFrame
    {
        &mut self.image_frame
    }
    #[inline]
    fn get_selection(&self) -> Cplx
    {
        self.selection
    }
    #[inline]
    fn select_point(&mut self, point: Cplx)
    {
        if self.selection != point {
            self.selection = point;
            self.marking.select_point(point);
            self.child_task = ChildTask::UpdateParam;
            self.schedule_redraw();
        }
    }
    #[inline]
    fn reset_selection(&mut self)
    {
        self.select_point(self.plane.default_selection());
        self.clear_marked_orbit();
        self.stop_following();
    }
    #[inline]
    fn reset(&mut self)
    {
        let bounds = self.plane.default_bounds();
        self.grid_mut().change_bounds(bounds);
        self.zoom_factor = 1.;
        self.reset_selection();
        self.clear_marked_orbit();
        self.schedule_recompute();
    }
    #[inline]
    fn stop_following(&mut self)
    {
        self.tasks_mut().follow = FollowState::Idle;
    }
    #[inline]
    fn set_follow_state(&mut self, follow_state: FollowState)
    {
        self.tasks_mut().follow = follow_state;
    }

    #[inline]
    fn degree(&self) -> AngleNum
    {
        self.plane().degree()
    }

    #[inline]
    fn draw_contour(&mut self, contour_type: ContourType)
    {
        let selection = self.get_selection();

        self.marking_mut().toggle_contour(contour_type, selection);
    }
    fn draw_aux_contours(&mut self)
    {
        let selection = self.get_selection();
        // let Some((mu, dmu)) = self.plane().auxiliary_value(selection) else {
        //     return;
        // };
        //
        // let val = mu.norm().ln();
        // let dval = dmu.norm() / val;

        for i in -8..=10 {
            // let target = dval.mul_add(Real::from(i) * 1e-1, val);
            let target = Real::from(i) / 2.0;

            self.marking_mut()
                .toggle_contour(ContourType::multiplier(target), selection);
        }
    }

    #[inline]
    fn marking(&self) -> &Marking
    {
        &self.marking
    }
    #[inline]
    fn marking_mut(&mut self) -> &mut Marking
    {
        &mut self.marking
    }

    fn select_nearby_point(&mut self, o: OrbitSchema) -> FindPointResult<Cplx>
    {
        self.plane
            .find_nearby_preperiodic_point(self.selection, o)
            .map(|pt| {
                self.select_point(pt);
                pt
            })
    }

    fn map_selection(&mut self)
    {
        if self.plane_type().is_dynamical() {
            let c = self.plane.param_map(self.selection);
            let mut z = self.plane.start_point(self.selection, &c);
            z = self.plane.map(z, &c);
            self.select_point(z.into());
        }
    }

    #[inline]
    fn cycle_active_plane(&mut self)
    {
        self.plane.cycle_active_plane();
        self.schedule_recompute();
        self.schedule_redraw();
    }

    fn change_compute_mode(&mut self, change: ChangeBoolean)
    {
        match change {
            ChangeBoolean::Enable => {
                self.plane.set_compute_mode(ComputeMode::DistanceEstimation);
            }
            ChangeBoolean::Disable => {
                self.plane.set_compute_mode(ComputeMode::SmoothPotential);
            }
            ChangeBoolean::Toggle => {
                self.plane.compute_mode_mut().cycle();
            }
        }
        self.schedule_recompute();
    }

    fn scale_max_iter(&mut self, factor: f64)
    {
        let iters = self.plane.max_iter_mut();
        *iters = ((*iters as f64) * factor) as IterCount;
        self.schedule_recompute();
        self.schedule_redraw();
    }

    fn change_height(&mut self, new_height: usize)
    {
        self.plane.point_grid_mut().resize_y(new_height);
        self.schedule_compute();
    }

    #[inline]
    fn zoom(&mut self, scale: Real, base_point: Cplx)
    {
        self.zoom_factor *= scale;
        self.grid_mut().zoom(scale, base_point);
        self.schedule_recompute_keep_old_annotations();
    }

    fn process_tasks(&mut self)
    {
        self.process_marking_tasks();

        match self.tasks_mut().follow.pop() {
            FollowState::Idle => {}
            FollowState::SelectRay { angle, follow } => {
                self.select_ray_landing_point_now(angle);
                if !follow {
                    self.stop_following();
                }
            }
            FollowState::SelectPeriodic {
                orbit_schema,
                follow,
            } => {
                if self.select_nearby_point(orbit_schema).is_err() {
                    // Don't draw orbit if we failed to converge
                    self.tasks_mut().orbit.skip();
                }
                if !follow {
                    self.stop_following();
                }
            }
        }

        if self.tasks_mut().orbit.pop() {
            self.mark_orbit_and_info(self.selection);
        } else {
            self.orbit_info = None;
        }

        match self.tasks_mut().compute.pop() {
            RepeatableTask::Rerun => {
                self.recompute();
            }
            RepeatableTask::DoNothing => {}
            RepeatableTask::InitRun => {
                self.compute();
            }
        }
        match self.tasks_mut().draw.pop() {
            RepeatableTask::Rerun => {
                self.redraw();
            }
            RepeatableTask::DoNothing => {}
            RepeatableTask::InitRun => {
                self.draw();
            }
        }
    }

    fn select_preperiod_smooth_coloring(&mut self)
    {
        let coloring_algorithm = self.plane.internal_potential_coloring();
        self.set_coloring_algorithm(coloring_algorithm);
    }

    fn select_preperiod_period_smooth_coloring(&mut self)
    {
        let coloring_algorithm = self.plane.potential_and_period_coloring();
        self.set_coloring_algorithm(coloring_algorithm);
    }

    fn select_preperiod_coloring(&mut self)
    {
        let coloring_algorithm = self.plane.preperiod_coloring();
        self.set_coloring_algorithm(coloring_algorithm);
    }

    fn save_image(&mut self, img_width: usize, filename: &Path)
    {
        let old_res_x = self.plane.point_grid().res_x;
        self.plane.point_grid_mut().resize_x(img_width);
        let iter_plane = self.plane.compute();
        // iter_plane.save(self.get_coloring(), filename.to_owned());

        let mut image = iter_plane.write_image(self.get_coloring());
        self.marking.mark_image(self.grid(), &mut image);

        if let Err(e) = image.save(filename) {
            println!("Error saving file: {e:?}");
        } else {
            println!("Image saved to {}", filename.to_string_lossy());
        }

        self.plane.point_grid_mut().resize_x(old_res_x);
    }

    fn save_palette(&mut self, filename: &Path)
    {
        if let Err(e) = self.coloring.save_to_file(filename) {
            println!("Error saving palette: {e:?}");
        } else {
            println!("Palette saved to {}", filename.to_string_lossy());
        }
    }

    fn load_palette(&mut self, filename: &Path)
    {
        if let Err(e) = self.coloring.load_palette(filename) {
            println!("Error loading palette: {e:?}");
        }
        self.schedule_redraw();
    }

    #[inline]
    fn clear_marked_orbit(&mut self)
    {
        self.marking.disable_orbit();
        self.tasks_mut().orbit.disable();
    }

    #[inline]
    fn clear_marked_rays(&mut self)
    {
        self.marking.disable_all_rays();
    }

    fn clear_equipotentials(&mut self)
    {
        self.marking.disable_all_contours();
    }

    fn clear_curves(&mut self)
    {
        self.marking.disable_all_curves();
        self.tasks_mut().orbit.disable();
    }

    fn put_marked_curves(&self, ui: &mut Ui)
    {
        let frame = self.frame();
        // let grid = self.grid();
        let painter = ui.painter().with_clip_rect(frame.region);

        self.marking()
            .draw_curves(&painter, self.grid(), self.frame());
    }

    fn clear_marked_points(&mut self)
    {
        self.marking.disable_all_points();
    }

    fn put_marked_points(&self, ui: &mut Ui)
    {
        let frame = self.frame();
        let grid = self.grid();
        let painter = ui.painter().with_clip_rect(frame.region);
        self.marking.draw_points(&painter, grid, frame);
    }

    fn state_info(&self) -> String
    {
        format!(
            "{iters_info}\n{selection_info}\n{orbit_info}\n\n{follow_state}",
            iters_info = self.describe_max_iter(),
            selection_info = self.describe_selection(),
            orbit_info = self.describe_orbit_info(),
            follow_state = self.tasks().follow,
        )
    }

    fn pop_child_task(&mut self) -> ChildTask
    {
        let res = self.child_task;
        self.child_task = ChildTask::Idle;
        res
    }

    fn plane_type(&self) -> PlaneType
    {
        self.plane.plane_type()
    }

    fn name(&self) -> String
    {
        self.plane.name()
    }

    fn long_name(&self) -> String
    {
        self.plane.name()
    }
}
