use dynamo_color::{Coloring, IncoloringAlgorithm};
use dynamo_common::math_utils::contour::{Contour, IntegralCurveParams, LevelCurveParams};
use dynamo_common::math_utils::newton::error::{Error::NanEncountered, NewtonResult};
use dynamo_common::math_utils::{
    arithmetic::{divisors, gcd, moebius, Integer},
    newton::{find_root_newton, find_target_newton_err_d},
};
use dynamo_common::prelude::*;
use dynamo_common::symbolic_dynamics::OrbitSchema;
use num_traits::{One, Zero};

use ndarray::{Array2, Axis};
use num_cpus;
use rayon::iter::{ParallelBridge, ParallelIterator};
use std::{cell::RefCell, f64::consts::TAU};
use thread_local::ThreadLocal;

pub mod covering_maps;
pub mod julia;
pub mod newton;

use crate::error::{FindPointError, FindPointResult};
use crate::orbit::{self, EscapeResult, Orbit, Potential};
use julia::JuliaSet;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PlaneType
{
    #[default]
    Parameter,
    Dynamical,
}
impl PlaneType
{
    #[must_use]
    pub const fn is_dynamical(&self) -> bool
    {
        matches!(self, Self::Dynamical)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ComputeMode
{
    #[default]
    SmoothPotential,
    DistanceEstimation,
}
impl ComputeMode
{
    pub fn cycle(&mut self)
    {
        match self {
            Self::DistanceEstimation => *self = Self::SmoothPotential,
            Self::SmoothPotential => *self = Self::DistanceEstimation,
        }
    }

    pub fn create_orbit<'a, P: EscapeEncoding>(
        &self,
        family: &'a P,
    ) -> RefCell<Box<dyn Orbit<Outcome = PointInfo<P::Deriv>> + 'a>>
    {
        match self {
            Self::SmoothPotential => RefCell::new(Box::new(orbit::CycleDetected::new(family))),
            Self::DistanceEstimation => {
                RefCell::new(Box::new(orbit::DistanceEstimation::new(family)))
            }
        }
    }
}

pub trait DynamicalFamily: Sync + Send
{
    type Var: Variable;
    type Param: Parameter;
    type MetaParam: ParamList + Clone + Send + Sync + Default + Summarize;
    type Deriv: Derivative;

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

    fn max_iter(&self) -> IterCount;
    fn max_iter_mut(&mut self) -> &mut IterCount;
    fn set_max_iter(&mut self, new_max_iter: IterCount);

    #[must_use]
    fn with_max_iter(self, max_iter: IterCount) -> Self;

    fn compute_mode(&self) -> ComputeMode;
    fn compute_mode_mut(&mut self) -> &mut ComputeMode;
    fn set_compute_mode(&mut self, compute_mode: ComputeMode)
    {
        *self.compute_mode_mut() = compute_mode;
    }

    fn name(&self) -> String;
    fn long_name(&self) -> String
    {
        let short_name = self.name();
        self.get_param().summarize().map_or_else(
            || self.name(),
            |param_desc| format!("{short_name}: {param_desc}"),
        )
    }
    fn description(&self) -> String
    {
        String::new()
    }

    /// The map defining the dynamical system.
    fn map(&self, z: Self::Var, c: &Self::Param) -> Self::Var;

    /// The dynamical map, together with its derivative. This is the primary computational
    /// bottleneck, and should usually be implemented manually for optimization purposes.
    fn map_and_multiplier(&self, z: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv);

    /// The dynamical map, together with its derivative and parameter derivative. Used to compute
    /// external rays in parameter planes.
    fn gradient(&self, z: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        let (fz, df_dz) = self.map_and_multiplier(z, c);
        (fz, df_dz, Self::Deriv::one())
    }

    /// If certain regions in parameter space are known (e.g. the main cardioid in the Mandelbrot set), we can
    /// avoid having to compute orbits for parameters in those regions.
    ///
    /// This function returns an `EscapeResult`, depending on the starting point and parameter,
    /// if the result can be predicted. It is called once before computing each orbit.
    ///
    /// If this function returns `None`, then the orbit is computed.
    /// Otherwise, the output of this function is forwarded to `encode_escape_result`.
    fn early_bailout(&self, _start: Self::Var, _c: &Self::Param) -> Option<PointInfo<Self::Deriv>>
    {
        None
    }

