use std::ops::{AddAssign, Neg};

use num_traits::{MulAddAssign, Num, NumAssignOps};

pub trait VariableOps: Clone + Num + NumAssignOps + Neg<Output = Self> + MulAddAssign {}
impl<T> VariableOps for T where T: Clone + Num + NumAssignOps + Neg<Output = Self> + MulAddAssign {}

pub trait PolynomialOps:
    Clone + Eval + Differentiable + Normalize + AddAssign + MulConst + DivideByAffine
{
}
impl<T> PolynomialOps for T where
    T: Clone + Eval + Differentiable + Normalize + AddAssign + MulConst + DivideByAffine
{
}

pub trait HasVar
{
    type Var: VariableOps;
}

pub trait MulConst: HasVar
{
    #[must_use]
    fn mul_const(self, c: Self::Var) -> Self;
    fn mul_const_assign(&mut self, c: Self::Var);
}

pub trait Eval: HasVar
{
    fn eval(&self, x: Self::Var) -> Self::Var;
}

pub trait Normalize: Sized
{
    /// Return a monic polynomial proportional to the input.
    #[must_use]
    fn normalize(self) -> Self;

    /// Normalize a polynomial inplace to make it monic.
    fn normalize_inplace(&mut self);
}

pub trait Differentiable: Sized
{
    #[must_use]
    fn derivative(&self) -> Self;
}

pub trait DivideByAffine: HasVar
{
    /// Divide self by x - a0
    #[must_use]
    fn divide_by_affine(&self, a0: Self::Var) -> Self;

    /// Divide self by x - a0 inplace
    fn divide_by_affine_inplace(&mut self, a0: Self::Var);

    /// Divide self by x
    #[must_use]
    fn divide_by_var(&self) -> Self;

    /// Divide self by x inplace
    fn divide_by_var_inplace(&mut self);
}
