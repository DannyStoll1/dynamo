use dynamo_common::coloring::*;
use dynamo_common::math_utils::newton::error::{Error::NanEncountered, NewtonResult};
use dynamo_common::math_utils::{arithmetic::*, newton::*};
use dynamo_common::prelude::*;
use dynamo_common::symbolic_dynamics::OrbitSchema;
use ndarray::{Array2, Axis};
use num_cpus;
use num_traits::{NumOps, One, Zero};
use rayon::iter::{ParallelBridge, ParallelIterator};
use std::{cell::RefCell, f64::consts::TAU};
use std::{fmt::Display, ops::AddAssign};
use thread_local::ThreadLocal;

pub mod covering_maps;
pub mod error;
pub mod julia;
pub mod newton;
pub mod orbit;
// pub mod simple_parameter_plane;
// pub mod functions;

use julia::JuliaSet;
use orbit::{CycleDetectedOrbitFloyd, SimpleOrbit};
use std::ops::{Add, Mul, MulAssign, Sub};

use self::error::{FindPointError, FindPointResult};
use self::orbit::OrbitParams;
// pub use simple_parameter_plane::SimpleParameterPlane;

pub trait Variable:
    Norm<Real>
    + Dist<Real>
    + Sub<Output = Self>
    + MaybeNan
    + Clone
    + Send
    + Default
    + From<Cplx>
    + Into<Cplx>
    + Display
{
}
pub trait Parameter:
    From<Cplx> + Clone + Copy + Send + Sync + Default + PartialEq + Summarize + std::fmt::Debug
{
}
pub trait Derivative:
    Polar<Real>
    + Send
    + Default
    + Zero
    + One
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + AddAssign
    + MulAssign
    + Display
    + Into<Cplx>
{
}

impl<V> Variable for V where
    V: Norm<Real>
        + Dist<Real>
        + Sub<Output = Self>
        + MaybeNan
        + Clone
        + Send
        + Default
        + From<Cplx>
        + Into<Cplx>
        + Display
{
}
impl<P> Parameter for P where
    P: From<Cplx> + Clone + Copy + Send + Sync + Default + PartialEq + Summarize + std::fmt::Debug
{
}
impl<D> Derivative for D where
    D: Polar<Real>
        + Send
        + Default
        + Zero
        + One
        + Add<Output = Self>
        + Sub<Output = Self>
        + Mul<Output = Self>
        + AddAssign
        + MulAssign
        + Display
        + Into<Cplx>
{
}

pub trait ParameterPlane: Sync + Send
{
    type Var: Variable;
    type Param: Parameter;
    type MetaParam: ParamList + Clone + Copy + Send + Sync + Default + Summarize;
    type Deriv: Derivative;
    type Child: ParameterPlane;

    fn point_grid(&self) -> &PointGrid;
    fn point_grid_mut(&mut self) -> &mut PointGrid;

    #[must_use]
    fn with_point_grid(self, point_grid: PointGrid) -> Self;

    #[must_use]
    fn with_bounds(self, bounds: Bounds) -> Self
    where
        Self: Sized,
    {
        let point_grid = self.point_grid().new_with_same_height(bounds);
        self.with_point_grid(point_grid)
    }

    /// Modify and return self with a different image height, and with width scaled to preserve aspect ratio
    #[must_use]
    fn with_res_y(mut self, res_y: usize) -> Self
    where
        Self: Sized,
    {
        self.point_grid_mut().resize_y(res_y);
        self
    }

    /// Modify and return self with a different image width, and with height scaled to preserve aspect ratio
    #[must_use]
    fn with_res_x(mut self, res_x: usize) -> Self
    where
        Self: Sized,
    {
        self.point_grid_mut().resize_x(res_x);
        self
    }

    fn max_iter(&self) -> Period;
    fn max_iter_mut(&mut self) -> &mut Period;
    fn set_max_iter(&mut self, new_max_iter: Period);

    #[must_use]
    fn with_max_iter(self, max_iter: Period) -> Self;

    fn name(&self) -> String;
    fn description(&self) -> String
    {
        String::new()
    }

    /// The map defining the dynamical system.
    fn map(&self, z: Self::Var, c: Self::Param) -> Self::Var;

    /// Derivative of the map with respect to the dynamical variable. Used for smooth coloration.
    fn dynamical_derivative(&self, z: Self::Var, c: Self::Param) -> Self::Deriv;

    /// Derivative of the map with respect to the paraameter. Used for external rays in parameter
    /// planes.
    fn parameter_derivative(&self, z: Self::Var, c: Self::Param) -> Self::Deriv;

