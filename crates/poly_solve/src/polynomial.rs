use std::cmp::Ordering;
use std::collections::{VecDeque, vec_deque};
use std::ops::{Add, AddAssign};

use derive_more::From;
use itertools::Itertools;
use num_traits::{NumOps, Zero};

use crate::newton::Newton;
use crate::normed::Semimetric;
use crate::poly_traits::{
    Differentiable, DivideByAffine, Eval, HasVar, MulConst, Normalize, VariableOps,
};
use crate::utils::Collapse;

#[derive(Clone, PartialEq, Eq, Debug, From)]
pub struct Polynomial<T>
{
    /// Coefficients of the polynomial, starting with constant term first
    pub coeffs: VecDeque<T>,
}

impl<T> Polynomial<T>
{
    const ZERO: Self = Self {
        coeffs: VecDeque::new(),
    };

    /// Degree of the polynomial
    #[must_use]
    pub fn degree(&self) -> i32
    {
        (self.coeffs.len() - 1) as i32
    }

    /// 1 + degree of the polynomial
    #[must_use]
    pub fn size(&self) -> usize
    {
        self.coeffs.len()
    }

    /// Other must have lower degree for this to be correct
    fn add_assign_lower_degree_poly(&mut self, other: &Self)
    where
        T: Clone + AddAssign,
    {
        self.coeffs.iter_mut().zip(other.iter()).for_each(|(a, b)| {
            *a += b.clone();
        });
    }

    fn clear_leading_zeros(&mut self)
    where
        T: Zero,
    {
        while self.coeffs.back().is_some_and(Zero::is_zero) {
            self.coeffs.pop_back();
        }
    }

    #[must_use]
    pub fn iter(&self) -> vec_deque::Iter<'_, T>
    {
        self.coeffs.iter()
    }

    #[must_use]
    pub fn iter_mut(&mut self) -> vec_deque::IterMut<'_, T>
    {
        self.coeffs.iter_mut()
    }

    pub fn add_with(&mut self, rhs: &Self)
    where
        T: Clone + Zero + AddAssign,
    {
        match rhs.size().cmp(&self.size()) {
            Ordering::Less => {
                self.add_assign_lower_degree_poly(rhs);
                return;
            }
            Ordering::Equal => {
                self.add_assign_lower_degree_poly(rhs);
                self.clear_leading_zeros();
                return;
            }
            Ordering::Greater => {}
        }

        let a_s = self.iter_mut();
        let mut b_s = rhs.iter().cloned();
        for a in a_s {
            #[allow(clippy::unwrap_used)]
            let b = b_s.next().unwrap(); // Guaranteed to be Some since rhs has higher degree
            *a += b;
        }
        self.coeffs.extend(b_s);
    }
}

impl<'a, T> IntoIterator for &'a Polynomial<T>
{
    type Item = &'a T;
    type IntoIter = vec_deque::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter
    {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Polynomial<T>
{
    type Item = &'a mut T;
    type IntoIter = vec_deque::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter
    {
        self.iter_mut()
    }
}

impl<R, T, const N: usize> From<[R; N]> for Polynomial<T>
where
    T: Clone + From<R>,
{
    fn from(coeff_arr: [R; N]) -> Self
    {
        coeff_arr.into_iter().map(T::from).collect()
    }
}

impl<T> From<Vec<T>> for Polynomial<T>
{
    fn from(coeffs: Vec<T>) -> Self
    {
        Self {
            coeffs: coeffs.into(),
        }
    }
}

impl<T: Clone> From<&[T]> for Polynomial<T>
{
    fn from(coeffs: &[T]) -> Self
    {
        Self {
            coeffs: coeffs.iter().cloned().collect(),
        }
    }
}

impl<T> FromIterator<T> for Polynomial<T>
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self
    {
        let coeffs: VecDeque<T> = iter.into_iter().collect();
        Self { coeffs }
    }
}

impl<T> IntoIterator for Polynomial<T>
{
    type Item = T;
    type IntoIter = std::collections::vec_deque::IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter
    {
        self.coeffs.into_iter()
    }
}

impl<T: VariableOps> HasVar for Polynomial<T>
{
    type Var = T;
}

impl<T> Eval for Polynomial<T>
where
    T: VariableOps,
{
    fn eval(&self, x: Self::Var) -> Self::Var
    {
        let mut u = Self::Var::zero();
        for a in self.iter().rev().cloned() {
            u.mul_add_assign(x.clone(), a);
        }
        u
    }
}

impl<T> Normalize for Polynomial<T>
where
    T: VariableOps,
{
    fn normalize(self) -> Self
    {
        let Some(an) = self.coeffs.back().cloned() else {
            return Self::ZERO;
        };
        self.into_iter().map(|a| a / an.clone()).collect()
    }

    fn normalize_inplace(&mut self)
    {
        self.clear_leading_zeros();
        let Some(an_ref) = self.coeffs.back() else {
            return;
        };
        let an = an_ref.clone();
        self.coeffs.iter_mut().for_each(|a| *a /= an.clone());
    }
}

// TODO: find a way to remove the From<f64> requirement
impl<T> Differentiable for Polynomial<T>
where
    T: Clone + NumOps + From<f64>,
{
    fn derivative(&self) -> Self
    {
        let coeffs = self
            .coeffs
            .iter()
            .cloned()
            .enumerate()
            .skip(1)
            .map(|(i, x)| T::from(i as f64) * x)
            .collect();
        Self { coeffs }
    }
}

impl<T: VariableOps> DivideByAffine for Polynomial<T>
{
    fn divide_by_var(&self) -> Self
    {
        let coeffs = self.coeffs.iter().skip(1).cloned().collect();
        Self { coeffs }
    }

    fn divide_by_var_inplace(&mut self)
    {
        self.coeffs.pop_front();
    }

    /// Synthetic division by (x - a0)
    fn divide_by_affine(&self, a0: Self::Var) -> Self
    {
        let mut quotient = self.clone();
        quotient.divide_by_affine_inplace(a0);
        quotient
    }

    /// Synthetic division inplace by (x - a0)
    fn divide_by_affine_inplace(&mut self, a0: Self::Var)
    {
        let mut u = Self::Var::zero();
        self.coeffs.iter_mut().skip(1).rev().for_each(|a| {
            u.mul_add_assign(a0.clone(), a.clone());
            *a = u.clone();
        });
        self.coeffs.pop_front();
    }
}

impl<T: VariableOps> Add for Polynomial<T>
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self
    {
        let mut coeffs = self
            .coeffs
            .into_iter()
            .zip_longest(rhs.coeffs)
            .map(|ab| ab.collapse(|a, b| a + b))
            .collect::<VecDeque<_>>();
        while let Some(a) = coeffs.back() {
            if a.is_zero() {
                coeffs.pop_back();
            } else {
                break;
            }
        }
        Self { coeffs }
    }
}

impl<T: VariableOps> AddAssign for Polynomial<T>
{
    fn add_assign(&mut self, rhs: Self)
    {
        self.add_with(&rhs);
    }
}

impl<T: VariableOps> MulConst for Polynomial<T>
{
    fn mul_const(self, c: Self::Var) -> Self
    {
        self.into_iter().map(|x| x * c.clone()).collect()
    }

    fn mul_const_assign(&mut self, c: Self::Var)
    {
        self.iter_mut().for_each(|x| *x *= c.clone());
    }
}

impl<T> Newton for Polynomial<T> where T: VariableOps + Semimetric + From<f64> {}
