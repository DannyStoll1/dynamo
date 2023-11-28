use std::f64::consts::LN_2;

use super::{EscapeResult, Orbit};
use crate::dynamics::DynamicalFamily;
use dynamo_common::prelude::*;
use num_traits::One;

pub struct Potential<'a, P: DynamicalFamily + ?Sized>
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
    pub iter: Period,
    pub state: Option<EscapeResult<P::Var, P::Deriv>>,
}

impl<'a, P: DynamicalFamily + ?Sized> Potential<'a, P>
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
    fn init(mut self, selection: Cplx) -> Self
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

    fn periodic_koenigs_d(&mut self, period: Period, mult_norm: Real) -> Option<(Real, Cplx)>
    {
        let z = self.z_fast.clone();
        let dz_dt = self.dz_dt_fast.into();
        for _ in 0..period {
            self.update_fast();
        }

        let err: Cplx = (self.z_fast - z).into() / self.periodicity_tolerance;
        let dz_dt_final = self.dz_dt_fast.into();
        let derr_dt = (dz_dt_final - dz_dt) / self.periodicity_tolerance;

        let mult_norm_log = -mult_norm.ln();
        Some((
            err.norm().ln() / mult_norm_log,
            -(derr_dt / err).conj() / mult_norm_log,
        ))
    }

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
        }

        let err: Cplx = (self.z_fast - self.z_slow).into();
        let dz_dt_fast = self.dz_dt_fast.into();
        let dz_dt_slow = self.dz_dt_slow.into();
        let derr_dt = dz_dt_fast - dz_dt_slow;

        let norm_err = err.norm_sqr();
        let log_norm_err = norm_err.ln();

        let phi = (log_norm_err / self.log_tol).ln() + (self.iter as Real) * LN_2;
        let grad_phi = err * (derr_dt / (log_norm_err * norm_err)).conj();
        Some((phi, grad_phi))
    }
}
impl<'a, P: DynamicalFamily + ?Sized> Orbit for Potential<'a, P>
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
            EscapeResult::Escaped { iters, final_value } => {
                let rescale = 2.0f64.powi(-(*iters as i32));
                let z: Cplx = final_value.clone().into();
                let dz_dt: Cplx = self.dz_dt_fast.into();
                let d_green = (dz_dt / z).conj() * rescale;
                let green = z.norm().ln() * rescale;
                return Some((green, d_green));
            }
            EscapeResult::Unknown => None,
            EscapeResult::Bounded(..) => None,
            EscapeResult::Periodic { info, .. } => {
                let mult_norm = info.multiplier.into().norm();
                let period = info.period;

                if mult_norm <= 1e-10 {
                    self.periodic_bottcher_d(period)
                } else {
                    self.periodic_koenigs_d(period, mult_norm)
                }
            }
        }
    }
}