    /// The dynamical map, together with its derivative. This is the primary computational
    /// bottleneck, and should usually be implemented manually for optimization purposes.
    fn map_and_multiplier(&self, z: Self::Var, c: Self::Param) -> (Self::Var, Self::Deriv)
    {
        (self.map(z, c), self.dynamical_derivative(z, c))
    }

    /// The dynamical map, together with its derivative and parameter derivative. Used to compute
    /// external rays in parameter planes.
    fn gradient(&self, z: Self::Var, c: Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        let (fz, df_dz) = self.map_and_multiplier(z, c);
        (fz, df_dz, self.parameter_derivative(z, c))
    }

    /// If certain regions in parameter space are known (e.g. the main cardioid in the Mandelbrot set), we can
    /// avoid having to compute orbits for parameters in those regions.
    ///
    /// This function returns an `EscapeState`, depending on the starting point and parameter.
    /// It is called once before computing each orbit.
    /// If this function returns `EscapeState::NotYetEscaped`, then the orbit is computed;
    /// otherwise, the output of this function is forwarded to `encode_escape_result`.
    fn early_bailout(
        &self,
        _start: Self::Var,
        _c: Self::Param,
    ) -> EscapeState<Self::Var, Self::Deriv>
    {
        EscapeState::NotYetEscaped
    }

    /// Minimum iterations before cycle detection is allowed.
    ///
    /// Useful for dynamical families with many parabolic systems, such as Cubic Per(1,1),
    /// in which orbits on the repelling side of the parabolic cylinder will remain
    /// near-periodic for a long time even if they will eventually escape.
    /// For such families, it is recommended to set `min_iter` to some constant fraction of
    /// `self.max_iter()`.
    #[inline]
    fn min_iter(&self) -> Period
    {
        0
    }

    /// Upper bound on the norm-squared of the dynamical variable,
    /// beyond which an orbit is considered to have escaped.
    ///
    /// Only relevant for maps with an attracting or parabolic cycle containing infinity. In the
    /// latter case, it is recommended to set escape_radius to a much smaller value.
    #[inline]
    fn escape_radius(&self) -> Real
    {
        1e12
    }

    /// Lower bound on distance-squared between fast and slow orbits. If the fast and slow
    /// variables are closer than this bound, then orbit computation teminates, and a cycle is
    /// detected.
    ///
    /// This value can be raised to save computational time, but this will increase the rate of
    /// false positives. Conversely, it can be lowered to increase accuracy, at the cost of needing
    /// more iterations to detect cycles.
    ///
    /// For dynamical families with many parabolic systems, such as Cubic Per(1,1),
    /// it is recommended to set this value to something much larger (e.g. 1e-6),
    /// since orbits take very long to converge toward parabolic cycles. This will lead to false
    /// positives, which can be mitigated by increasing `self.min_iter`.
    ///
    /// This value can be set dynamically, for instance to shrink the radius as the image is zoomed
    /// in. This is done in the default implementation.
    ///
    /// Setting this value to 0 disables cycle detection.
    #[inline]
    fn periodicity_tolerance(&self) -> Real
    {
        self.point_grid().bounds.area() * 1e-14
    }

    /// The starting value for the dynamical variable. Depends on two parameters: the raw point in
    /// the image that is being computed, and the parameter value. Generally, for parameter planes,
    /// `start_point` depends only on the parameter, and for dynamical planes, `start_point` depends
    /// only on the point.
    ///
    /// For Julia sets, `start_point` is computed by applying `self.dynam_map` to the raw point.
    /// For parameter planes, `start_point` needs to be implemented manually.
    fn start_point(&self, _point: Cplx, c: Self::Param) -> Self::Var;

    /// Start point, its partial derivative with respect to the point,
    /// and its partial derivative with respect to the parameter
    fn start_point_d(&self, point: Cplx, c: Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        (
            self.start_point(point, c),
            Self::Deriv::zero(),
            Self::Deriv::zero(),
        )
    }

    /// Map points in the image to parameters. Used for multi-parameter systems or covering maps
    /// over existing parameter planes.
    #[inline]
    fn param_map(&self, point: Cplx) -> Self::Param
    {
        point.into()
    }

    /// param_map together with its derivative.
    /// TODO: implement this correctly
    #[inline]
    fn param_map_d(&self, point: Cplx) -> (Self::Param, Self::Deriv)
    {
        (point.into(), Self::Deriv::one())
    }

    /// Map points in the image to dynamical variables. Used for multivariable systems or covering maps
    /// over existing dynamical planes.
    ///
    /// Currently, this is only called in the implementation for `start_point` in Julia sets.
    #[inline]
    fn dynam_map(&self, point: Cplx) -> Self::Var
    {
        point.into()
    }

