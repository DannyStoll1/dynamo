use super::{EscapeResult, OrbitParams};
use crate::prelude::DynamicalFamily;
use dynamo_common::prelude::*;
use num_traits::One;

pub struct CycleDetected<'a, P: DynamicalFamily>
{
    family: &'a P,
    param: P::Param,
    periodicity_tolerance: Real,
    pub z_slow: P::Var,
    pub z_fast: P::Var,
    pub multiplier: P::Deriv,
    pub iter: Period,
    pub state: Option<EscapeResult<P::Var, P::Deriv>>,
    running: bool,
}

impl<'a, P: DynamicalFamily> CycleDetected<'a, P>
{
    pub fn new(family: &'a P, start: P::Var, param: P::Param) -> Self
    {
        Self {
            family,
            param,
            periodicity_tolerance: family.periodicity_tolerance(),
            z_slow: start,
            z_fast: start,
            multiplier: P::Deriv::one(),
            iter: 0,
            state: None,
            running: true,
        }
    }

    #[inline]
    fn apply_map_to_slow(&mut self)
    {
        self.z_slow = self.family.map(self.z_slow, self.param);
    }

    #[inline]
    fn apply_map_and_update_multiplier(&mut self)
    {
        let (z_new, deriv) = self.family.map_and_multiplier(self.z_fast, self.param);
        self.multiplier *= deriv;
        self.z_fast = z_new;
    }

    pub fn reset(&mut self, param: P::Param, start_point: P::Var)
    {
        self.state = None;
        self.param = param;
        self.z_slow = start_point;
        self.z_fast = start_point;
        self.multiplier = P::Deriv::one();
        self.iter = 0;
    }

    pub fn run_until_complete(&mut self) -> EscapeResult<P::Var, P::Deriv>
    {
        if let Some(res) = self.family.early_bailout(self.z_fast, self.param) {
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
        #[allow(clippy::unwrap_used)]
        self.state.clone().unwrap()
    }

    #[inline]
    fn enforce_stop_condition(&mut self) -> bool
    {
        if let Some(state) = self
            .family
            .stop_condition(self.z_fast, self.param, self.iter)
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
            (z, dz) = self.family.map_and_multiplier(z, self.param);
            mult *= dz;
            if z.dist_sqr(self.z_fast) <= tolerance {
                return Period::try_from(i).ok().map(|n| (n, mult));
            }
        }
        None
    }
}

impl<'a, P: DynamicalFamily> Iterator for CycleDetected<'a, P>
{
    type Item = (P::Var, Option<EscapeResult<P::Var, P::Deriv>>);

    fn next(&mut self) -> Option<Self::Item>
    {
        if self.state.is_none() {
            let retval = self.z_fast;
            self.iter += 1;
            if self.iter % 2 == 1 {
                self.apply_map_to_slow();
                self.apply_map_and_update_multiplier();
                self.enforce_stop_condition();
            } else {
                self.apply_map_and_update_multiplier();
                self.check_periodicity();
            }
            Some((retval, self.state.clone()))
        } else if self.running {
            self.running = false;
            Some((self.z_fast, self.state.clone()))
        } else {
            None
        }
    }
}
