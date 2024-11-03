use std::f64::consts::LN_2;

use super::{EscapeResult, Orbit};
use crate::dynamics::InfinityFirstReturnMap;
use dynamo_common::prelude::*;
use num_traits::One;

/// An orbit that tracks the gradient of f in order to compute the Green's function and its
/// derivative at a poin.
///
/// Example usage:
///
/// ```
/// let mandelbrot = dynamo_profiles::Mandelbrot::new();
/// let orbit = Potential::new(family).init(Cplx::new(-0.75, 0.001));
/// let (green, d_green) = orbit.run_until_complete()?;
/// ```
pub struct Potential<'a, P: InfinityFirstReturnMap + ?Sized>
{
    family: &'a P,
    periodicity_tolerance: Real,
    log_tol: Real,
    selection: Cplx,
    param: P::Param,
    dc_dt: P::Deriv,
    pub z_init: P::Var,
    pub z_slow: P::Var,
    pub z_fast: P::Var,
    pub dz_dt_fast: P::Deriv,
    pub dz_dt_slow: P::Deriv,
    pub iter: IterCount,
    pub state: Option<EscapeResult<P::Var, P::Deriv>>,
}

impl<'a, P: InfinityFirstReturnMap + ?Sized> Potential<'a, P>
{
    pub fn new(family: &'a P) -> Self
    {
        let periodicity_tolerance = family.periodicity_tolerance();
        let log_tol = periodicity_tolerance.ln();
        Self {
            family,
            selection: ZERO,
            param: P::Param::default(),
            periodicity_tolerance,
            log_tol,
            dc_dt: P::Deriv::one(),
            z_init: P::Var::default(),
            z_slow: P::Var::default(),
            z_fast: P::Var::default(),
            dz_dt_fast: P::Deriv::one(),
            dz_dt_slow: P::Deriv::one(),
            iter: 0,
            state: None,
        }
    }

    #[must_use]
    pub fn init(mut self, selection: Cplx) -> Self
    {
        self.reset(selection);
        self
    }

    #[inline]
    fn update_slow(&mut self)
    {
        let (f, df_dz, df_dc) = self.family.gradient(self.z_slow, &self.param);

        self.dz_dt_slow = df_dz * self.dz_dt_slow + df_dc * self.dc_dt;
        self.z_slow = f;
    }

    #[inline]
    fn update_fast(&mut self)
    {
        let (f, df_dz, df_dc) = self.family.gradient(self.z_fast, &self.param);

        self.dz_dt_fast = df_dz * self.dz_dt_fast + df_dc * self.dc_dt;
        self.z_fast = f;
    }

    #[inline]
    fn update_fast_lazy(&mut self)
    {
        self.z_fast = self.family.map(self.z_fast, &self.param);
    }

    #[inline]
    fn update_slow_lazy(&mut self)
    {
        self.z_slow = self.family.map(self.z_slow, &self.param);
    }

    #[inline]
    fn enforce_stop_condition(&mut self) -> bool
    {
        if let Some(state) = self
            .family
            .stop_condition(self.z_fast, &self.param, self.iter)
        {
            self.state = Some(state);
            true
        } else {
            false
        }
    }

    fn check_periodicity(&mut self)
    {
        if self.enforce_stop_condition() {
            return;
        }

        let error = self.z_fast.dist_sqr(self.z_slow);
        if error < self.periodicity_tolerance {
            if let Some((period, multiplier)) =
                self.compute_period(self.periodicity_tolerance.powf(0.75), self.iter as usize)
            {
                let info = PointInfoPeriodic {
                    preperiod: self.iter,
                    period,
                    multiplier,
                    final_error: error,
                };
                self.state = Some(EscapeResult::Periodic {
                    info,
                    final_value: self.z_fast,
                });
            }
        }
    }

    fn compute_period(&self, tolerance: Real, patience: usize) -> Option<(Period, P::Deriv)>
    {
        let mut z = self.z_fast;
        let mut dz: P::Deriv;
        let mut mult = P::Deriv::one();
        for i in 1..=patience {
            (z, dz) = self.family.map_and_multiplier(z, &self.param);
            mult *= dz;
            if z.dist_sqr(self.z_fast) <= tolerance {
                return Period::try_from(i).ok().map(|n| (n, mult));
            }
        }
        None
    }

    /// Logarithm of the Koenigs coordinate, together with its gradient
    fn periodic_koenigs_d(&mut self, period: Period, mult_norm: Real) -> Option<(Real, Cplx)>
    {
        self.reset(self.selection);
        for _ in 0..period {
            self.update_fast();
        }
        while self.z_fast.dist_sqr(self.z_slow) > self.periodicity_tolerance {
            self.update_slow();
            self.update_fast();
            self.iter += 1;
            if self.iter == self.family.max_iter() {
                return None;
            }
        }

        let err = (self.z_fast - self.z_slow).into();
        let derr_dt = (self.dz_dt_fast - self.dz_dt_slow).into();

        // err = λ^n ϕ
        // ϕ = 1/λ^n err
        // log ϕ = log(err) - n log(λ)
        let log_phi = mult_norm
            .ln()
            .mul_add(-(self.iter as Real), err.norm_sqr().ln());

        Some((log_phi, 2.0 * (derr_dt / err).conj()))
    }

    /// Brute force calculation of multiplier derivative
    fn periodic_koenigs_d_param(
        &mut self,
        period: Period,
        multiplier: P::Deriv,
    ) -> Option<(Real, Cplx)>
    {
        const EPS: Real = 1e-6;

        self.reset(self.selection + EPS);

        for _ in 0..period * 50 {
            self.update_fast_lazy();
        }
        while self.z_fast.dist_sqr(self.z_slow) > self.periodicity_tolerance {
            self.update_slow_lazy();
            self.update_fast_lazy();
            self.iter += 1;
            if self.iter == self.family.max_iter() {
                return None;
            }
        }

        let mut mult_delta = P::Deriv::one();
        let mut dz: P::Deriv;

        for _ in 0..period {
            (self.z_fast, dz) = self.family.map_and_multiplier(self.z_fast, &self.param);
            mult_delta *= dz;
        }

        let d_mult = (mult_delta - multiplier).into() / EPS;
        Some((
            multiplier.norm_sqr(),
            2. * d_mult.conj() * multiplier.into(),
        ))
    }

    /// Logarithm of the Green's function, together with its gradient,
    /// for bounded orbits
    fn periodic_bottcher_d(&mut self, period: Period) -> Option<(Real, Cplx)>
    {
        self.reset(self.selection);
        for _ in 0..period {
            self.update_fast();
        }
        while self.z_fast.dist_sqr(self.z_slow) > self.periodicity_tolerance {
            self.update_slow();
            self.update_fast();
            self.iter += 1;
            if self.iter == self.family.max_iter() {
                return None;
            }
        }

        let err = (self.z_fast - self.z_slow).into();
        let derr_dt = (self.dz_dt_fast - self.dz_dt_slow).into();

        let norm_err = err.norm_sqr();
        let norm_err_log = norm_err.ln();

        let phi = LN_2.mul_add(self.iter as Real, (norm_err_log / self.log_tol).ln());
        let grad_phi = 2.0 * err * (derr_dt / (norm_err_log * norm_err)).conj();
        Some((phi, grad_phi))
    }

    /// Logarithm of the Green's function, together with its gradient,
    /// for unbounded orbits
    fn external_bottcher_d(&self, iters: IterCount) -> (Real, Cplx)
    {
        let log_dn = iters as Real * self.family.degree_real().ln();

        let z: Cplx = self.z_fast.into();
        let dz_dt: Cplx = self.dz_dt_fast.into();

        let norm_z = z.norm_sqr();
        let norm_z_log = norm_z.ln();

        let phi = norm_z_log
            .ln()
            .mul_add(-Real::from(self.family.escaping_period()), log_dn);
        let grad_phi = -2.0 * z * (dz_dt / (norm_z_log * norm_z)).conj();

        (phi, grad_phi)
    }
}
impl<P: InfinityFirstReturnMap + ?Sized> Orbit for Potential<'_, P>
{
    type Outcome = Option<(Real, Cplx)>;