    #[inline]
    fn dynam_map_d(&self, point: Cplx) -> (Self::Var, Self::Deriv)
    {
        (point.into(), Self::Deriv::one())
    }


    #[inline]
    fn get_meta_params(&self) -> Self::MetaParam
    {
        Self::MetaParam::default()
    }

    #[inline]
    fn get_param(&self) -> <Self::MetaParam as ParamList>::Param
    {
        <Self::MetaParam as ParamList>::Param::default()
    }

    #[inline]
    fn set_meta_param(&mut self, _value: Self::MetaParam) {}

    #[inline]
    fn set_param(&mut self, _value: <Self::MetaParam as ParamList>::Param) {}

    #[inline]
    #[must_use]
    fn with_param(mut self, param: <Self::MetaParam as ParamList>::Param) -> Self
    where
        Self: Sized,
    {
        self.set_param(param);
        self
    }

    /// Critical points of the map associated to a given parameter, which can be marked on the dynamical plane.
    #[inline]
    fn critical_points_child(&self, _c: Self::Param) -> Vec<Self::Var>
    {
        vec![]
    }

    /// Critical points of the map, if the plane is dynamical.
    #[inline]
    fn critical_points(&self) -> Vec<Self::Var>
    {
        vec![]
    }

    /// Implementation of `cycles` for Julia sets spawned from this parameter plane.
    /// Used to mark selected periodic points on the dynamical plane.
    #[inline]
    fn cycles_child(&self, _c: Self::Param, _period: Period) -> Vec<Self::Var>
    {
        vec![]
    }

    /// Implementation of `precycles` for Julia sets spawned from this parameter plane.
    /// Used to mark selected preperiodic points on the dynamical plane.
    #[inline]
    fn precycles_child(&self, _c: Self::Param, _orbit_schema: OrbitSchema) -> Vec<Self::Var>
    {
        vec![]
    }

    /// Parameter values known to yield periodic cycles of a given period.
    /// These are drawn on the parameter plane despite having type `Self::Var`, since `Self::Param`
    /// doesn't always implement `Into<Cplx>`. This only produces the correct result if `param_map`
    /// is the identity.
    /// FIXME: enforce types correctly here. This involves inverting the `param_map` to convert a
    /// parameter back to a complex number.
    ///
    /// Generally used to mark post-critically finite parameters or centers of hyperbolic
    /// components, or in Julia sets to mark periodic points.
    #[inline]
    fn cycles(&self, _period: Period) -> Vec<Self::Var>
    {
        vec![]
    }

    /// Parameter values known to yield preperiodic orbits of a given preperiod and period.
    /// These are drawn on the parameter plane despite having type `Self::Var`, since `Self::Param`
    /// doesn't always implement `Into<Cplx>`. This only produces the correct result if `param_map`
    /// is the identity.
    /// FIXME: enforce types correctly here. This involves inverting the `param_map` to convert a
    /// parameter back to a complex number.
    ///
    /// Generally used to mark Misiurewicz points, or in Julia sets to mark preperiodic points.
    #[inline]
    fn precycles(&self, _orbit_schema: OrbitSchema) -> Vec<Self::Var>
    {
        vec![]
    }

