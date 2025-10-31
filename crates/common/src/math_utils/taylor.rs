use std::ops::{Add, AddAssign, Mul, Sub};

use num_traits::Zero;

pub trait LinOps<Rhs = Self, Output = Self>:
    Add<Rhs, Output = Output>
    + Sub<Rhs, Output = Output>
    + Mul<Rhs, Output = Output>
    + Mul<f64, Output = Self>
    + AddAssign<Rhs>
    + Clone
    + Copy
    + Zero
{
}

impl<T, Rhs, Output> LinOps<Rhs, Output> for T where
    T: Add<Rhs, Output = Output>
        + Sub<Rhs, Output = Output>
        + Mul<Rhs, Output = Output>
        + Mul<f64, Output = Self>
        + AddAssign<Rhs>
        + Clone
        + Copy
        + Zero
{
}

/// Data needed to compute a 1nd-order (linear) approximation
pub struct Taylor1<T: LinOps, const N: usize>
{
    pub f:    T,
    pub grad: [T; N],
}
impl<T: LinOps, const N: usize> Taylor1<T, N>
{
    /// Estimate the value of f(z+delta) - f(z)
    #[inline]
    pub fn estimate_diff(&self, delta: [T; N]) -> T
    {
        dot_prod(self.grad, delta)
    }

    /// Estimate the value of f(z+delta)
    #[inline]
    pub fn estimate(&self, delta: [T; N]) -> T
    {
        self.f + self.estimate_diff(delta)
    }
}

/// Data needed to compute a 2rd-order (quadratic) approximation
pub struct Taylor2<T: LinOps, const N: usize>
{
    pub taylor1: Taylor1<T, N>,
    pub hess:    [[T; N]; N],
}
impl<T: LinOps, const N: usize> Taylor2<T, N>
{
    /// Estimate the value of f(z+delta) - f(z)
    pub fn estimate_diff(&self, delta: [T; N]) -> T
    {
        self.taylor1.estimate_diff(delta) + quad_form(delta, self.hess) * 0.5_f64
    }

    /// Estimate the value of f(z+delta)
    #[inline]
    pub fn estimate(&self, delta: [T; N]) -> T
    {
        self.estimate_diff(delta) + quad_form(delta, self.hess) * 0.5_f64
    }
}

/// Evaluate a quadratic form defined by matrix `mat` on a vector `v`
fn quad_form<const N: usize, T: LinOps>(v: [T; N], mat: [[T; N]; N]) -> T
{
    let v_at = matvec(mat, v);
    dot_prod(v_at, v)
}

/// Evaluate the dot product of two vectors
fn dot_prod<const N: usize, T: LinOps>(v: [T; N], w: [T; N]) -> T
{
    (0..N).map(|i| v[i] * w[i]).fold(T::zero(), |a, b| a + b)
}

fn matvec<const N: usize, const M: usize, T: LinOps>(mat: [[T; N]; M], v: [T; N]) -> [T; M]
{
    std::array::from_fn(|row| dot_prod(mat[row], v))
}
