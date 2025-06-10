use super::{EscapeResult, Orbit};
use crate::dynamics::EscapeEncoding;
use dynamo_common::prelude::*;
use num_traits::One;

pub struct DistanceEstimation<'a, P: EscapeEncoding + ?Sized>
{
    family: &'a P,
    param: P::Param,
    periodicity_tolerance: Real,
    pub z_init: P::Var,
    pub z_slow: P::Var,
    pub z_fast: P::Var,
    pub multiplier: P::Deriv,
    pub dc_dt: P::Deriv,
    pub dz_dt: P::Deriv,
    pub iter: IterCount,
    pub state: Option<EscapeResult<P::Var, P::Deriv>>,
}

impl<'a, P: EscapeEncoding + ?Sized> DistanceEstimation<'a, P>
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
            multiplier: P::Deriv::one(),
            dc_dt: P::Deriv::one(),
            dz_dt: P::Deriv::one(),
            iter: 0,
            state: None,
        }
    }

    #[must_use]
    #[allow(clippy::similar_names)]
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

        self.multiplier *= df_dz;
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
impl<P: EscapeEncoding + ?Sized> Orbit for DistanceEstimation<'_, P>
{
    type Outcome = PointInfo<P::Deriv>;

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
        self.multiplier = P::Deriv::one();
        self.dc_dt = dc_dt;
        self.dz_dt = dz_dt;
        self.iter = 0;
    }

    fn run_until_complete(&mut self) -> Self::Outcome
    {
        if let Some(res) = self.family.early_bailout(self.z_fast, &self.param) {
            return res;
        }

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

        if let Some(EscapeResult::Escaped { iters, final_value }) = self.state {
            let norm_z = final_value.into().norm();
            let distance = norm_z * norm_z.ln() / self.dz_dt.norm();
            return PointInfo::DistanceEstimate {
                distance,
                phase: (iters % IterCount::from(self.family.escaping_period())) as Period,
            };
        }

        #[allow(clippy::unwrap_used)]
        self.family
            .encode_escape_result(self.state.clone().unwrap(), self.z_init, &self.param)
    }
}