    /// Try to find a (pre)periodic point near a given base point
    fn find_nearby_preperiodic_point(
        &self,
        start_point: Cplx,
        OrbitSchema {
            period: n,
            preperiod: k,
        }: OrbitSchema,
    ) -> FindPointResult<Cplx>
    {
        if n == 0
        {
            return Err(FindPointError::PeriodIsZero);
        }

        // Number of unitary divisors of n
        let num_factors = divisors(n).filter(|d| gcd(n / d, *d) == 1).count();

        // Values and derivatives of (f^{m+k}(z0) - f^k(z0))^(mu(n/m)) for m a unitary divisor of n
        let mut values = vec![ONE; num_factors];
        let mut derivs = vec![ZERO; num_factors];

        let diff = |t| {
            // Initial coordinates
            let (c, dc_dt) = self.param_map_d(t);
            let (mut z, mut dz_dt, dz_dc) = self.start_point_d(t, c);

            // Multivariable chain rule: dz/dt = ∂z/∂t + ∂z/∂c * dc/dt
            dz_dt += dc_dt * dz_dc;

            let mut df_dz: Self::Deriv;
            let mut df_dc: Self::Deriv;

            // f^(k-1)(z) and its derivative with respect to t
            let (mut zk1, mut zk1_dt) = (ZERO, ZERO);

            // If k > 0, these will become 1/(f^(k+n-1) - f^(k-1)(z)) and its derivative with respect to t
            // We initialize them so as to have no effect if k = 0
            let mut early_cycle = ONE;
            let mut early_cycle_dt = ZERO;

            let mut term_count: usize = 0;

            // Preperiodic part
            if k > 0
            {
                for _ in 0..k - 1
                {
                    (z, df_dz, df_dc) = self.gradient(z, c);
                    dz_dt = dz_dt * df_dz + df_dc;
                }

                zk1 = z.into();
                zk1_dt = dz_dt.into();
                (z, df_dz, df_dc) = self.gradient(z, c);
                dz_dt = dz_dt * df_dz + df_dc;
            }

            let mut w = z;
            let mut dw_dt = dz_dt;

            // Periodic part

            for i in 1..n
            {
                (w, df_dz, df_dc) = self.gradient(w, c);
                dw_dt = dw_dt * df_dz + df_dc;

                // Divide out lower order periods
                let (q, r) = n.div_rem(&i);
                if r == 0
                {
                    let mu = moebius(q);
                    if mu == 1
                    {
                        values[term_count] = (w - z).into();
                        derivs[term_count] = (dw_dt - dz_dt).into();
                        term_count += 1;
                    }
                    else if mu == -1
                    {
                        let dg = (dz_dt - dw_dt).into();
                        let val = (w - z).into().inv();
                        values[term_count] = val;
                        derivs[term_count] = dg * val * val;
                        term_count += 1;
                    }
                }
            }

            // At this point we have done k+n-1 iterations
            if k > 0
            {
                // f^(k+n-1)(z) and its derivative with respect to t
                let zkn1 = w.into();
                let zkn1_dt = dw_dt.into();

                // 1/(f^(k+n-1)(z) - f^(k-1)(z)) and its derivative with respect to t
                early_cycle = (zkn1 - zk1).inv();
                early_cycle_dt = early_cycle * early_cycle * (zk1_dt - zkn1_dt);
            }

            // Perform final iteration manually
            (w, df_dz, df_dc) = self.gradient(w, c);
            dw_dt = dw_dt * df_dz + df_dc;

            values[term_count] = (w - z).into();
            derivs[term_count] = (dw_dt - dz_dt).into();

            // Iteratively apply product rule to compute derivative
            let out = values
                .iter()
                .zip(derivs.iter())
                .fold((early_cycle, early_cycle_dt), |(u, du), (v, dv)| {
                    (u * v, u * dv + v * du)
                });
            out
        };

        find_root_newton(diff, start_point).map_err(FindPointError::NewtonError)
    }



    fn run_point(&self, start: Self::Var, c: Self::Param) -> EscapeState<Self::Var, Self::Deriv>
    {
        let orbit_params = OrbitParams {
            max_iter: self.max_iter(),
            min_iter: self.min_iter(),
            periodicity_tolerance: self.periodicity_tolerance(),
            escape_radius: self.escape_radius(),
        };
        let orbit = CycleDetectedOrbitFloyd::new(
            |z, c| self.map(z, c),
            |z, c| self.map_and_multiplier(z, c),
            |z, c| self.early_bailout(z, c),
            start,
            c,
            &orbit_params,
        );
        if let Some((_, state)) = orbit.last()
        {
            state
        }
        else
        {
            EscapeState::Bounded
        }
    }

    fn iter_orbit(&self, point: Cplx) -> Box<dyn Iterator<Item = Self::Var> + '_>
    {
        let param = self.param_map(point);
        let start = self.start_point(point, param);
        Box::new(
            SimpleOrbit::new(
                |z, c| self.map(z, c),
                start,
                param,
                self.max_iter(),
                self.escape_radius(),
            )
            .map(|(z, _s)| z),
        )
    }

    fn get_orbit_vec(&self, point: Cplx) -> Vec<Self::Var>
    {
        let param = self.param_map(point);
        let start = self.start_point(point, param);
        let orbit = SimpleOrbit::new(
            |z, c| self.map(z, c),
            start,
            param,
            self.max_iter(),
            self.escape_radius(),
        );
        orbit.map(|(z, _s)| z).collect()
    }


    /// Default bounds for this plane
    fn default_bounds(&self) -> Bounds;

    /// Default bounds for Julia sets spawned from this plane. This is only called by Julia sets,
    /// who reference the parent's `default_julia_bounds` in their `default_bounds`
    /// implementations.
    #[inline]
    fn default_julia_bounds(&self, _point: Cplx, _c: Self::Param) -> Bounds
    {
        Bounds::centered_square(2.2)
    }

