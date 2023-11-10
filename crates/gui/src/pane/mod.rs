use egui::{Color32, Pos2, Ui};
use std::path::Path;

use super::image_frame::ImageFrame;
use super::marked_points::Marking;
use dynamo_color::prelude::*;
use dynamo_common::prelude::*;
use dynamo_core::error::FindPointResult;
use dynamo_core::prelude::*;

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
    fn select_ray_landing_point(&mut self, angle: RationalAngle);
    fn map_selection(&mut self);
    fn follow_ray_landing_point(&mut self, angle: RationalAngle);
    fn stop_following_ray_landing_point(&mut self);
    fn ray_state(&self) -> RayState;
    fn degree(&self) -> AngleNum;

    fn draw_equipotential(&mut self);

    fn get_image_frame(&self) -> &ImageFrame;
    fn get_image_frame_mut(&mut self) -> &mut ImageFrame;

    fn mark_orbit(&mut self, zs: ComplexVec, color: Color32);
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

    fn rescale(&mut self, new_bounds: Bounds)
    {
        self.grid_mut().change_bounds(new_bounds);
        self.schedule_compute();
    }

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

    fn compute(&mut self);

    fn recompute(&mut self);

    fn draw(&mut self);
    fn redraw(&mut self);

    fn process_marking_tasks(&mut self);

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

    fn process_tasks(&mut self)
    {
        self.process_marking_tasks();

        let tasks = self.tasks_mut().pop();
        match tasks.compute {
            RepeatableTask::Rerun => {
                self.recompute();
            }
            RepeatableTask::DoNothing => {}
            RepeatableTask::InitRun => {
                self.compute();
            }
        }
        match tasks.draw {
            RepeatableTask::Rerun => {
                self.redraw();
            }
            RepeatableTask::DoNothing => {}
            RepeatableTask::InitRun => {
                self.draw();
            }
        }
    }

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

    fn scale_max_iter(&mut self, factor: f64);

    fn save_image(&mut self, img_width: usize, filename: &Path);
    fn save_palette(&mut self, filename: &Path);
    fn load_palette(&mut self, filename: &Path);

    fn change_height(&mut self, new_height: usize);

    fn mark_orbit_and_info(&mut self, pointer_value: Cplx);
    fn describe_selection(&self) -> String;
    fn describe_orbit_info(&self) -> String;
    fn pop_child_task(&mut self) -> ChildTask;
}

/// `WindowPane` is a struct that represents a window pane in the GUI.
/// It holds the plane being displayed, the coloring information, the image frame,
/// tasks for computation and drawing, and other state related to the dynamical system.
///
/// # Type Parameters
///
/// * `P`: The type of the plane being displayed, which must implement the `dynamo_core::dynamics::Displayable` trait.
pub(super) struct WindowPane<P>
where
    P: Displayable,
{
    pub plane: P,
    pub coloring: Coloring,
    iter_plane: IterPlane<P::Deriv>,
    pub image_frame: ImageFrame,
    tasks: PaneTasks,
    selection: Cplx,
    orbit_info: Option<orbit::Info<P::Param, P::Var, P::Deriv>>,
    pub marking: Marking,
    pub zoom_factor: Real,
    pub ray_state: RayState,
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
            if matches!(self.ray_state, RayState::Idle) {
                self.select_point(self.plane.default_selection());
            }
            self.schedule_recompute();
            self.schedule_redraw();
            true
        };

        self.clear_marked_orbit();
        self.clear_equipotentials();

        if let RayState::Following(angle) = self.ray_state() {
            self.select_ray_landing_point_now(angle);
        }

        update
    }

    #[must_use]
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
            ray_state: RayState::Idle,
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
    fn select_point_keep_following(&mut self, point: Cplx)
    {
        if self.selection != point {
            self.selection = point;
            self.marking.select_point(point);
            self.child_task = ChildTask::UpdateParam;
            self.schedule_redraw();
        }
    }

    #[inline]
    fn select_ray_landing_point_now(&mut self, angle: RationalAngle)
    {
        if let Some(approx_landing_point) = self.marking().ray_landing_point(angle) {
            self.select_point_keep_following(approx_landing_point);
        }
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
    fn reset_selection(&mut self)
    {
        self.select_point(self.plane.default_selection());
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
    fn stop_following_ray_landing_point(&mut self)
    {
        self.ray_state = RayState::Idle;
    }
    #[inline]
    fn ray_state(&self) -> RayState
    {
        self.ray_state
    }

    #[inline]
    fn degree(&self) -> AngleNum
    {
        self.plane().degree()
    }

    #[inline]
    fn draw_equipotential(&mut self)
    {
        let selection = self.get_selection();
        self.marking_mut().toggle_equipotential(selection);
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
    #[inline]
    fn select_point(&mut self, point: Cplx)
    {
        self.select_point_keep_following(point);
        self.stop_following_ray_landing_point();
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
            let mut z = self.plane.start_point(self.selection, c);
            z = self.plane.map(z, c);
            self.select_point(z.into());
        }
    }

    fn follow_ray_landing_point(&mut self, angle: RationalAngle)
    {
        self.ray_state = RayState::Following(angle);
    }

    fn select_ray_landing_point(&mut self, angle: RationalAngle)
    {
        self.ray_state = RayState::SelectOnce(angle);
    }

    #[inline]
    fn cycle_active_plane(&mut self)
    {
        self.plane.cycle_active_plane();
        self.schedule_recompute();
        self.schedule_redraw();
    }

    fn scale_max_iter(&mut self, factor: f64)
    {
        let iters = self.plane.max_iter_mut();
        *iters = ((*iters as f64) * factor) as Period;
        self.schedule_recompute();
        self.schedule_redraw();
    }

    #[inline]
    fn draw(&mut self)
    {
        let image = self.iter_plane.render(self.get_coloring());
        let image_frame = self.frame_mut();
        image_frame.image = image;
        image_frame.update_texture();
    }

    #[inline]
    fn redraw(&mut self)
    {
        let coloring = self.coloring.clone();
        self.iter_plane
            .render_into(&mut self.image_frame.image, &coloring);
        self.image_frame.update_texture();
    }
    fn process_marking_tasks(&mut self)
    {
        let period_coloring = self.coloring.get_period_coloring();
        self.marking
            .process_all_tasks(&self.plane, self.selection, period_coloring);
        match self.ray_state {
            RayState::SelectOnce(angle) => {
                self.select_ray_landing_point_now(angle);
                self.ray_state = RayState::Idle;
            }
            RayState::Following(angle) => {
                self.select_ray_landing_point_now(angle);
            }
            RayState::Idle => {}
        }
    }

    fn change_height(&mut self, new_height: usize)
    {
        self.plane.point_grid_mut().resize_y(new_height);
        self.schedule_compute();
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

    #[inline]
    fn zoom(&mut self, scale: Real, base_point: Cplx)
    {
        self.zoom_factor *= scale;
        self.grid_mut().zoom(scale, base_point);
        self.schedule_recompute();
        self.schedule_redraw();
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

    fn clear_marked_orbit(&mut self)
    {
        self.marking.disable_orbit();
    }

    fn clear_marked_rays(&mut self)
    {
        self.marking.disable_all_rays();
    }

    fn clear_equipotentials(&mut self)
    {
        self.marking.disable_all_equipotentials();
    }

    fn clear_curves(&mut self)
    {
        self.marking.disable_all_curves();
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
