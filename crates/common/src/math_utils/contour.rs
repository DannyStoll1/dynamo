use crate::prelude::{Cplx, Dist, Norm, Real};

const I: Cplx = Cplx::new(0., 1.);

#[derive(Copy, Clone, Debug, Default)]
pub enum Error
{
    #[default]
    FunctionUndefined,
    NanEncountered,
    Converged,
    LoopDetected,
}

pub struct LevelCurveParams
{
    step_size: Real,
    max_steps: usize,
    return_radius: Real,
}
impl LevelCurveParams
{
    #[must_use]
    pub const fn step_size(mut self, step_size: Real) -> Self
    {
        self.step_size = step_size;
        self
    }

    #[must_use]
    pub const fn return_radius(mut self, return_radius: Real) -> Self
    {
        self.return_radius = return_radius;
        self
    }

    #[must_use]
    pub const fn max_steps(mut self, max_steps: usize) -> Self
    {
        self.max_steps = max_steps;
        self
    }

    #[must_use]
    pub fn contour<D, FD, T>(
        self,
        point: Cplx,
        dmap: D,
        map_dmap: FD,
    ) -> Option<LevelCurve<D, FD, T>>
    where
        D: Fn(Cplx) -> Option<Cplx>,
        FD: Fn(Cplx) -> Option<(T, Cplx)>,
        T: Norm<Real>
            + std::ops::Sub<Output = T>
            + std::ops::Div<Cplx, Output = Cplx>
            + std::fmt::Debug,
    {
        LevelCurve::try_new(self, point, dmap, map_dmap)
    }
}
impl Default for LevelCurveParams
{
    fn default() -> Self
    {
        Self {
            step_size: 1e-2,
            max_steps: 20000,
            return_radius: 1e-2,
        }
    }
}

pub struct LevelCurve<D, FD, T>
where
    D: Fn(Cplx) -> Option<Cplx>,
    FD: Fn(Cplx) -> Option<(T, Cplx)>,
    T: Norm<Real> + std::ops::Sub<Output = T> + std::ops::Div<Cplx, Output = Cplx>,
{
    params: LevelCurveParams,
    point: Cplx,
    deriv: Cplx,
    target: T,
    dmap: D,
    map_dmap: FD,
    has_exited_return_radius: bool,
}

impl<D, FD, T> LevelCurve<D, FD, T>
where
    D: Fn(Cplx) -> Option<Cplx>,
    FD: Fn(Cplx) -> Option<(T, Cplx)>,
    T: Norm<Real>
        + std::ops::Sub<Output = T>
        + std::ops::Div<Cplx, Output = Cplx>
        + std::fmt::Debug,
{
    pub fn try_new(params: LevelCurveParams, point: Cplx, dmap: D, map_dmap: FD) -> Option<Self>
    {
        let (target, deriv) = (map_dmap)(point)?;
        let deriv = target / deriv.conj();
        Some(Self {
            params,
            point,
            deriv,
            target,
            dmap,
            map_dmap,
            has_exited_return_radius: false,
        })
    }

    fn step_vector(&self, t: Cplx) -> Option<Cplx>
    {
        let (f, df) = (self.map_dmap)(t)?;
        Some(f / df.conj() * I)
    }

    fn rk_step(&self) -> Option<Cplx>
    {
        let t = self.point;
        let h = self.params.step_size;

        let k0 = self.deriv * I;
        let k1 = self.step_vector(t + 0.5 * h * k0)?;
        let k2 = self.step_vector(t + 0.5 * h * k1)?;
        let k3 = self.step_vector(t + h * k2)? * I;
        Some(h / 6.0 * (k0 + 2. * (k1 + k2) + k3))
    }

    fn apply_newton_correction(&mut self) -> Option<()>
    {
        let (val, d_val) = (self.map_dmap)(self.point)?;
        self.deriv = val / d_val.conj();

        // The perturbation vector is parallel to d_val
        let correction = (self.target - val) / d_val.conj();
        if correction.norm_sqr() > self.params.step_size {
            return None;
        }

        self.point += correction;

        Some(())
    }

    #[inline]
    fn do_step(&mut self, seed: Cplx) -> Result<(), Error>
    {
        let dt = self.rk_step().ok_or(Error::FunctionUndefined)?;

        self.point -= dt;

        self.apply_newton_correction()
            .ok_or(Error::FunctionUndefined)?;

        if self.point.is_nan() {
            Err(Error::NanEncountered)
        } else if self.point.dist_sqr(seed) < self.params.return_radius {
            Err(Error::LoopDetected)
        } else {
            Ok(())
        }
    }

    pub fn compute(mut self) -> Vec<Cplx>
    {
        let seed = self.point;
        let mut t_list = vec![self.point];

        for _k in 0..self.params.max_steps {
            match self.do_step(seed) {
                Ok(()) => {
                    self.has_exited_return_radius = true;
                    t_list.push(self.point);
                }
                Err(Error::LoopDetected) if self.has_exited_return_radius => {
                    // a few extra iterations to fill in the gap
                    for _ in 0..25 {
                        t_list.push(self.point);
                        let _ = self.do_step(seed);
                    }
                    t_list.push(self.point);
                    break;
                }
                Err(Error::LoopDetected) => {
                    t_list.push(self.point);
                }
                Err(_) => break,
            }
        }

        t_list
    }
}

