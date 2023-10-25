use dynamo_common::prelude::*;
use num_traits::One;

#[derive(Clone, Debug, Default)]
pub enum EscapeResult<V, D>
{
    Escaped
    {
        iters: Period,
        final_value: V,
    },
    Periodic
    {
        info: PointInfoPeriodic<D>,
        final_value: V,
    },
    KnownPotential(PointInfoKnownPotential<D>),
    #[default]
    Bounded,
}

pub struct OrbitParams
{
    pub max_iter: Period,
    pub min_iter: Period,
    pub periodicity_tolerance: Real,
    pub escape_radius: Real,
}

pub struct SimpleOrbit<V, P, F>
where
    F: Fn(V, P) -> V,
    P: Copy,
    V: Norm<Real>,
{
    f: F,
    param: P,
    max_iter: Period,
    escape_radius: Real,
    pub z: V,
    pub iter: Period,
    pub state: Option<EscapeResult<V, V>>,
}

impl<V, P, F> SimpleOrbit<V, P, F>
where
    F: Fn(V, P) -> V,
    P: Copy,
    V: Norm<Real> + MaybeNan,
{
    pub const fn new(f: F, z: V, param: P, max_iter: Period, escape_radius: Real) -> Self
    {
        Self {
            f,
            z,
            param,
            max_iter,
            escape_radius,
            iter: 0,
            state: None,
        }
    }

    #[inline]
    fn apply_map(&mut self)
    {
        self.z = (self.f)(self.z, self.param);
    }

    fn enforce_stop_condition(&mut self)
    {
        if self.iter > self.max_iter
        {
            self.state = Some(EscapeResult::Bounded);
            return;
        }

        let r = self.z.norm_sqr();
        if r > self.escape_radius || self.z.is_nan()
        {
            self.state = Some(EscapeResult::Escaped {
                // Subtract 1 to undo the offset from iteration start
                iters: self.iter - 1,
                final_value: self.z,
            });
        }
    }
}

impl<V, P, F> Iterator for SimpleOrbit<V, P, F>
where
    F: Fn(V, P) -> V,
    P: Copy,
    V: Norm<Real> + MaybeNan,
{
    type Item = (V, Option<EscapeResult<V, V>>);

    fn next(&mut self) -> Option<Self::Item>
    {
        if self.iter == 0
        {
            self.iter = 1;
            self.enforce_stop_condition();
            return Some((self.z, self.state.clone()));
        }

        if self.state.is_none()
        {
            self.apply_map();
            self.iter += 1;
            self.enforce_stop_condition();
            Some((self.z, self.state.clone()))
        }
        else if self.escape_radius.is_finite()
        {
            self.escape_radius = Real::NAN;
            Some((self.z, self.state.clone()))
        }
        else
        {
            None
        }
    }
}

pub struct CycleDetectedOrbitFloyd<V, P, D, F, G, B>
where
    F: Fn(V, P) -> V,
    G: Fn(V, P) -> (V, D),
    P: Copy,
    V: Norm<Real> + Dist<Real>,
    D: Norm<Real> + std::ops::MulAssign + One,
{
    f: F,
    map_and_multiplier: G,
    early_bailout: B,
    param: P,
    max_iter: Period,
    min_iter: Period,
    periodicity_tolerance: Real,
    escape_radius: Real,
    pub z_slow: V,
    pub z_fast: V,
    pub multiplier: D,
    pub iter: Period,
    pub state: Option<EscapeResult<V, D>>,
}

