use super::{EscapeResult, OrbitParams};
use dynamo_common::prelude::*;
use num_traits::One;

pub struct CycleDetected<V, P, D, F, G, S, B>
where
    F: Fn(V, P) -> V,
    G: Fn(V, P) -> (V, D),
    S: Fn(V, P, Period) -> Option<EscapeResult<V, D>>,
    P: Copy,
    V: Norm<Real> + Dist<Real>,
    D: Norm<Real> + std::ops::MulAssign + One,
{
    f: F,
    map_and_multiplier: G,
    early_bailout: B,
    stop_condition: S,
    param: P,
    periodicity_tolerance: Real,
    escape_radius: Real,
    pub z_slow: V,
    pub z_fast: V,
    pub multiplier: D,
    pub iter: Period,
    pub state: Option<EscapeResult<V, D>>,
}

impl<V, P, D, F, G, S, B> CycleDetected<V, P, D, F, G, S, B>
where
    F: Fn(V, P) -> V,
    G: Fn(V, P) -> (V, D),
    S: Fn(V, P, Period) -> Option<EscapeResult<V, D>>,
    P: Copy,
    V: Norm<Real> + Dist<Real> + MaybeNan,
    D: Norm<Real> + std::ops::MulAssign + One,
{
    pub fn new(
        f: F,
        map_and_multiplier: G,
        stop_condition: S,
        early_bailout: B,
        z: V,
        param: P,
        orbit_params: &OrbitParams,
    ) -> Self
    {
        Self {
            f,
            map_and_multiplier,
            early_bailout,
            stop_condition,
            param,
            z_slow: z,
            z_fast: z,
            multiplier: D::one(),
            iter: 0,
            state: None,
            periodicity_tolerance: orbit_params.periodicity_tolerance,
            escape_radius: orbit_params.escape_radius,
        }
    }

    #[inline]
    fn get_map_value(&self, z: V) -> V
    {
        (self.f)(z, self.param)
    }

    #[inline]
    fn get_map_value_and_derivative(&self, z: V) -> (V, D)
    {
        (self.map_and_multiplier)(z, self.param)
    }

    #[inline]
    fn apply_map_to_slow(&mut self)
    {
        self.z_slow = (self.f)(self.z_slow, self.param);
    }

    #[inline]
    fn apply_map_and_update_multiplier(&mut self)
    {
        let (z_new, deriv) = (self.map_and_multiplier)(self.z_fast, self.param);
        self.multiplier *= deriv;
        self.z_fast = z_new;
    }

    #[inline]
    fn derivative(&self, z: V) -> D
    {
        (self.map_and_multiplier)(z, self.param).1
    }

    #[inline]
    fn early_bailout_result(&mut self) -> Option<EscapeResult<V, D>>
    where
        B: Fn(V, P) -> Option<EscapeResult<V, D>>,
    {
        (self.early_bailout)(self.z_slow, self.param)
    }

    pub fn reset(&mut self, param: P, start_point: V)
    {
        self.state = None;
        self.param = param;
        self.z_slow = start_point;
        self.z_fast = start_point;
        self.multiplier = D::one();
        self.iter = 0;
    }

    pub fn run_until_complete(&mut self) -> EscapeResult<V, D>
    where
        B: Fn(V, P) -> Option<EscapeResult<V, D>>,
    {
        if let Some(res) = self.early_bailout_result() {
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
        if let Some(state) = (self.stop_condition)(self.z_fast, self.param, self.iter) {
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

    fn compute_period(&self, tolerance: Real, patience: usize) -> Option<(Period, D)>
    {
        let mut z = self.z_fast;
        let mut dz: D;
        let mut mult = D::one();
        for i in 1..=patience {
            (z, dz) = self.get_map_value_and_derivative(z);
            mult *= dz;
            if z.dist_sqr(self.z_fast) <= tolerance {
                return Period::try_from(i).ok().map(|n| (n, mult));
            }
        }
        None
    }
}

impl<V, P, D, F, G, S, B> Iterator for CycleDetected<V, P, D, F, G, S, B>
where
    F: Fn(V, P) -> V,
    G: Fn(V, P) -> (V, D),
    S: Fn(V, P, Period) -> Option<EscapeResult<V, D>>,
    P: Copy,
    V: Norm<Real> + Dist<Real> + MaybeNan,
    D: Norm<Real> + std::ops::MulAssign + One,
{
    type Item = (V, Option<EscapeResult<V, D>>);

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
        } else if self.escape_radius.is_finite() {
            self.escape_radius = Real::NAN;
            Some((self.z_fast, self.state.clone()))
        } else {
            None
        }
    }
}
