use dynamo_common::prelude::*;
use num_traits::One;

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
    pub state: EscapeState<V, V>,
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
            state: EscapeState::NotYetEscaped,
        }
    }

    fn apply_map(&self, z: V) -> V
    {
        (self.f)(z, self.param)
    }

    fn stop_condition(&self, iter: Period, z: V) -> EscapeState<V, V>
    {
        if iter > self.max_iter
        {
            return EscapeState::Bounded;
        }

        let r = z.norm_sqr();
        if r > self.escape_radius || z.is_nan()
        {
            return EscapeState::Escaped {
                iters: iter,
                final_value: z,
            };
        }
        EscapeState::NotYetEscaped
    }

    // pub fn from_plane(plane: Box<dyn ParameterPlane>, param: V) -> Self
    // {
    //     let start = plane.start_point(param);
    //     Self::new(
    //         |z, c| plane.map(z, c),
    //         |i, z| plane.stop_condition(i, z),
    //         start,
    //         param,
    //     )
    // }
}

impl<V, P, F> Iterator for SimpleOrbit<V, P, F>
where
    F: Fn(V, P) -> V,
    P: Copy,
    V: Norm<Real> + MaybeNan,
{
    type Item = (V, EscapeState<V, V>);

    fn next(&mut self) -> Option<Self::Item>
    {
        if self.iter == 0
        {
            self.state = self.stop_condition(self.iter, self.z);
            self.iter = 1;
            return Some((self.z, self.state));
        }

        if matches!(self.state, EscapeState::NotYetEscaped)
        {
            self.z = self.apply_map(self.z);
            self.state = self.stop_condition(self.iter, self.z);
            self.iter += 1;
            Some((self.z, self.state))
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
    B: Fn(V, P) -> EscapeState<V, D>,
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
    pub state: EscapeState<V, D>,
}

impl<V, P, D, F, G, B> CycleDetectedOrbitFloyd<V, P, D, F, G, B>
where
    F: Fn(V, P) -> V,
    G: Fn(V, P) -> (V, D),
    B: Fn(V, P) -> EscapeState<V, D>,
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
            state: EscapeState::NotYetEscaped,
            max_iter: orbit_params.max_iter,
            min_iter: orbit_params.min_iter,
            periodicity_tolerance: orbit_params.periodicity_tolerance,
            escape_radius: orbit_params.escape_radius,
        }
    }

    fn get_map_value(&self, z: V) -> V
    {
        (self.f)(z, self.param)
    }

    fn get_map_value_and_derivative(&self, z: V) -> (V, D)
    {
        (self.map_and_multiplier)(z, self.param)
    }

    fn apply_map_and_update_multiplier(&mut self)
    {
        let (z_new, deriv) = (self.map_and_multiplier)(self.z_fast, self.param);
        self.multiplier *= deriv;
        self.z_fast = z_new;
    }

    fn derivative(&self, z: V) -> D
    {
        (self.map_and_multiplier)(z, self.param).1
    }

    fn check_early_bailout(&mut self)
    {
        self.state = (self.early_bailout)(self.z_slow, self.param);
    }

    pub fn reset(&mut self, param: P, start_point: V)
    {
        self.param = param;
        self.z_slow = start_point;
        self.z_fast = start_point;
        self.multiplier = D::one();
        self.iter = 0;
        self.state = EscapeState::NotYetEscaped;
    }

    pub fn run_until_complete(&mut self) -> EscapeState<V, D>
    {
        self.check_early_bailout();

        while matches!(self.state, EscapeState::NotYetEscaped)
        {
            self.iter += 1;
            if self.iter % 2 == 1
            {
                self.z_slow = self.get_map_value(self.z_slow);
                self.apply_map_and_update_multiplier();
                self.state = self.stop_condition(self.iter, self.z_fast);
            }
            else
            {
                self.apply_map_and_update_multiplier();
                self.state =
                    self.check_periodicity(self.iter, self.z_slow, self.z_fast, self.param);
            }
        }
        self.state
    }

    fn stop_condition(&self, iter: Period, z: V) -> EscapeState<V, D>
    {
        if iter > self.max_iter
        {
            return EscapeState::Bounded;
        }
        if iter < self.min_iter
        {
            return EscapeState::NotYetEscaped;
        }

        let r = z.norm_sqr();
        if r > self.escape_radius || z.is_nan()
        {
            return EscapeState::Escaped {
                iters: iter,
                final_value: z,
            };
        }
        EscapeState::NotYetEscaped
    }

    fn check_periodicity(&self, iter: Period, z_slow: V, z_fast: V, param: P) -> EscapeState<V, D>
    {
        if iter > self.max_iter
        {
            return EscapeState::Bounded;
        }
        if iter < self.min_iter
        {
            return EscapeState::NotYetEscaped;
        }

        let r = z_fast.norm_sqr();
        if r > self.escape_radius || z_fast.is_nan()
        {
            EscapeState::Escaped {
                iters: iter,
                final_value: z_fast,
            }
        }
        else if z_fast.dist_sqr(z_slow) < self.periodicity_tolerance
        {
            if let Some((period, multiplier)) = self.compute_period(
                z_fast,
                param,
                self.periodicity_tolerance.powf(0.75),
                iter as usize,
            )
            {
                EscapeState::Periodic(PointInfoPeriodic {
                    value: z_fast,
                    preperiod: iter,
                    period,
                    multiplier,
                    final_error: z_fast.dist_sqr(z_slow),
                })
            }
            else
            {
                EscapeState::NotYetEscaped
            }
        }
        else
        {
            EscapeState::NotYetEscaped
        }
    }

    fn compute_period(&self, z0: V, _c: P, tolerance: Real, patience: usize)
        -> Option<(Period, D)>
    {
        let mut z = z0;
        let mut dz: D;
        let mut mult = D::one();
        for i in 1..=patience
        {
            (z, dz) = self.get_map_value_and_derivative(z);
            mult *= dz;
            if z.dist_sqr(z0) <= tolerance
            {
                if let Ok(period) = Period::try_from(i)
                {
                    return Some((period, mult));
                }
                return None;
            }
        }
        None
    }
}