pub struct GradientCurveParams
{
    step_size: Real,
    max_steps: usize,
    tolerance: Real,
    escape_radius: Real,
}
impl GradientCurveParams
{
    #[must_use]
    pub const fn tolerance(mut self, tolerance: Real) -> Self
    {
        self.tolerance = tolerance;
        self
    }

    #[must_use]
    pub const fn escape_radius(mut self, radius: Real) -> Self
    {
        self.escape_radius = radius;
        self
    }

    #[must_use]
    pub const fn step_size(mut self, step_size: Real) -> Self
    {
        self.step_size = step_size;
        self
    }

    #[must_use]
    pub const fn max_steps(mut self, max_steps: usize) -> Self
    {
        self.max_steps = max_steps;
        self
    }

    #[must_use]
    pub const fn contour<D>(self, point: Cplx, dmap: D) -> GradientAscent<D>
    where
        D: Fn(Cplx) -> Option<Cplx>,
    {
        GradientAscent::new(self, point, dmap)
    }
}

pub struct GradientAscent<D>
where
    D: Fn(Cplx) -> Option<Cplx>,
{
    params: GradientCurveParams,
    point: Cplx,
    direction: D,
}

impl<D> GradientAscent<D>
where
    D: Fn(Cplx) -> Option<Cplx>,
{
    pub const fn new(params: GradientCurveParams, point: Cplx, direction: D) -> Self
    {
        Self {
            params,
            point,
            direction,
        }
    }

    fn rk_step(&self) -> Option<Cplx>
    {
        let t = self.point;
        let h = self.params.step_size;

        let k0 = (self.direction)(t)?;
        let k1 = (self.direction)(t + 0.5 * h * k0)?;
        let k2 = (self.direction)(t + 0.5 * h * k1)?;
        let k3 = (self.direction)(t + h * k2)?;
        Some(h / 6.0 * (k0 + 2. * (k1 + k2) + k3))
    }

    #[inline]
    fn do_step(&mut self) -> Result<(), Error>
    {
        let dt = self.rk_step().ok_or(Error::FunctionUndefined)?;

        self.point += dt;

        if self.point.is_nan() {
            return Err(Error::NanEncountered);
        }
        let r = dt.norm_sqr();
        if r < self.params.tolerance || r > self.params.escape_radius {
            Err(Error::Converged)
        } else {
            Ok(())
        }
    }

    pub fn compute(mut self) -> Vec<Cplx>
    {
        let mut t_list = vec![self.point];

        for _k in 0..self.params.max_steps {
            match self.do_step() {
                Ok(()) => {
                    t_list.push(self.point);
                }
                Err(_) => return t_list,
            }
        }

        t_list
    }
}
