use dynamo_common::prelude::*;
use num_traits::One;

use super::{EscapeResult, Orbit};
use crate::dynamics::EscapeEncoding;
use crate::prelude::DynamicalFamily;

pub struct CycleDetected<'a, P: DynamicalFamily>
{
    family: &'a P,
    periodicity_tolerance: Real,
    pub param: P::Param,
    pub z_init: P::Var,
    pub z_slow: P::Var,
    pub z_fast: P::Var,
    pub iter: IterCount,
    pub state: Option<EscapeResult<P::Var, P::Deriv>>,
    running: bool,
}

impl<'a, P: DynamicalFamily> CycleDetected<'a, P>
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
            iter: 0,
            state: None,
            running: true,
        }
    }

    /// Initialize an orbit. Should only be called once, before running any computations.
    #[must_use]
    pub fn init(mut self, selection: Cplx) -> Self
    {
        let c = self.family.param_map(selection);
        let z = self.family.start_point(selection, &c);

        self.param = c;
        self.z_init = z;
        self.z_slow = z;
        self.z_fast = z;
        self
    }

    #[inline]
    fn apply_map_to_slow(&mut self)
    {
        self.z_slow = self.family.map(self.z_slow, &self.param);
    }

    #[inline]
    fn apply_map_to_fast(&mut self)
    {
        self.z_fast = self.family.map(self.z_fast, &self.param);
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

impl<P: EscapeEncoding> Orbit for CycleDetected<'_, P>
{
    type Outcome = PointInfo<P::Deriv>;

    fn run_until_complete(&mut self) -> Self::Outcome
    {
        if let Some(res) = self.family.early_bailout(self.z_fast, &self.param) {
            return res;
        }

        while self.state.is_none() {
            self.iter += 1;
            if self.iter % 2 == 1 {
                self.apply_map_to_slow();
                self.apply_map_to_fast();
                self.enforce_stop_condition();
            } else {
                self.apply_map_to_fast();
                self.check_periodicity();
            }
        }
        #[allow(clippy::unwrap_used)]
        self.family
            .encode_escape_result(self.state.clone().unwrap(), self.z_init, &self.param)
    }

    fn reset(&mut self, selection: Cplx)
    {
        let c = self.family.param_map(selection);
        let z = self.family.start_point(selection, &c);

        self.state = None;
        self.param = c;
        self.z_init = z;
        self.z_slow = z;
        self.z_fast = z;
        self.iter = 0;
    }
}

impl<P: DynamicalFamily> Iterator for CycleDetected<'_, P>
{
    type Item = (P::Var, Option<EscapeResult<P::Var, P::Deriv>>);

    fn next(&mut self) -> Option<Self::Item>
    {
        if self.state.is_none() {
            let retval = self.z_fast;
            self.iter += 1;
            if self.iter % 2 == 1 {
                self.apply_map_to_slow();
                self.apply_map_to_fast();
                self.enforce_stop_condition();
            } else {
                self.apply_map_to_fast();
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
