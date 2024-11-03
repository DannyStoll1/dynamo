use crate::prelude::Conj;
use crate::traits::{Arg, FloatLike, MaybeNan, Named, Norm};
use crate::types::{Cplx, Real};
use derive_more::{Add, AddAssign, Display, From, Sub};
use num_traits::{One, Zero};

#[derive(Default, Clone, Copy, Debug, Add, Sub, AddAssign, Display, From, PartialEq)]
#[display("({x}, {y})")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Point
{
    pub x: Real,
    pub y: Real,
}
impl Named for Point
{
    fn name(&self) -> &'static str
    {
        "p"
    }
}
impl FloatLike for Point {}

impl Point
{
    fn dot(&self, other: &Self) -> Real
    {
        self.x.mul_add(other.x, self.y * other.y)
    }
}
impl Zero for Point
{
    fn zero() -> Self
    {
        Self { x: 0., y: 0. }
    }
    fn is_zero(&self) -> bool
    {
        self.x.is_zero() && self.y.is_zero()
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
}

impl Arg<Real> for Point
{
    fn arg(self) -> Real
    {
        self.y.atan2(self.x)
    }
}
impl MaybeNan for Point
{
    fn is_nan(&self) -> bool
    {
        self.x.is_nan() || self.y.is_nan()
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

#[derive(Default, Debug, Clone, Copy, Add, Sub, AddAssign, Display, From, PartialEq)]
#[display("[{v0}, {v1}]")]
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
    const fn trace(&self) -> Real
    {
        self.v0.x + self.v1.y
    }
    const fn transpose(mut self) -> Self
    {
        let tmp = self.v0.y;
        self.v0.y = self.v1.x;
        self.v1.x = tmp;
        self
    }
}
impl From<Matrix2x2> for Cplx
{
    fn from(value: Matrix2x2) -> Self
    {
        Self::new(value.v0.x * value.v1.y, value.v0.y * value.v1.x)
    }
}
impl Zero for Matrix2x2
{
    fn zero() -> Self
    {
        Self::new(0., 0., 0., 0.)
    }
    fn is_zero(&self) -> bool
    {
        self.v0.is_zero() && self.v1.is_zero()
    }
}
impl One for Matrix2x2
{
    fn one() -> Self
    {
        Self::new(1., 0., 0., 1.)
    }
}
impl std::ops::Mul for Matrix2x2
{
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output
    {
        let v0 = Point {
            x: self.v0.x.mul_add(rhs.v0.x, self.v1.x * rhs.v0.y),
            y: self.v0.y.mul_add(rhs.v0.x, self.v1.y * rhs.v0.y),
        };
        let v1 = Point {
            x: self.v0.x.mul_add(rhs.v1.x, self.v1.x * rhs.v1.y),
            y: self.v0.y.mul_add(rhs.v1.x, self.v1.y * rhs.v1.y),
        };
        Self { v0, v1 }
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
}
impl Arg<Real> for Matrix2x2
{
    fn arg(self) -> Real
    {
        self.v0.arg()
    }
}
impl Conj for Matrix2x2
{
    fn conj(&self) -> Self
    {
        self.transpose()
    }
}
impl MaybeNan for Matrix2x2
{
    fn is_nan(&self) -> bool
    {
        self.v0.is_nan() || self.v1.is_nan()
    }
}