    /// Point to select when the plane is first created.
    #[inline]
    fn default_selection(&self) -> Cplx
    {
        Cplx::default()
    }

    /// For some families (e.g. maps with multiple free critical points),
    /// there are many possible starting points. In this case, we can maintain
    /// a plane identifier in `self`, which can by cycled at runtime to switch
    /// views.
    ///
    /// By itself, this function simply modifies `self` when the corresponding hotkey is pressed
    /// (default: Ctrl-P).
    ///
    /// To have the desired effect, the implementation of `start_point` needs to
    /// be set up to refer to the plane object's internal plane identifier, and
    /// `cycle_active_plane` needs to be implemented to modify this identifier.
    fn cycle_active_plane(&mut self) {}

    /// Whether or not the plane is considered "dynamical", in the sense that the dynamical map is
    /// independent of the pixel being computed.
    ///
    /// Currently unused.
    #[inline]
    fn plane_type(&self) -> PlaneType
    {
        PlaneType::Parameter
    }

    // fn escape_coeff_d(&self, t: Cplx, c: Self::Param) -> (Cplx, Self::Deriv, Self::Deriv) {
    //     (ONE, ZERO, ZERO)
    // }
    // //
    // /// Scaling factor by which we may conjugate our map to make the first return map at infinity
    // /// monic.
    // ///
    // /// Used for computing external rays.
    // fn monic_conj_d(&self, t: Cplx, c: Self::Param) -> (Cplx, Cplx, Cplx) {
    //     if self.degree() == 1 || self.escape_coeff(c) == ONE {
    //         (ONE, ZERO, ZERO)
    //     } else {
    //         self.escape_coeff(c).powf(
    //         1.0 / (self.degree_real() - 1.0))
    //     }
    // }

    /// Default coloring algorithm to apply when loading the parameter plane.
    fn default_coloring(&self) -> Coloring
    {
        let mut coloring = Coloring::default();
        coloring.set_interior_algorithm(IncoloringAlgorithm::PeriodMultiplier);
        coloring
    }

    /// Attracting periodic points that are specially marked. Used for custom colorings, e.g. to
    /// color Newton parameter planes according to which root the critical orbit converges to.
    #[inline]
    fn get_marked_points(&self, _c: Self::Param) -> Vec<(Self::Var, PointClassId)>
    {
        vec![]
    }

    /// Number of marked point classes.
    #[inline]
    fn num_marked_point_classes(&self) -> usize
    {
        0
    }

    /// Lower bound on norm-squared, below which an orbit is considered to have reached a marked
    /// point.
    #[inline]
    fn marked_point_tolerance(&self) -> Real
    {
        self.periodicity_tolerance()
    }

    /// Internal: Detect if a periodic orbit has landed near a marked point.
    fn identify_marked_points(
        &self,
        c: Self::Param,
        data: PointInfoPeriodic<Self::Var, Self::Deriv>,
    ) -> PointInfo<Self::Var, Self::Deriv>
    {
        let marked_points = self.get_marked_points(c);
        for (zi, class_id) in marked_points.iter()
        {
            if data.value.dist_sqr(*zi) < self.marked_point_tolerance()
            {
                return PointInfo::MarkedPoint {
                    data,
                    class_id: *class_id,
                    num_point_classes: marked_points.len(),
                };
            }
        }
        PointInfo::Periodic { data }
    }

    /// Internal: Since the internal potential coloring algorithm depends on the periodicity
    /// tolerance, we need to obtain it from this trait.
    fn preperiod_smooth_coloring(&self) -> IncoloringAlgorithm
    {
        IncoloringAlgorithm::InternalPotential {
            periodicity_tolerance: self.periodicity_tolerance(),
        }
    }

    /// Internal: Since the period + internal potential coloring algorithm depends on the periodicity
    /// tolerance, we need to obtain it from this trait.
    fn preperiod_period_smooth_coloring(&self) -> IncoloringAlgorithm
    {
        IncoloringAlgorithm::PreperiodPeriodSmooth {
            periodicity_tolerance: self.periodicity_tolerance(),
            fill_rate: 0.04,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PlaneType
{
    #[default]
    Parameter,
    Dynamical,
}
impl PlaneType
{
    pub const fn is_dynamical(&self) -> bool
    {
        matches!(self, Self::Dynamical)
    }
}
impl std::fmt::Display for PlaneType
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self
        {
            Self::Parameter => write!(f, "parameter"),
            Self::Dynamical => write!(f, "dynamical"),
        }
    }
}

pub trait InfinityFirstReturnMap: ParameterPlane {
    /// Order of vanishing of the first return map of $1/f(1/z)$ at $z=0$.
    ///
    /// Should be set to NAN if $f$ has an essential singularity at infinity,
    /// or if infinity is not periodic under $f$.
    /// In such cases, external rays are unsupported (unless manually implemented).
    #[inline]
    fn degree_real(&self) -> Real
    {
        2.0
    }

