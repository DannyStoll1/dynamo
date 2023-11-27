use super::{EscapeResult, Orbit};
use crate::dynamics::DynamicalFamily;
use dynamo_common::prelude::*;
use num_traits::One;

pub struct Potential<'a, P: DynamicalFamily + ?Sized>
{
    family: &'a P,
    param: P::Param,
    periodicity_tolerance: Real,
    pub z_init: P::Var,
    pub z_slow: P::Var,
    pub z_fast: P::Var,
    pub dc_dt: P::Deriv,
    pub dz_dt: P::Deriv,
    pub iter: Period,
    pub state: Option<EscapeResult<P::Var, P::Deriv>>,
}

impl<'a, P: DynamicalFamily + ?Sized> Potential<'a, P>
{
    pub fn new(family: &'a P) -> Self
    {
        Self {
            family,
            param: P::Param::default(),
            periodicity_tolerance: family.periodicity_tolerance(),
            z_init: P::Var::default(),
            z_slow: P::Var::default(),
            z_fast: P::Var::default(),
            dc_dt: P::Deriv::one(),
            dz_dt: P::Deriv::one(),
            iter: 0,
            state: None,
        }
    }

    #[must_use]
    fn init(mut self, selection: Cplx) -> Self
    {
        let (c, dc_dt) = self.family.param_map_d(selection);
        let (z, mut dz_dt, dz_dc) = self.family.start_point_d(selection, &c);
        dz_dt += dz_dc * dc_dt;

        self.param = c;
        self.z_init = z;
        self.z_slow = z;
        self.z_fast = z;
        self.dc_dt = dc_dt;
        self.dz_dt = dz_dt;
        self
    }

    #[inline]
    fn apply_map_to_slow(&mut self)
    {
        self.z_slow = self.family.map(self.z_slow, &self.param);
    }

    #[inline]
    fn apply_map_and_update_multiplier(&mut self)
    {
        let (f, df_dz, df_dc) = self.family.gradient(self.z_fast, &self.param);

        self.dz_dt = df_dz * self.dz_dt + df_dc * self.dc_dt;
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
        self.param = c;
        self.z_init = z;
        self.z_slow = z;
        self.z_fast = z;
        self.dc_dt = dc_dt;
        self.dz_dt = dz_dt;
        self.iter = 0;
    }

    fn run_until_complete(&mut self) -> Self::Outcome
    {
        while self.state.is_none() {
            self.iter += 1;
            if self.iter % 2 == 1 {
                self.apply_map_to_slow();
                self.apply_map_and_update_multiplier();
                self.enforce_stop_condition();
            } else {
                self.apply_map_and_update_multiplier();
                self.check_periodicity();
            }
        }

        let state = self.state.as_ref()?;

        match state {
            EscapeResult::Escaped { iters, final_value } => {
                let rescale = 2.0f64.powi(-(*iters as i32));
                let z: Cplx = final_value.clone().into();
                let dz_dt: Cplx = self.dz_dt.into();
                let d_green = (dz_dt / z).conj() * rescale;
                let green = z.norm().ln() * rescale;
                return Some((green, d_green));
            }
            EscapeResult::Unknown => None,
            EscapeResult::Bounded(..) => None,
            EscapeResult::Periodic { info, final_value } => {
                let z = final_value.clone();
                let dz_dt = self.dz_dt.into();
                let mult_norm_log = -info.multiplier.into().norm().ln();
                for _ in 0..info.period {
                    self.apply_map_and_update_multiplier();
                }
                let err: Cplx = (self.z_fast - z).into() / self.periodicity_tolerance;
                let dz_dt_final = self.dz_dt.into();
                let derr_dt = (dz_dt_final - dz_dt) / self.periodicity_tolerance;

                Some((
                    err.norm().ln() / mult_norm_log,
                    -(derr_dt / err).conj() / mult_norm_log,
                ))
            }
        }
    }
}
