use super::{param_stack::Summarize, Cplx, Real};
use crate::consts::ZERO;
use derive_more::{Add, Display, From, Sub};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub trait Norm<R>: Copy
{
    fn norm(&self) -> R;
    fn norm_sqr(&self) -> R;
    fn arg(&self) -> R;
    fn is_nan(&self) -> bool;
}

impl Norm<Real> for Cplx
{
    #[inline]
    fn norm(&self) -> Real
    {
        <Self>::norm(*self)
    }
    #[inline]
    fn norm_sqr(&self) -> Real
    {
        <Self>::norm_sqr(self)
    }
    #[inline]
    fn arg(&self) -> Real
    {
        <Self>::arg(*self)
    }
    #[inline]
    fn is_nan(&self) -> bool
    {
        <Self>::is_nan(*self)
    }
}

impl Norm<Real> for Point
{
    fn norm(&self) -> Real
    {
        self.norm_sqr().sqrt()
    }

    fn norm_sqr(&self) -> Real
    {
        self.x.mul_add(self.x, self.y * self.y)
    }

    fn arg(&self) -> Real
    {
        self.y.atan2(self.x)
    }

    fn is_nan(&self) -> bool
    {
        self.x.is_nan() || self.y.is_nan()
    }
}
#[derive(Default, Clone, Copy, Debug, Add, Sub, Display, From, PartialEq)]
#[display(fmt = "({x}, {y})")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Point
{
    pub x: Real,
    pub y: Real,
}
impl Summarize for Point {}

impl Point
{
    fn dot(&self, other: &Self) -> Real
    {
        self.x.mul_add(other.x, self.y * other.y)
    }
}
impl From<Cplx> for Point
{
    fn from(value: Cplx) -> Self
    {
        Self {
            x: value.re,
            y: value.im,
        }
    }
}
impl From<Point> for Cplx
{
    fn from(value: Point) -> Self
    {
        Self::new(value.x, value.y)
    }
}

#[derive(Default, Debug, Clone, Copy, Add, Sub, Display, From, PartialEq)]
#[display(fmt = "[{v0}, {v1}]")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Matrix2x2
{
    pub v0: Point,
    pub v1: Point,
}
impl Matrix2x2
{
    #[must_use]
    pub const fn new(v00: Real, v01: Real, v10: Real, v11: Real) -> Self
    {
        let v0 = Point { x: v00, y: v01 };
        let v1 = Point { x: v10, y: v11 };
        Self { v0, v1 }
    }
    #[must_use]
    pub const fn diag(v00: Real, v11: Real) -> Self
    {
        let v0 = Point { x: v00, y: 0. };
        let v1 = Point { x: 0., y: v11 };
        Self { v0, v1 }
    }
    #[must_use]
    pub const fn identity() -> Self
    {
        Self::diag(1., 1.)
    }
    fn det(&self) -> Real
    {
        self.v0.x.mul_add(self.v1.y, -self.v0.y * self.v1.x)
    }
    fn trace(&self) -> Real
    {
        self.v0.x + self.v1.y
    }
}
impl From<Matrix2x2> for Cplx
{
    fn from(value: Matrix2x2) -> Self
    {
        Self::new(value.v0.x * value.v1.y, value.v0.y * value.v1.x)
    }
}
impl From<Real> for Matrix2x2
{
    fn from(value: Real) -> Self
    {
        Self::new(value, 0., 0., value)
    }
}
impl std::ops::MulAssign for Matrix2x2
{
    fn mul_assign(&mut self, rhs: Self)
    {
        self.v0.x = self.v0.x.mul_add(rhs.v0.x, self.v1.x * rhs.v0.y);
        self.v0.y = self.v0.y.mul_add(rhs.v0.x, self.v1.y * rhs.v0.y);
        self.v1.x = self.v0.x.mul_add(rhs.v1.x, self.v1.x * rhs.v1.y);
        self.v1.y = self.v0.y.mul_add(rhs.v1.x, self.v1.y * rhs.v1.y);
    }
}
impl Norm<Real> for Matrix2x2
{
    fn norm_sqr(&self) -> Real
    {
        let u = self.det();
        u * u
    }
    fn norm(&self) -> Real
    {
        self.det().abs()
    }
    fn arg(&self) -> Real
    {
        self.v0.arg()
    }
    fn is_nan(&self) -> bool
    {
        self.v0.is_nan() || self.v1.is_nan()
    }
}