    /// Order of vanishing of the first return map of $1/f(1/z)$ at $z=0$.
    ///
    /// Should be set to 0 if $f$ has an essential singularity at infinity,
    /// or if infinity is not periodic under $f$.
    /// In such cases, external rays are unsupported (unless manually implemented).
    #[inline]
    fn degree(&self) -> AngleNum
    {
        self.degree_real().try_round().unwrap_or(0)
    }

    /// Period of infinity under $f$. Should be set to 0 if infinity is not periodic.
    ///
    /// Used for computing external rays, for which we use an iterate of the map instead of the map
    /// itself.
    #[inline]
    fn escaping_period(&self) -> Period { 1 }

    /// For very large values of the parameter, how many iterations before the variable
    /// value is large?
    ///
    /// Used for computing external rays, for which we use an iterate of the map instead of the map
    /// itself.
    ///
    /// Almost always 0 or 1.
    #[inline]
    fn escaping_phase(&self) -> Period { 1 }

    /// Argument of f_c^k(z0) for c very large with a given argument,
    /// where k = self.escaping_phase().
    ///
    /// Used to seed initial point for external rays.
    #[inline]
    fn angle_map_large_param(&self, angle: RationalAngle) -> RationalAngle { angle}

    // /// Leading coefficient of the self-return map at infinity,
    // /// together with its derivative.
    // ///
    // /// Used for computing external rays.
    fn escape_coeff_d(&self, _c: Self::Param) -> (Cplx, Cplx) {
        (ONE, ZERO)
    }
}

pub trait ExternalRays : ParameterPlane + InfinityFirstReturnMap {

    fn external_ray_helper(&self, angle: RationalAngle) -> Option<Vec<Cplx>>
    {
        const R: Real = 16.0;
        let escape_radius_log2 = R.log2() * self.degree_real().abs();

        let deg_real = self.degree_real().abs();
        if deg_real.is_nan()
        {
            return None;
        }
        let deg_log2 = deg_real.log2();

        let pixel_width = self.point_grid().pixel_width() * 0.03;
        let error = self.point_grid().res_x as Real * 1e-8;

        // let base_point = escape_radius * angle.to_circle();
        // Arbitrary starting guess that is likely to escape
        let base_point: Cplx = 65.0 * angle.to_circle();
        let mut t_list = vec![];

        // degree of each additional batch of iterations
        let deg = self.degree();

        // Target angle for the composite map at each step.
        // Initialized to value after self.escaping_phase() iterations.
        let mut target_angle = self.angle_map_large_param(angle);

        for k in 0..RAY_DEPTH
        {
            // Relative log2-norms of targets
            // jth target norm = escape_radius^deg^(-j/S)
            // u_j = log2(escape_radius) * deg^(-j/S)
            let us = (0..RAY_SHARPNESS).map(|j| {
                escape_radius_log2
                    * ((-Real::from(j) * deg_log2) / Real::from(RAY_SHARPNESS)).exp2()
            });

            let v = target_angle.to_circle();
            let targets = us.map(|u| u.exp2() * v);

            let mut t_curr = *t_list.last().unwrap_or(&base_point);
            let mut dist: Real;

            let num_iters = k * self.escaping_period() + self.escaping_phase();

            let fk_and_dfk = |t: Cplx| {
                let (c, dc_dt) = self.param_map_d(t);
                let (mut z, mut dz_dt, dz_dc) = self.start_point_d(t, c);
                dz_dt += dz_dc * dc_dt;

                for _i in 0..num_iters
                {
                    let (f, df_dz, df_dc) = self.gradient(z, c);
                    dz_dt = dz_dt * df_dz + df_dc * dc_dt;
                    z = f;
                }

                (z.into(), dz_dt.into())
            };

            for target in targets
            {
                match find_target_newton_err_d(fk_and_dfk, t_curr, target, error)
                {
                    Ok((sol, t_k, d_k)) =>
                    {
                        // dbg!(target, sol);
                        t_curr = sol;

                        if t_curr.is_nan()
                        {
                            return Some(t_list);
                        }

                        t_list.push(t_curr);

                        dist = (2. * t_k.norm() * (t_k.norm()).log(deg_real)) / d_k.norm();
                        if dist < pixel_width
                        {
                            return Some(t_list);
                        }
                    }
                    Err(NanEncountered) =>
                    {
                        return Some(t_list);
                    }
                    _ =>
                    {}
                }
            }
            target_angle *= deg;
        }

        Some(t_list)
    }