    /// Minimum iterations before cycle detection is allowed.
    ///
    /// Useful for dynamical families with many parabolic systems, such as Cubic Per(1,1),
    /// in which orbits on the repelling side of the parabolic cylinder will remain
    /// near-periodic for a long time even if they will eventually escape.
    /// For such families, it is recommended to set `min_iter` to some constant fraction of
    /// `self.max_iter()`.
    #[inline]
    fn min_iter(&self) -> IterCount
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

    #[inline]
    fn extra_stop_condition(
        &self,
        z: Self::Var,
        _c: &Self::Param,
        iter: IterCount,
    ) -> Option<EscapeResult<Self::Var, Self::Deriv>>
    {
        let r = z.norm_sqr();
        if r > self.escape_radius() || z.is_nan() {
            Some(EscapeResult::Escaped {
                iters: iter,
                final_value: z,
            })
        } else {
            None
        }
    }

    #[inline]
    fn stop_condition(
        &self,
        z: Self::Var,
        c: &Self::Param,
        iter: IterCount,
    ) -> Option<EscapeResult<Self::Var, Self::Deriv>>
    {
        if iter < self.min_iter() {
            return None;
        }
        if iter > self.max_iter() {
            return Some(EscapeResult::Bounded(z));
        }

        self.extra_stop_condition(z, c, iter)
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
    fn start_point(&self, _point: Cplx, c: &Self::Param) -> Self::Var;

    /// Start point, its partial derivative with respect to the point,
    /// and its partial derivative with respect to the parameter
    fn start_point_d(&self, point: Cplx, c: &Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        (
            self.start_point(point, c),
            Self::Deriv::zero(),
            Self::Deriv::zero(),
        )
    }

    /// Map points in the image to parameters. Used for multi-parameter systems or covering maps
    /// over existing parameter planes.
    fn param_map(&self, point: Cplx) -> Self::Param;

    /// param_map together with its derivative.
    /// TODO: implement this correctly
    #[inline]
    fn param_map_d(&self, point: Cplx) -> (Self::Param, Self::Deriv)
    {
        (self.param_map(point), Self::Deriv::one())
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

    /// Try to find a (pre)periodic point near a given base point
    #[allow(clippy::suspicious_operation_groupings)]
    fn find_nearby_preperiodic_point(
        &self,
        start_point: Cplx,
        OrbitSchema {
            period: n,
            preperiod: k,
        }: OrbitSchema,
    ) -> FindPointResult<Cplx>
    {
        if n == 0 {
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
            let (mut z, mut dz_dt, dz_dc) = self.start_point_d(t, &c);

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
            if k > 0 {
                for _ in 0..k - 1 {
                    (z, df_dz, df_dc) = self.gradient(z, &c);
                    dz_dt = dz_dt * df_dz + df_dc;
                }

                zk1 = z.into();
                zk1_dt = dz_dt.into();
                (z, df_dz, df_dc) = self.gradient(z, &c);
                dz_dt = dz_dt * df_dz + df_dc;
            }

            let mut w = z;
            let mut dw_dt = dz_dt;

            // Periodic part

            for i in 1..n {
                (w, df_dz, df_dc) = self.gradient(w, &c);
                dw_dt = dw_dt * df_dz + df_dc;

                // Divide out lower order periods
                let (q, r) = n.div_rem(&i);
                if r == 0 {
                    let mu = moebius(q);
                    if mu == 1 {
                        values[term_count] = (w - z).into();
                        derivs[term_count] = (dw_dt - dz_dt).into();
                        term_count += 1;
                    } else if mu == -1 {
                        let dg = (dz_dt - dw_dt).into();
                        let val = (w - z).into().inv();
                        values[term_count] = val;
                        derivs[term_count] = dg * val * val;
                        term_count += 1;
                    }
                }
            }

            // At this point we have done k+n-1 iterations
            if k > 0 {
                // f^(k+n-1)(z) and its derivative with respect to t
                let zkn1 = w.into();
                let zkn1_dt = dw_dt.into();

                // 1/(f^(k+n-1)(z) - f^(k-1)(z)) and its derivative with respect to t
                early_cycle = (zkn1 - zk1).inv();
                early_cycle_dt = early_cycle * early_cycle * (zk1_dt - zkn1_dt);
            }

            // Perform final iteration manually
            (w, df_dz, df_dc) = self.gradient(w, &c);
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

    fn run_point(&self, selection: Cplx) -> EscapeResult<Self::Var, Self::Deriv>
    where
        Self: Clone,
    {
        let orbit = orbit::CycleDetected::new(self).init(selection);
        if let Some((_, state)) = orbit.last() {
            state.unwrap_or_default()
        } else {
            EscapeResult::Unknown
        }
    }

    fn iter_orbit(&self, point: Cplx) -> Box<dyn Iterator<Item = Self::Var> + '_>
    {
        let param = self.param_map(point);
        let start = self.start_point(point, &param);
        Box::new(
            orbit::simple::Simple::new(
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
        let start = self.start_point(point, &param);
        let orbit = orbit::simple::Simple::new(
            |z, c| self.map(z, c),
            start,
            param,
            self.max_iter(),
            self.escape_radius(),
        );
        orbit.map(|(z, _s)| z).collect()
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
    #[inline]
    fn plane_type(&self) -> PlaneType
    {
        PlaneType::Parameter
    }

    /// Define a custom fill rate for perperiod based coloring.
    fn preperiod_coloring(&self) -> IncoloringAlgorithm
    {
        IncoloringAlgorithm::PreperiodPeriod { fill_rate: 0.02 }
    }

    /// Internal: Since the internal potential coloring algorithm depends on the periodicity
    /// tolerance, we need to obtain it from this trait.
    fn internal_potential_coloring(&self) -> IncoloringAlgorithm
    {
        IncoloringAlgorithm::InternalPotential {
            periodicity_tolerance: self.periodicity_tolerance(),
            crit_degree: 2.0,
        }
    }

    /// Internal: Since the period + internal potential coloring algorithm depends on the periodicity
    /// tolerance, we need to obtain it from this trait.
    fn potential_and_period_coloring(&self) -> IncoloringAlgorithm
    {
        IncoloringAlgorithm::PotentialAndPeriod {
            periodicity_tolerance: self.periodicity_tolerance(),
            crit_degree: 2.0,
            fill_rate: 0.01,
        }
    }

    /// Optional map for superimposed contours
    fn auxiliary_value(&self, _t: Cplx) -> Option<(Cplx, Cplx)>
    {
        None
    }
}

impl std::fmt::Display for PlaneType
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self {
            Self::Parameter => write!(f, "parameter"),
            Self::Dynamical => write!(f, "dynamical"),
        }
    }
}

pub trait FamilyDefaults: DynamicalFamily + InfinityFirstReturnMap
{
    /// Default bounds for this plane
    fn default_bounds(&self) -> Bounds;

    /// Point to select when the plane is first created.
    #[inline]
    fn default_selection(&self) -> Cplx
    {
        Cplx::default()
    }

    /// Default coloring algorithm to apply when loading the parameter plane.
    fn default_coloring(&self) -> Coloring
    {
        let mut coloring = Coloring::default().with_escape_period(self.escaping_period());
        coloring.set_interior_algorithm(IncoloringAlgorithm::PeriodMultiplier);
        coloring
    }

    #[must_use]
    fn with_default_bounds(self) -> Self
    where
        Self: Sized,
    {
        let bounds = self.default_bounds();
        self.with_bounds(bounds)
    }
}

pub trait HasJulia: DynamicalFamily + InfinityFirstReturnMap
{
    #[inline]
    fn default_max_iter_child(&self) -> IterCount
    {
        128
    }

    /// Default bounds for Julia sets spawned from this plane. This is only called by Julia sets,
    /// who reference the parent's `default_bounds_child` in their `default_bounds`
    /// implementations.
    #[inline]
    fn default_bounds_child(&self, _point: Cplx, _c: &Self::Param) -> Bounds
    {
        Bounds::centered_square(2.2)
    }

    /// Default coloring algorithm to apply when loading the Julia set.
    fn default_coloring_child(&self) -> Coloring
    {
        Coloring::default()
            .with_interior_algorithm(self.internal_potential_coloring())
            .with_escape_period(1)
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
}

pub trait HasChild<C: DynamicalFamily>: DynamicalFamily
{
    fn to_child_param(param: Self::Param) -> <C::MetaParam as ParamList>::Param;
}

impl<T: HasJulia> HasChild<JuliaSet<T>> for T
{
    fn to_child_param(
        param: Self::Param,
    ) -> <<JuliaSet<T> as DynamicalFamily>::MetaParam as ParamList>::Param
    {
        param
    }
}

pub trait MarkedPoints: DynamicalFamily
{
    /// Critical points of the map associated to a given parameter, which can be marked on the dynamical plane.
    #[inline]
    fn critical_points_child(&self, c: &Self::Param) -> Vec<Self::Var>
    {
        vec![self.start_point(ZERO, c)]
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
    fn cycles_child(&self, _c: &Self::Param, _period: Period) -> Vec<Self::Var>
    {
        vec![]
    }

    /// Implementation of `precycles` for Julia sets spawned from this parameter plane.
    /// Used to mark selected preperiodic points on the dynamical plane.
    #[inline]
    fn precycles_child(&self, _c: &Self::Param, _orbit_schema: OrbitSchema) -> Vec<Self::Var>
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

    #[inline]
    fn other_marked_points(&self) -> Vec<Cplx>
    {
        Vec::new()
    }

    /// Attracting periodic points that are specially marked. Used for custom colorings, e.g. to
    /// color Newton parameter planes according to which root the critical orbit converges to.
    #[inline]
    fn get_marked_points(&self, _c: &Self::Param) -> Vec<(Self::Var, PointClassId)>
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
        z: Self::Var,
        c: &Self::Param,
        info: PointInfoPeriodic<Self::Deriv>,
    ) -> PointInfo<Self::Deriv>
    {
        let marked_points = self.get_marked_points(c);
        for (zi, class_id) in &marked_points {
            if z.dist_sqr(*zi) < self.marked_point_tolerance() {
                return PointInfo::MarkedPoint {
                    data: info,
                    class_id: *class_id,
                    num_point_classes: marked_points.len(),
                };
            }
        }
        PointInfo::Periodic(info)
    }
}

pub trait InfinityFirstReturnMap: DynamicalFamily
{
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
    fn escaping_period(&self) -> Period
    {
        1
    }

    /// For very large values of the parameter, how many iterations before the variable
    /// value is large?
    ///
    /// Used for computing external rays, for which we use an iterate of the map instead of the map
    /// itself.
    ///
    /// Almost always 0 or 1.
    #[inline]
    fn escaping_phase(&self) -> Period
    {
        1
    }

    /// Argument of f_c^k(z0) for c very large with a given argument,
    /// where k = self.escaping_phase().
    ///
    /// Used to seed initial point for external rays.
    #[inline]
    fn angle_map_large_param(&self, angle: RationalAngle) -> RationalAngle
    {
        angle
    }

    /// Leading coefficient of the self-return map at infinity.
    /// Used for computing external rays.
    #[inline]
    fn escape_coeff(&self, c: &Self::Param) -> Cplx
    {
        self.escape_coeff_d(c).0
    }

    /// Leading coefficient of the self-return map at infinity,
    /// together with its derivative.
    /// Used for computing external rays.
    fn escape_coeff_d(&self, _c: &Self::Param) -> (Cplx, Cplx)
    {
        (ONE, ZERO)
    }

    /// Evaluate Green's function given the escape time and final value
    fn smooth_iter_count(&self, iters: IterCount, z: Self::Var, c: &Self::Param) -> Real
    {
        let u = self.escape_radius().ln();
        let v = z.norm_sqr().ln();
        let q = self.escape_coeff(c).norm().ln();
        let residual = ((u + q) / (v + q)).log(self.degree_real()) as IterCountSmooth;
        residual.mul_add(
            IterCountSmooth::from(self.escaping_period()),
            iters as IterCountSmooth,
        )
    }

    /// External Green's function at a point
    fn external_potential_d(&self, t: Cplx) -> Option<(Real, Cplx)>
    {
        let mut orbit = Potential::new(self);
        orbit.reset(t);
        orbit.run_until_complete()
    }

    fn external_distance_estimate(&self, t: Cplx) -> Option<Real>
    {
        self.external_potential_d(t).map(|(g, dg)| (g / dg).norm())
    }
}

pub trait ExternalRays: DynamicalFamily + InfinityFirstReturnMap
{
    /// Default implementation of external rays. Only valid if the self-return map at infinity is
    /// monic.
    fn external_ray_helper(&self, angle: RationalAngle) -> Option<Vec<Cplx>>
    {
        const R: Real = 16.0;
        let escape_radius_log = R.ln() * self.degree_real().abs();

        let deg_real = self.degree_real().abs();
        if deg_real.is_nan() {
            return None;
        }

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

        let factor = (-deg_real.log2() / Real::from(RAY_SHARPNESS)).exp2();

        // Assumes escape_coeff is constant
        let a = self.escape_coeff(&self.param_map(ONE));
        let target_shift = a.ln() / Real::from(RAY_SHARPNESS);

        for k in 0..RAY_DEPTH {
            let num_iters = k * self.escaping_period() + self.escaping_phase();

            let fk_and_dfk = |t: Cplx| {
                let (c, dc_dt) = self.param_map_d(t);
                let (mut z, mut dz_dt, dz_dc) = self.start_point_d(t, &c);
                dz_dt += dz_dc * dc_dt;

                for _i in 0..num_iters {
                    let (f, df_dz, df_dc) = self.gradient(z, &c);
                    dz_dt = dz_dt * df_dz + df_dc * dc_dt;
                    z = f;
                }

                (z.into(), dz_dt.into())
            };

            let mut u = escape_radius_log;
            let mut v = Real::from(target_angle) * TAU;
            let mut t_curr = *t_list.last().unwrap_or(&base_point);

            for _j in 0..RAY_SHARPNESS {
                let target = Cplx::new(u, v).exp();
                match find_target_newton_err_d(fk_and_dfk, t_curr, target, error) {
                    Ok((sol, t_k, d_k)) => {
                        t_curr = sol;

                        if t_curr.is_nan() {
                            return Some(t_list);
                        }

                        t_list.push(t_curr);

                        let dist = (2. * t_k.norm() * (t_k.norm()).log(deg_real)) / d_k.norm();
                        if dist < pixel_width {
                            return Some(t_list);
                        }
                    }
                    Err(NanEncountered) => {
                        return Some(t_list);
                    }
                    _ => {}
                }
                u *= factor;
                u -= target_shift.re;
                v -= target_shift.im;
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
    /// Currently only stable for quadratic polynomials.
    fn external_ray(&self, angle: RationalAngle) -> Option<Vec<Cplx>>
    {
        // Remove off the end if distance is increasing,
        // as the helper method may return erroneous values near the end.
        // We use l1 norms to preserve precision.
        if let Some(mut t_list) = self.external_ray_helper(angle) {
            let t0 = t_list.last()?;
            let mut t1 = t_list.get(t_list.len() - 2)?;
            let mut t2 = t_list.get(t_list.len() - 3)?;
            let mut dist0 = (t0 - t1).l1_norm();
            let mut dist1 = (t1 - t2).l1_norm();
            while dist0 > dist1 {
                t_list.pop();
                t1 = t_list.last()?;
                t2 = t_list.get(t_list.len() - 2)?;
                dist0 = dist1;
                dist1 = (t1 - t2).l1_norm();
            }
            Some(t_list)
        } else {
            None
        }
    }
}

pub trait Equipotential: DynamicalFamily
{
    /// Compute an equipotential curve through a given point.
    fn equipotential<'a>(&'a self, t0: Cplx) -> Box<dyn Contour<Target = Real> + 'a>;

    /// Compute a level curve for the auxiliary map.
    fn aux_contour<'a>(&'a self, t0: Cplx) -> Box<dyn Contour<Target = Real> + 'a>;

    /// Compute a ray from t0 away from the bifurcation locus
    fn extend_ray<'a>(&'a self, t0: Cplx) -> Box<dyn Contour<Target = Real> + 'a>;

    /// Compute a ray from t0 towards from the bifurcation locus
    fn inward_ray<'a>(&'a self, t0: Cplx) -> Box<dyn Contour<Target = Real> + 'a>;
}
impl<P> Equipotential for P
where
    P: DynamicalFamily + InfinityFirstReturnMap,
{
    /// Equipotential through $t_0$
    ///
    /// Compute an equipotential by solving the ODE gamma'(t) = i∇G(t),
    /// where G is the exterior Green's function.
    fn equipotential<'a>(&'a self, t0: Cplx) -> Box<dyn Contour<Target = Real> + 'a>
    {
        let step_size = if self.plane_type().is_dynamical() {
            1e-2
        } else {
            1e-1
        };

        let contour = LevelCurveParams::default()
            .return_radius(self.point_grid().pixel_width().powi(2) * 100.0)
            .max_steps(500_000)
            .use_distance_estimation()
            .step_size(step_size)
            .contour(|t| self.external_potential_d(t))
            .init_seed(t0);

        Box::new(contour)
    }

    /// Outward external ray from $t_0$
    ///
    /// Compute an equipotential away from the bifurcation locus
    /// by solving the ODE gamma'(t) = -∇G(t),
    /// where G is the exterior Green's function.
    fn extend_ray<'a>(&'a self, t0: Cplx) -> Box<dyn Contour<Target = Real> + 'a>
    {
        Box::new(
            IntegralCurveParams::default()
                .step_size(1e-2)
                .max_steps(20000)
                .escape_radius(500.)
                .convergence_radius(self.periodicity_tolerance())
                .contour(t0, |t| {
                    self.external_potential_d(t).map(|(g, dg)| -g / dg.conj())
                }),
        )
    }

    /// Inward ray from $t_0$
    ///
    /// Compute an equipotential towards the bifurcation locus
    /// by solving the ODE gamma'(t) = ∇G(t),
    /// where G is the exterior Green's function.
    ///
    /// The ODE is stiff in this direction, and this method is likely
    /// to produce inaccurate results near the bifurcation locus.
    fn inward_ray<'a>(&'a self, t0: Cplx) -> Box<dyn Contour<Target = Real> + 'a>
    {
        Box::new(
            IntegralCurveParams::default()
                .step_size(1e-2)
                .max_steps(20000)
                .escape_radius(500.)
                .convergence_radius(self.periodicity_tolerance())
                .contour(t0, |t| {
                    self.external_potential_d(t).map(|(g, dg)| g / dg.conj())
                }),
        )
    }

    fn aux_contour<'a>(&'a self, t0: Cplx) -> Box<dyn Contour<Target = Real> + 'a>
    {
        Box::new(
            LevelCurveParams::default()
                .step_size(1e-2)
                .return_radius(self.point_grid().pixel_width().powi(2) * 100.0)
                .max_steps(5000)
                // .use_distance_estimation()
                .contour(|t| {
                    self.auxiliary_value(t)
                        .map(|(mu, dmu)| (mu.norm().ln(), (dmu / mu).conj()))
                })
                .init_seed(t0),
        )
    }
}

pub trait EscapeEncoding: DynamicalFamily + InfinityFirstReturnMap + MarkedPoints
{
    /// Map temporary `EscapeResult` (used in orbit computation) to `PointInfo`, encoding the result of the computation.
    ///
    /// `start_point` is normally unused, but is available as an input in case
    /// it is needed for a user-defined encoding, e.g. to cache escape results.
    fn encode_escape_result(
        &self,
        result: EscapeResult<Self::Var, Self::Deriv>,
        _start_point: Self::Var,
        c: &Self::Param,
    ) -> PointInfo<Self::Deriv>
    {
        match result {
            EscapeResult::Escaped { iters, final_value } => {
                self.encode_escaping_point(iters, final_value, c)
            }
            EscapeResult::Periodic { info, final_value } => {
                self.identify_marked_points(final_value, c, info)
            }
            EscapeResult::Bounded(_) => PointInfo::Bounded,
            EscapeResult::Unknown => PointInfo::Unknown,
        }
    }

