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

#[derive(Clone, Debug)]
pub struct LevelCurveParams
{
    step_size: Real,
    max_steps: usize,
    return_radius: Real,
    use_distance_estimation: bool,
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
    pub const fn use_distance_estimation(mut self) -> Self
    {
        self.use_distance_estimation = true;
        self
    }

    #[must_use]
    pub fn contour<FD, T>(self, point: Cplx, map_dmap: FD) -> Option<LevelCurve<FD, T>>
    where
        FD: Fn(Cplx) -> Option<(T, Cplx)>,
        T: Norm<Real>
            + std::ops::Sub<Output = T>
            + std::ops::Div<Cplx, Output = Cplx>
            + std::fmt::Debug,
    {
        LevelCurve::try_new(self, point, map_dmap)
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
            use_distance_estimation: false,
        }
    }
}

pub struct LevelCurve<FD, T>
where
    FD: Fn(Cplx) -> Option<(T, Cplx)>,
    T: Norm<Real> + std::ops::Sub<Output = T> + std::ops::Div<Cplx, Output = Cplx>,
{
    params: LevelCurveParams,
    seed: Cplx,
    point: Cplx,
    deriv: Cplx,
    target: T,
    map_dmap: FD,
    has_exited_return_radius: bool,
}

impl<FD, T> LevelCurve<FD, T>
where
    FD: Fn(Cplx) -> Option<(T, Cplx)>,
    T: Norm<Real>
        + std::ops::Sub<Output = T>
        + std::ops::Div<Cplx, Output = Cplx>
        + std::fmt::Debug,
{
    pub fn try_new(params: LevelCurveParams, point: Cplx, map_dmap: FD) -> Option<Self>
    {
        let (target, deriv) = (map_dmap)(point)?;
        let deriv = target / deriv.conj();
        Some(Self {
            params,
            seed: point,
            point,
            deriv,
            target,
            map_dmap,
            has_exited_return_radius: false,
        })
    }

    fn step_vector(&self, t: Cplx) -> Option<Cplx>
    {
        let (f, df) = (self.map_dmap)(t)?;
        if self.params.use_distance_estimation {
            Some(f / df.conj() * I)
        } else {
            Some(df * I)
        }
    }

    fn rk_step(&self) -> Option<Cplx>
    {
        let t = self.point;
        let h = self.params.step_size;

        let k0 = h * self.deriv * I;
        let k1 = h * self.step_vector(t + 0.5 * k0)?;
        let k2 = h * self.step_vector(t + 0.5 * k1)?;
        let k3 = h * self.step_vector(t + k2)?;
        Some((k0 + 2. * (k1 + k2) + k3) / 6.0)
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
    fn do_step(&mut self) -> Result<(), Error>
    {
        let dt = self.rk_step().ok_or(Error::FunctionUndefined)?;

        self.point -= dt;

        self.apply_newton_correction()
            .ok_or(Error::FunctionUndefined)?;

        if self.point.is_nan() {
            Err(Error::NanEncountered)
        } else if self.point.dist_sqr(self.seed) < self.params.return_radius {
            Err(Error::LoopDetected)
        } else {
            Ok(())
        }
    }

    fn close_loop(&mut self, t_list: &mut Vec<Cplx>)
    {
        let mut dist = Real::INFINITY;
        let mut new_dist = self.point.dist_sqr(self.seed);
        while new_dist < dist {
            dist = new_dist;
            let _ = self.do_step();
            t_list.push(self.point);
            new_dist = self.point.dist_sqr(self.seed);
        }
    }

    pub fn compute(mut self) -> Vec<Cplx>
    {
        let mut t_list = vec![self.point];

        for _k in 0..self.params.max_steps {
            match self.do_step() {
                Ok(()) => {
                    self.has_exited_return_radius = true;
                    t_list.push(self.point);
                }
                Err(Error::LoopDetected) if self.has_exited_return_radius => {
                    self.close_loop(&mut t_list);
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

#[derive(Clone, Debug)]
pub struct IntegralCurveParams
{
    step_size: Real,
    max_steps: usize,
    convergence_radius: Real,
    escape_radius: Real,
}
impl IntegralCurveParams
{
    #[must_use]
    pub const fn convergence_radius(mut self, radius: Real) -> Self
    {
        self.convergence_radius = radius;
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
    pub const fn contour<D>(self, point: Cplx, dmap: D) -> IntegralCurve<D>
    where
        D: Fn(Cplx) -> Option<Cplx>,
    {
        IntegralCurve::new(self, point, dmap)
    }
}
impl Default for IntegralCurveParams
{
    fn default() -> Self
    {
        Self {
            step_size: 1e-2,
            max_steps: 20000,
            escape_radius: 1e2,
            convergence_radius: 1e-6,
        }
    }
}

pub struct IntegralCurve<D>
where
    D: Fn(Cplx) -> Option<Cplx>,
{
    params: IntegralCurveParams,
    point: Cplx,
    direction: D,
}

impl<D> IntegralCurve<D>
where
    D: Fn(Cplx) -> Option<Cplx>,
{
    pub const fn new(params: IntegralCurveParams, point: Cplx, direction: D) -> Self
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

        let k0 = h * (self.direction)(t)?;
        let k1 = h * (self.direction)(t + k0 / 3.0)?;
        let k2 = h * (self.direction)(t - k0 / 3.0 + k1)?;
        let k3 = h * (self.direction)(t + k0 - k1 + k2)?;
        Some(0.125 * (k0 + 3. * (k1 + k2) + k3))
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
        if r < self.params.convergence_radius || r > self.params.escape_radius {
            Err(Error::Converged)
        } else {
            Ok(())
        }
    }

    pub fn compute(mut self) -> Option<Vec<Cplx>>
    {
        let _ = (self.direction)(self.point)?;

        let mut t_list = vec![self.point];

        for _k in 0..self.params.max_steps {
            match self.do_step() {
                Ok(()) => {
                    t_list.push(self.point);
                }
                Err(_) => return Some(t_list),
            }
        }

        Some(t_list)
    }
}
