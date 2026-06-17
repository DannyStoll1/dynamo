use dynamo_common::prelude::*;
use num_traits::One as _;

use super::{EscapeResult, Orbit};
use crate::dynamics::EscapeEncoding;

pub struct DistanceEstimation<'a, P: EscapeEncoding>
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

struct StartState<V, Param, Deriv>
{
    param:       Param,
    value:       V,
    param_deriv: Deriv,
    total_deriv: Deriv,
}

impl<'a, P: EscapeEncoding> DistanceEstimation<'a, P>
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
    pub fn init(mut self, selection: Cplx) -> Self
    {
        self.apply_start_state(self.start_state(selection));
        self
    }

    fn start_state(&self, selection: Cplx) -> StartState<P::Var, P::Param, P::Deriv>
    {
        let (param, param_deriv) = self.family.param_map_d(selection);
        let (value, mut total_deriv, param_scale) = self.family.start_point_d(selection, &param);
        total_deriv += param_scale * param_deriv;

        StartState {
            param,
            value,
            param_deriv,
            total_deriv,
        }
    }

    fn apply_start_state(&mut self, start: StartState<P::Var, P::Param, P::Deriv>)
    {
        self.param = start.param;
        self.z_init = start.value;
        self.z_slow = start.value;
        self.z_fast = start.value;
        self.dc_dt = start.param_deriv;
        self.dz_dt = start.total_deriv;
    }

    #[inline]
    fn apply_map_to_slow(&mut self)
    {
        self.z_slow = self.family.map(self.z_slow, &self.param);
    }

    #[inline]
    fn apply_map_and_update_multiplier(&mut self)
    {
        let (next_value, z_scale, c_scale) = self.family.gradient(self.z_fast, &self.param);

        self.multiplier *= z_scale;
        self.dz_dt = z_scale * self.dz_dt + c_scale * self.dc_dt;
        self.z_fast = next_value;
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
        if error < self.periodicity_tolerance
            && let Some((period, multiplier)) =
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
impl<P: EscapeEncoding> Orbit for DistanceEstimation<'_, P>
{
    type Outcome = PointInfo<P::Deriv>;

    fn reset(&mut self, selection: Cplx)
    {
        self.state = None;
        self.multiplier = P::Deriv::one();
        self.apply_start_state(self.start_state(selection));
        self.iter = 0;
    }

    fn run_until_complete(&mut self) -> Self::Outcome
    {
        if let Some(res) = self.family.early_bailout(self.z_fast, &self.param) {
            return res;
        }

        let state = loop {
            self.iter += 1;
            if self.iter % 2 == 1 {
                self.apply_map_to_slow();
                self.apply_map_and_update_multiplier();
                self.enforce_stop_condition();
            } else {
                self.apply_map_and_update_multiplier();
                self.check_periodicity();
            }
            if let Some(state) = self.state.take() {
                break state;
            }
        };

        if let EscapeResult::Escaped { iters, final_value } = state {
            let norm_z = final_value.into().norm();
            let distance = norm_z * norm_z.ln() / self.dz_dt.norm();
            return PointInfo::DistanceEstimate {
                distance,
                phase: (iters % IterCount::from(self.family.escaping_period())) as Period,
            };
        }

        self.family
            .encode_escape_result(state, self.z_init, &self.param)
    }
}