    /// Compute an external ray for a given rational angle.
    /// The same implementation would work for any real angle,
    /// but we stick to rationals for compatibility with other modules
    /// and to maintain precision.
    ///
    /// Currently works correctly only for quadratic polynomials.
    fn external_ray(&self, angle: RationalAngle) -> Option<Vec<Cplx>>
    {
        // Remove off the end if distance is increasing,
        // as the helper method may return erroneous values near the end.
        // We use l1 norms to preserve precision.
        if let Some(mut t_list) = self.external_ray_helper(angle)
        {
            let t0 = t_list.last()?;
            let mut t1 = t_list.get(t_list.len() - 2)?;
            let mut t2 = t_list.get(t_list.len() - 3)?;
            let mut dist0 = (t0 - t1).l1_norm();
            let mut dist1 = (t1 - t2).l1_norm();
            while dist0 > dist1
            {
                t_list.pop();
                t1 = t_list.last()?;
                t2 = t_list.get(t_list.len() - 2)?;
                dist0 = dist1;
                dist1 = (t1 - t2).l1_norm();
            }
            Some(t_list)
        }
        else
        {
            None
        }
    }
}

pub trait Equipotential: ParameterPlane {
    /// Compute an equipotential curve through a given point.
    fn equipotential(&self, t0: Cplx) -> Option<Vec<Cplx>>;
}
impl<P> Equipotential for P where P: ParameterPlane + InfinityFirstReturnMap
{
    fn equipotential(&self, t0: Cplx) -> Option<Vec<Cplx>>
    {
        let c0 = self.param_map(t0);
        let z0 = self.start_point(t0, c0);

        // Computation time is exponential in iteration count, so we limit ourselves to 13.
        let max_iter = 13;
        let escape_radius = 30.;
        let theta0 = 0.02;

        let orbit = SimpleOrbit::new(|z, c| self.map(z, c), z0, c0, max_iter, escape_radius);
        let state = orbit.last()?.1;
        let EscapeState::Escaped { iters, final_value } = state
        else
        {
            return None;
        };

        let mut target = final_value.into();

        let compute = |t| {
            let (c, dc_dt) = self.param_map_d(t);
            let (mut z, mut dz_dt, dz_dc) = self.start_point_d(t, c);

            // Multivariable chain rule: dz/dt = ∂z/∂t + ∂z/∂c * dc/dt
            dz_dt += dc_dt * dz_dc;

            let mut df_dz: Self::Deriv;
            let mut df_dc: Self::Deriv;

            for _ in 0..iters
            {
                (z, df_dz, df_dc) = self.gradient(z, c);
                dz_dt = dz_dt * df_dz + df_dc;
            }
            (z.into(), dz_dt.into())
        };

        let num_points = (self.degree_real().powi(iters as i32) / theta0) as usize;
        let rotate = (theta0 * TAUI).exp();

        // let mut result = vec![t0; num_points];
        let mut t = t0;

        let result = std::iter::once(t)
            .chain((0..num_points).map(|_| {
                target *= rotate;
                t = find_target_newton_relative(compute, t, target).unwrap_or(t);
                t
            }))
            .collect();

        Some(result)
    }
}

pub trait EscapeEncoding: ParameterPlane + InfinityFirstReturnMap {
    /// Map temporary `EscapeState` (used in orbit computation) to `PointInfo`, encoding the result of the computation.
    fn encode_escape_result(
        &self,
        state: EscapeState<Self::Var, Self::Deriv>,
        c: Self::Param,
    ) -> PointInfo<Self::Var, Self::Deriv>
    {
        match state
        {
            EscapeState::NotYetEscaped | EscapeState::Bounded => PointInfo::Bounded,
            EscapeState::Periodic { data } => self.identify_marked_points(c, data),
            EscapeState::Escaped { iters, final_value } =>
            {
                self.encode_escaping_point(iters, final_value, c)
            }
        }
    }

    fn encode_escaping_point(
        &self,
        iters: Period,
        z: Self::Var,
        _c: Self::Param,
    ) -> PointInfo<Self::Var, Self::Deriv>
    {
        if z.is_nan()
        {
            return PointInfo::Escaping {
                potential: Real::from(iters) - 1.,
            };
        }

        let u = self.escape_radius().log2();
        let v = z.norm_sqr().log2();
        let residual = (v / u).log(self.degree_real());
        PointInfo::Escaping {
            potential: IterCount::from(iters * self.escaping_period()) - (residual as IterCount),
        }
    }
}

pub trait Computable: ParameterPlane + EscapeEncoding {
    fn compute(&self) -> IterPlane<Self::Var, Self::Deriv>;