    /// Encode the potential of an escaping point.
    /// The potential returned is equal to
    /// log_D(log(E)) - log_D(G) - 1,
    /// where E is the escape radius, D is the escaping degree, and G is the Green's function.
    fn encode_escaping_point(
        &self,
        iters: IterCount,
        z: Self::Var,
        c: &Self::Param,
    ) -> PointInfo<Self::Deriv>
    {
        if z.is_nan() {
            return PointInfo::Escaping {
                potential: (iters as IterCountSmooth).exp(),
                phase: None,
            };
        }

        let potential = self.smooth_iter_count(iters, z, c);
        PointInfo::Escaping {
            potential,
            phase: None,
        }
    }
}

pub trait Computable: DynamicalFamily
{
    fn compute(&self) -> IterPlane<Self::Deriv>
    {
        let mut iter_plane = IterPlane::create(self.point_grid().clone());
        self.compute_into(&mut iter_plane);
        iter_plane
    }

    fn compute_into(&self, iter_plane: &mut IterPlane<Self::Deriv>);

    fn get_orbit_and_info(
        &self,
        point: Cplx,
    ) -> orbit::OrbitAndInfo<Self::Param, Self::Var, Self::Deriv>;

    fn orbit_summary_conf(&self) -> orbit::OrbitSummaryConf
    {
        orbit::OrbitSummaryConf {
            show_parameter: true,
            show_selection: true,
            show_start_point: !self.plane_type().is_dynamical(),
            float_prec: DISPLAY_PREC,
        }
    }
}

impl<P> Computable for P
where
    P: DynamicalFamily + EscapeEncoding,
{
    fn get_orbit_and_info(
        &self,
        point: Cplx,
    ) -> orbit::OrbitAndInfo<Self::Param, Self::Var, Self::Deriv>
    {
        let orbit = orbit::CycleDetected::new(self).init(point);
        let start = orbit.z_fast;
        let param = orbit.param.clone();
        let mut final_state = None;
        let trajectory: Vec<Self::Var> = orbit
            .map(|(z, s)| {
                final_state = s;
                z
            })
            .collect();
        let result = self.encode_escape_result(final_state.unwrap_or_default(), start, &param);
        orbit::OrbitAndInfo {
            orbit: trajectory,
            info: orbit::Info {
                param,
                start,
                result,
            },
        }
    }

    fn compute_into(&self, iter_plane: &mut IterPlane<Self::Deriv>)
    {
        if self.point_grid().is_nan() {
            return;
        }

        let orbits = ThreadLocal::new();

        let chunk_size = self.point_grid().res_y / num_cpus::get();

        iter_plane
            .iter_counts
            .axis_chunks_iter_mut(Axis(1), chunk_size)
            .enumerate()
            .par_bridge()
            .for_each(|(chunk_idx, mut chunk)| {
                chunk.indexed_iter_mut().for_each(|((x, local_y), count)| {
                    let y = chunk_idx * chunk_size + local_y;
                    let mut orbit = orbits
                        .get_or(|| self.compute_mode().create_orbit(self))
                        .borrow_mut();

                    let point = self.point_grid().map_pixel(x, y);
                    orbit.reset(point);
                    *count = orbit.run_until_complete();
                });
            });
    }
}

pub trait Displayable:
    DynamicalFamily + FamilyDefaults + ExternalRays + Equipotential + Computable + MarkedPoints
{
}
impl<P> Displayable for P where
    P: DynamicalFamily + FamilyDefaults + ExternalRays + Equipotential + Computable + MarkedPoints
{
}
