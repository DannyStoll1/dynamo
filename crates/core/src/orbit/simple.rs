use super::{EscapeResult, OrbitParams};
use dynamo_common::prelude::*;
use num_traits::One;

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
        if self.iter > self.max_iter {
            self.state = Some(EscapeResult::Bounded);
            return;
        }

        let r = self.z.norm_sqr();
        if r > self.escape_radius || self.z.is_nan() {
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
        if self.iter == 0 {
            self.iter = 1;
            self.enforce_stop_condition();
            return Some((self.z, self.state.clone()));
        }

        if self.state.is_none() {
            self.apply_map();
            self.iter += 1;
            self.enforce_stop_condition();
            Some((self.z, self.state.clone()))
        } else if self.escape_radius.is_finite() {
            self.escape_radius = Real::NAN;
            Some((self.z, self.state.clone()))
        } else {
            None
        }
    }
}