    fn compute_into(&self, iter_plane: &mut IterPlane<Self::Var, Self::Deriv>);

    fn run_and_encode_point(
        &self,
        start: Self::Var,
        c: Self::Param,
    ) -> PointInfo<Self::Var, Self::Deriv>
    {
        let orbit_params = OrbitParams {
            max_iter: self.max_iter(),
            min_iter: self.min_iter(),
            periodicity_tolerance: self.periodicity_tolerance(),
            escape_radius: self.escape_radius(),
        };
        let orbit = CycleDetectedOrbitFloyd::new(
            |z, c| self.map(z, c),
            |z, c| self.map_and_multiplier(z, c),
            |z, c| self.early_bailout(z, c),
            start,
            c,
            &orbit_params,
        );
        if let Some((_, state)) = orbit.last()
        {
            self.encode_escape_result(state, c)
        }
        else
        {
            PointInfo::Bounded
        }
    }

    fn get_orbit_info(&self, point: Cplx) -> OrbitInfo<Self::Var, Self::Param, Self::Deriv>
    {
        let param = self.param_map(point);
        let start = self.start_point(point, param);
        let result = self.run_and_encode_point(start, param);
        OrbitInfo {
            start,
            param,
            result,
        }
    }

    fn get_orbit_and_info(&self, point: Cplx) -> OrbitAndInfo<Self::Var, Self::Param, Self::Deriv>
    {
        let param = self.param_map(point);
        let start = self.start_point(point, param);
        let orbit_params = OrbitParams {
            max_iter: self.max_iter(),
            min_iter: self.min_iter(),
            periodicity_tolerance: self.periodicity_tolerance(),
            escape_radius: self.escape_radius(),
        };
        let orbit = CycleDetectedOrbitFloyd::new(
            |c, z| self.map(c, z),
            |c, z| self.map_and_multiplier(c, z),
            |c, z| self.early_bailout(c, z),
            start,
            param,
            &orbit_params,
        );
        let mut final_state = EscapeState::Bounded;
        let trajectory: Vec<Self::Var> = orbit
            .map(|(z, s)| {
                final_state = s;
                z
            })
            .collect();
        let result = self.encode_escape_result(final_state, param);
        OrbitAndInfo {
            orbit: trajectory,
            info: OrbitInfo {
                start,
                param,
                result,
            },
        }
    }
}

impl<P> Computable for P where P: ParameterPlane + EscapeEncoding {
    fn compute(&self) -> IterPlane<Self::Var, Self::Deriv>

    {
        let mut iter_plane = IterPlane::create(self.point_grid().clone());
        self.compute_into(&mut iter_plane);
        iter_plane
    }

    fn compute_into(&self, iter_plane: &mut IterPlane<Self::Var, Self::Deriv>)
    {
        if self.point_grid().is_nan()
        {
            return;
        }

        let orbits = ThreadLocal::new();

        let chunk_size = self.point_grid().res_y / num_cpus::get(); // or another value that gives optimal performance

        iter_plane
            .iter_counts
            .axis_chunks_iter_mut(Axis(1), chunk_size)
            .enumerate()
            .par_bridge()
            .for_each(|(chunk_idx, mut chunk)| {
                let orbit_params = OrbitParams {
                    max_iter: self.max_iter(),
                    min_iter: self.min_iter(),
                    periodicity_tolerance: self.periodicity_tolerance(),
                    escape_radius: self.escape_radius(),
                };

                chunk.indexed_iter_mut().for_each(|((x, local_y), count)| {
                    let y = chunk_idx * chunk_size + local_y;
                    let point = self.point_grid().map_pixel(x, y);
                    let param = self.param_map(point);
                    let start = self.start_point(point, param);
                    let mut orbit = orbits
                        .get_or(|| {
                            RefCell::new(CycleDetectedOrbitFloyd::new(
                                |c, z| self.map(c, z),
                                |c, z| self.map_and_multiplier(c, z),
                                |c, z| self.early_bailout(c, z),
                                start,
                                param,
                                &orbit_params,
                            ))
                        })
                        .borrow_mut();

                    orbit.reset(param, start);
                    let result = orbit.run_until_complete();
                    *count = self.encode_escape_result(result, param);
                });
            });
    }
}

pub trait Displayable: ParameterPlane + ExternalRays + Equipotential + Computable {}
impl<P> Displayable for P where
P: ParameterPlane + ExternalRays + Equipotential + Computable {}
