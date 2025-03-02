use crate::newton::Newton;
use crate::poly_traits::{Differentiable, DivideByAffine, Eval, MulConst, Normalize};
use crate::polynomial::Polynomial;
use num_complex::{Complex, Complex64};
use rand::{rngs::ThreadRng, Rng};

fn compute_cauchy_poly(poly: &Polynomial<Complex64>) -> Polynomial<f64>
{
    let coeffs = poly.coeffs.iter().map(|a| a.norm()).collect();
    Polynomial { coeffs }
}

pub struct JenkinsTraubSolver
{
    pub poly: Polynomial<Complex64>,
    h_poly: Polynomial<Complex64>,
    h_init: Option<Polynomial<Complex64>>,
    cauchy_root: f64,
    rng: ThreadRng,
    best_root: Complex64,
    best_norm: f64,
    stage_1_shift: Complex64,
}
impl JenkinsTraubSolver
{
    const M: usize = 12;
    const LIMIT_STAGE_1: usize = 50;
    const LIMIT_STAGE_2: usize = 160;
    const MAX_TRIES_STAGE_1: usize = 8;
    const MAX_TRIES_STAGE_2: usize = 4;
    const ERR: f64 = 1e-17_f64;
    const ZERO: Complex64 = Complex::new(0., 0.);
    const NAN: Complex64 = Complex::new(f64::NAN, f64::NAN);

    #[must_use]
    pub fn new(mut poly: Polynomial<Complex64>) -> Self
    {
        poly.normalize_inplace();
        let h_poly = poly.derivative();
        let cauchy_poly = compute_cauchy_poly(&poly);
        let cauchy_root = cauchy_poly.find_root_newton(1.0, Self::ERR).unwrap_or(1.0);
        let rng = rand::rng();

        Self {
            poly,
            h_poly,
            h_init: None,
            cauchy_root,
            rng,
            best_root: Self::NAN,
            best_norm: f64::INFINITY,
            stage_1_shift: Self::NAN,
        }
    }

    fn increment_h_poly_no_shift(&mut self)
    {
        let mut c = -self.poly.eval(Self::ZERO) / self.h_poly.eval(Self::ZERO);
        if c.is_nan() {
            c = -self.poly.eval(Self::ERR.into()) / self.h_poly.eval(Self::ERR.into());
        }
        self.h_poly.mul_const_assign(c);
        self.h_poly.add_with(&self.poly);
        self.h_poly.divide_by_var_inplace();
    }

    fn increment_h_poly(&mut self, shift: Complex64)
    {
        let c = -self.poly.eval(shift) / self.h_poly.eval(shift);
        self.h_poly.mul_const_assign(c);
        self.h_poly.add_with(&self.poly);
        self.h_poly.divide_by_affine_inplace(shift);
    }

    fn get_seed(&mut self) -> Complex64
    {
        let theta = self.rng.random_range(0.0..std::f64::consts::TAU);
        let z = Complex::new(0., theta).exp();
        self.cauchy_root * z
    }

    fn current_estimate(&self, shift: Complex64) -> Complex64
    {
        shift - self.poly.eval(shift) / self.h_poly.eval(shift)
    }

    /// Reset the H-polynomial to the value obtained after stage 0.
    /// Panics if called before stage 0 is complete.
    fn reset_h_poly(&mut self)
    {
        self.h_init
            .as_ref()
            .expect("reset_h_poly called before completion of stage 0")
            .coeffs
            .clone_into(&mut self.h_poly.coeffs);
    }

    /// Reset best root and distance
    const fn reset(&mut self)
    {
        self.best_root = Self::NAN;
        self.best_norm = f64::INFINITY;
    }

    fn stage_0(&mut self)
    {
        for _ in 0..Self::M {
            self.increment_h_poly_no_shift();
        }
        self.h_init = Some(self.h_poly.clone());
    }

    fn stage_1(&mut self) -> Option<Complex64>
    {
        let s = self.get_seed();

        let mut t_curr = Complex::from(f64::NAN);
        let mut t_next: Complex64;
        let mut was_close = false;

        for _ in 0..Self::LIMIT_STAGE_1 {
            self.increment_h_poly(s);
            t_next = self.current_estimate(s);

            if (t_curr - t_next).norm_sqr() < 0.25 * t_curr.norm_sqr() {
                if was_close {
                    return Some(t_next);
                }
                was_close = true;
            } else if s.is_nan() {
                return None;
            } else {
                was_close = false;
            }

            t_curr = t_next;
        }

        self.stage_1_shift = s;
        None
    }

    fn loop_stage_1(&mut self) -> Complex64
    {
        for _ in 0..Self::MAX_TRIES_STAGE_1 {
            if let Some(res) = self.stage_1() {
                return res;
            }
        }

        self.stage_1_shift
    }

    fn stage_2(&mut self, mut s: Complex64) -> Option<Complex64>
    {
        for _ in 0..Self::LIMIT_STAGE_2 {
            self.increment_h_poly(s);
            s = self.current_estimate(s);

            let norm = self.poly.eval(s).norm_sqr();
            if norm < Self::ERR {
                return Some(s);
            } else if norm < self.best_norm {
                self.best_root = s;
                self.best_norm = norm;
            }

            if s.is_nan() {
                return None;
            }
        }
        None
    }

    pub fn find_smallest_root(&mut self) -> Complex64
    {
        self.stage_0();

        for _ in 0..Self::MAX_TRIES_STAGE_2 {
            let s = self.loop_stage_1();

            if let Some(root) = self.stage_2(s) {
                return root;
            }
            self.reset_h_poly();
        }
        self.best_root
    }

    pub fn find_all_roots(&mut self) -> Vec<Complex64>
    {
        (0..self.poly.degree())
            .map(|_| {
                let r = self.find_smallest_root();
                self.poly.divide_by_affine_inplace(r);
                r
            })
            .collect()
    }
}