impl<V, P, D, F, G, B> CycleDetectedOrbitFloyd<V, P, D, F, G, B>
where
    F: Fn(V, P) -> V,
    G: Fn(V, P) -> (V, D),
    P: Copy,
    V: Norm<Real> + Dist<Real> + MaybeNan,
    D: Norm<Real> + std::ops::MulAssign + One,
{
    pub fn new(
        f: F,
        map_and_multiplier: G,
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
            param,
            z_slow: z,
            z_fast: z,
            multiplier: D::one(),
            iter: 0,
            state: None,
            max_iter: orbit_params.max_iter,
            min_iter: orbit_params.min_iter,
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
        if let Some(res) = self.early_bailout_result()
        {
            return res;
        }

        while self.state.is_none()
        {
            self.iter += 1;
            if self.iter % 2 == 1
            {
                self.apply_map_to_slow();
                self.apply_map_and_update_multiplier();
                self.enforce_stop_condition();
            }
            else
            {
                self.apply_map_and_update_multiplier();
                self.check_periodicity();
            }
        }
        #[allow(clippy::unwrap_used)]
        self.state.clone().unwrap()
    }

    #[inline]
    fn enforce_stop_condition(&mut self)
    {
        if self.iter > self.max_iter
        {
            self.state = Some(EscapeResult::Bounded);
            return;
        }
        if self.iter < self.min_iter
        {
            return;
        }

        let r = self.z_fast.norm_sqr();
        if r > self.escape_radius || self.z_fast.is_nan()
        {
            self.state = Some(EscapeResult::Escaped {
                iters: self.iter,
                final_value: self.z_fast,
            });
        }
    }

    fn check_periodicity(&mut self)
    {
        if self.iter > self.max_iter
        {
            self.state = Some(EscapeResult::Bounded);
            return;
        }
        if self.iter < self.min_iter
        {
            return;
        }

        let r = self.z_fast.norm_sqr();
        if r > self.escape_radius || self.z_fast.is_nan()
        {
            self.state = Some(EscapeResult::Escaped {
                iters: self.iter,
                final_value: self.z_fast,
            });
            return;
        }
        let error = self.z_fast.dist_sqr(self.z_slow);
        if error < self.periodicity_tolerance
        {
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
        for i in 1..=patience
        {
            (z, dz) = self.get_map_value_and_derivative(z);
            mult *= dz;
            if z.dist_sqr(self.z_fast) <= tolerance
            {
                return Period::try_from(i).ok().map(|n| (n, mult));
            }
        }
        None
    }
}

impl<V, P, D, F, G, B> Iterator for CycleDetectedOrbitFloyd<V, P, D, F, G, B>
where
    F: Fn(V, P) -> V,
    G: Fn(V, P) -> (V, D),
    P: Copy,
    V: Norm<Real> + Dist<Real> + MaybeNan,
    D: Norm<Real> + std::ops::MulAssign + One,
{
    type Item = (V, Option<EscapeResult<V, D>>);

    fn next(&mut self) -> Option<Self::Item>
    {
        if self.state.is_none()
        {
            let retval = self.z_fast;
            self.iter += 1;
            if self.iter % 2 == 1
            {
                self.apply_map_to_slow();
                self.apply_map_and_update_multiplier();
                self.enforce_stop_condition();
            }
            else
            {
                self.apply_map_and_update_multiplier();
                self.check_periodicity();
            }
            Some((retval, self.state.clone()))
        }
        else if self.escape_radius.is_finite()
        {
            self.escape_radius = Real::NAN;
            Some((self.z_fast, self.state.clone()))
        }
        else
        {
            None
        }
    }
}

// pub struct CycleDetectedOrbitBrent<V, P, D, F, G, C, B>
// where
//     F: Fn(V, P) -> V,
//     G: Fn(V, P) -> (V, D),
//     C: Fn(Period, V, V, P) -> EscapeResult<V, D>,
//     B: Fn(V, P) -> EscapeResult<V, D>,
// {
//     f: F,
//     map_and_multiplier: G,
//     check_periodicity: C,
//     early_bailout: B,
//     param: V,
//     pub z_slow: V,
//     pub z_fast: V,
//     pub multiplier: V,
//     pub iter: Period,
//     pub state: EscapeResult<V, D>,
//     period_limit: Period,
//     period: Period,
// }

// impl<V, P, D, F, G, C, B> CycleDetectedOrbitBrent<V, P, D, F, G, C, B>
// where
//     F: Fn(V, P) -> V,
//     G: Fn(V, P) -> (V, D),
//     C: Fn(Period, V, V, P) -> EscapeResult<V, D>,
//     B: Fn(V, P) -> EscapeResult<V, D>,
// {
//     pub fn new(
//         f: F,
//         map_and_multiplier: G,
//         check_periodicity: C,
//         early_bailout: B,
//         z: V,
//         param: V,
//     ) -> Self
//     {
//         let z_fast = f(z, param);
//         let multiplier = V::new(1., 0.);
//         Self {
//             f,
//             map_and_multiplier,
//             check_periodicity,
//             early_bailout,
//             param,
//             z_slow: z,
//             z_fast,
//             multiplier,
//             iter: 0,
//             state: EscapeResult::NotYetEscaped,
//             period_limit: 1,
//             period: 1,
//         }
//     }
//
//     fn apply_map(&self, z: V) -> V
//     {
//         (self.f)(z, self.param)
//     }
//
//     fn apply_map_and_update_multiplier(&mut self)
//     {
//         let (z_new, deriv) = (self.map_and_multiplier)(self.z_fast, self.param);
//         self.multiplier *= deriv;
//         self.z_fast = z_new;
//     }
//
//     fn check_early_bailout(&mut self)
//     {
//         self.state = (self.early_bailout)(self.z_slow, self.param);
//     }
//
//     fn derivative(&self, z: V) -> V
//     {
//         (self.map_and_multiplier)(z, self.param).1
//     }
//
//     pub fn reset(&mut self, param: V, start_point: V)
//     {
//         self.param = param;
//         self.z_slow = start_point;
//         self.z_fast = start_point;
//         self.multiplier = (1.).into();
//         self.iter = 0;
//         self.state = Some(EscapeResult::NotYetEscaped);
//         self.period = 1;
//         self.period_limit = 1;
//     }
//
//     pub fn run_until_complete(&mut self) -> EscapeResult<V, D>
//     {
//         self.check_early_bailout();
//
//         while matches!(self.state, EscapeResult::NotYetEscaped)
//         {
//             if self.period_limit == self.period
//             {
//                 self.z_slow = self.z_fast;
//                 self.period_limit *= 2;
//                 self.period = 0;
//             }
//             self.z_fast = self.apply_map(self.z_fast);
//             self.multiplier *= self.derivative(self.z_fast);
//
//             self.period += 1;
//             self.iter += 1;
//
//             self.state = (self.check_periodicity)(self.iter, self.z_slow, self.z_fast, self.param);
//         }
//         self.state
//     }
// }
//
// impl<V, P, D, F, G, C, B> Iterator for CycleDetectedOrbitBrent<V, P, D, F, G, C, B>
// where
//     F: Fn(V, P) -> V,
//     G: Fn(V, P) -> (V, D),
//     C: Fn(Period, V, V, P) -> EscapeResult<V, D>,
//     B: Fn(V, P) -> EscapeResult<V, D>,
// {
//     type Item = (V, EscapeResult<V, D>);
//
//     fn next(&mut self) -> Option<Self::Item>
//     {
//         if matches!(self.state, EscapeResult::NotYetEscaped)
//         {
//             let retval = self.z_fast;
//             if self.period_limit == self.period
//             {
//                 self.z_slow = self.z_fast;
//                 self.period_limit *= 2;
//                 self.period = 0;
//             }
//             self.apply_map_and_update_multiplier();
//
//             self.period += 1;
//             self.iter += 1;
//
//             self.state = (self.check_periodicity)(self.iter, self.z_slow, self.z_fast, self.param);
//             Some((retval, self.state))
//         }
//         else
//         {
//             None
//         }
//     }
// }

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct OrbitInfo<P, V, D>
{
    pub param: P,
    pub start: V,
    pub result: PointInfo<D>,
}
pub struct OrbitAndInfo<P, V, D>
{
    pub orbit: Vec<V>,
    pub info: OrbitInfo<P, V, D>,
}

impl<P, V, D> std::fmt::Display for OrbitInfo<P, V, D>
where
    P: Describe,
    V: std::fmt::Display,
    D: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        use PointInfo::*;
        let result_summary = match &self.result
        {
            Escaping { potential } => format!("Escaped, potential: {potential:.DISPLAY_PREC$}"),
            Periodic(data) | MarkedPoint { data, .. } => data.to_string(),
            PeriodicKnownPotential(data) => data.to_string(),
            Bounded => "Bounded (no cycle detected or period too high)".to_owned(),
            Wandering => "Wandering (appears to escape very slowly)".to_owned(),
        };
        write!(
            f,
            "Starting point: {start:.*}\n\
            {param_desc}\
            {result_summary}",
            DISPLAY_PREC,
            start = self.start,
            param_desc = self.param.describe_in_orbit_info()
        )
    }
}