impl<V, P, D, F, G, B> Iterator for CycleDetectedOrbitFloyd<V, P, D, F, G, B>
where
    F: Fn(V, P) -> V,
    G: Fn(V, P) -> (V, D),
    B: Fn(V, P) -> EscapeState<V, D>,
    P: Copy,
    V: Norm<Real> + Dist<Real> + MaybeNan,
    D: Norm<Real> + std::ops::MulAssign + One,
{
    type Item = (V, EscapeState<V, D>);

    fn next(&mut self) -> Option<Self::Item>
    {
        if matches!(self.state, EscapeState::NotYetEscaped)
        {
            let retval = self.z_fast;
            self.iter += 1;
            if self.iter % 2 == 1
            {
                self.z_slow = self.get_map_value(self.z_slow);
                self.apply_map_and_update_multiplier();
                self.state = self.stop_condition(self.iter, self.z_fast);
            }
            else
            {
                self.apply_map_and_update_multiplier();
                // dbg!(self.z_fast.dist_sqr(self.z_slow));
                self.state =
                    self.check_periodicity(self.iter, self.z_slow, self.z_fast, self.param);
            }
            Some((retval, self.state))
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
//     C: Fn(Period, V, V, P) -> EscapeState<V, D>,
//     B: Fn(V, P) -> EscapeState<V, D>,
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
//     pub state: EscapeState<V, D>,
//     period_limit: Period,
//     period: Period,
// }

// impl<V, P, D, F, G, C, B> CycleDetectedOrbitBrent<V, P, D, F, G, C, B>
// where
//     F: Fn(V, P) -> V,
//     G: Fn(V, P) -> (V, D),
//     C: Fn(Period, V, V, P) -> EscapeState<V, D>,
//     B: Fn(V, P) -> EscapeState<V, D>,
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
//             state: EscapeState::NotYetEscaped,
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
//         self.state = EscapeState::NotYetEscaped;
//         self.period = 1;
//         self.period_limit = 1;
//     }
//
//     pub fn run_until_complete(&mut self) -> EscapeState<V, D>
//     {
//         self.check_early_bailout();
//
//         while matches!(self.state, EscapeState::NotYetEscaped)
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
//     C: Fn(Period, V, V, P) -> EscapeState<V, D>,
//     B: Fn(V, P) -> EscapeState<V, D>,
// {
//     type Item = (V, EscapeState<V, D>);
//
//     fn next(&mut self) -> Option<Self::Item>
//     {
//         if matches!(self.state, EscapeState::NotYetEscaped)
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