pub trait Dist<R>
{
    fn dist(&self, other: Self) -> R;
    fn dist_sqr(&self, other: Self) -> R;
}

impl<R, T> Dist<R> for T
where
    T: Norm<R> + std::ops::Sub<Output = T>,
{
    fn dist(&self, other: Self) -> R
    {
        (*self - other).norm()
    }
    fn dist_sqr(&self, other: Self) -> R
    {
        (*self - other).norm_sqr()
    }
}

#[derive(Copy, Clone, Debug, Display)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PlaneID
{
    #[display(fmt = "w-plane")]
    ZPlane,
    #[display(fmt = "z-plane")]
    WPlane,
}
impl PlaneID
{
    #[must_use]
    pub const fn swap(&self) -> Self
    {
        match self
        {
            Self::ZPlane => Self::WPlane,
            Self::WPlane => Self::ZPlane,
        }
    }
}

#[derive(Copy, Clone, Debug, Display)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Bicomplex
{
    #[display(fmt = "PlaneA({_0})")]
    PlaneA(Cplx),
    #[display(fmt = "PlaneB({_0})")]
    PlaneB(Cplx),
}

impl From<Cplx> for Bicomplex
{
    fn from(value: Cplx) -> Self
    {
        Self::PlaneA(value)
        // Self::PlaneB(value)
    }
}
impl From<Bicomplex> for Cplx
{
    fn from(value: Bicomplex) -> Self
    {
        use Bicomplex::{PlaneA, PlaneB};
        match value
        {
            PlaneA(z) | PlaneB(z) => z,
        }
    }
}
impl Norm<Real> for Bicomplex
{
    fn norm(&self) -> Real
    {
        match self
        {
            Self::PlaneA(z) | Self::PlaneB(z) => z.norm(),
        }
    }
    fn norm_sqr(&self) -> Real
    {
        match self
        {
            Self::PlaneA(z) | Self::PlaneB(z) => z.norm_sqr(),
        }
    }
    fn arg(&self) -> Real
    {
        match self
        {
            Self::PlaneA(z) | Self::PlaneB(z) => z.arg(),
        }
    }
    fn is_nan(&self) -> bool
    {
        match self
        {
            Self::PlaneA(z) | Self::PlaneB(z) => z.is_nan(),
        }
    }
}

impl Default for Bicomplex
{
    fn default() -> Self
    {
        Self::PlaneA(ZERO)
    }
}

impl Dist<Real> for Bicomplex
{
    fn dist(&self, rhs: Self) -> Real
    {
        match self
        {
            Self::PlaneA(z) => match rhs
            {
                Self::PlaneA(w) => (z - w).norm(),
                Self::PlaneB(_) => Real::INFINITY,
            },
            Self::PlaneB(z) => match rhs
            {
                Self::PlaneA(_) => Real::INFINITY,
                Self::PlaneB(w) => (z - w).norm(),
            },
        }
    }
    fn dist_sqr(&self, rhs: Self) -> Real
    {
        match self
        {
            Self::PlaneA(z) => match rhs
            {
                Self::PlaneA(w) => (z - w).norm_sqr(),
                Self::PlaneB(_) => Real::INFINITY,
            },
            Self::PlaneB(z) => match rhs
            {
                Self::PlaneA(_) => Real::INFINITY,
                Self::PlaneB(w) => (z - w).norm_sqr(),
            },
        }
    }
}