    fn reset(&mut self, selection: Cplx)
    {
        let (c, dc_dt) = self.family.param_map_d(selection);
        let (z, mut dz_dt, dz_dc) = self.family.start_point_d(selection, &c);
        dz_dt += dz_dc * dc_dt;

        self.state = None;
        self.selection = selection;
        self.param = c;
        self.z_init = z;
        self.z_slow = z;
        self.z_fast = z;
        self.dc_dt = dc_dt;
        self.dz_dt_fast = dz_dt;
        self.dz_dt_slow = dz_dt;
        self.iter = 0;
    }

    fn run_until_complete(&mut self) -> Self::Outcome
    {
        use EscapeResult::{Bounded, Escaped, Periodic, Unknown};

        while self.state.is_none() {
            self.iter += 1;
            if self.iter % 2 == 1 {
                self.update_slow();
                self.update_fast();
                self.enforce_stop_condition();
            } else {
                self.update_fast();
                self.check_periodicity();
            }
        }

        let state = self.state.as_ref()?;

        match state {
            Escaped { iters, .. } => Some(self.external_bottcher_d(*iters)),
            Unknown | Bounded(..) => None,
            Periodic { info, .. } => {
                let mult_norm = info.multiplier.into().norm_sqr();
                let period = info.period;

                if mult_norm <= 1e-10 {
                    self.periodic_bottcher_d(period)
                } else if self.family.plane_type().is_dynamical() {
                    self.periodic_koenigs_d(period, mult_norm)
                } else {
                    self.periodic_koenigs_d_param(period, info.multiplier)
                }
            }
        }
    }
}
