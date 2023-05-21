use super::{ComplexNum, RealNum};
use derive_more::{Add, Display, From, Sub};

pub trait Norm<R>: Copy
{
    fn norm(&self) -> R;
    fn norm_sqr(&self) -> R;
    fn arg(&self) -> R;
    fn is_nan(&self) -> bool;
}

impl Norm<RealNum> for ComplexNum
{
    #[inline]
    fn norm(&self) -> RealNum
    {
        <Self>::norm(*self)
    }
    #[inline]
    fn norm_sqr(&self) -> RealNum
    {
        <Self>::norm_sqr(self)
    }
    #[inline]
    fn arg(&self) -> RealNum
    {
        <Self>::arg(*self)
    }
    #[inline]
    fn is_nan(&self) -> bool
    {
        <Self>::is_nan(*self)
    }
}

impl Norm<RealNum> for Point
{
    fn norm(&self) -> RealNum
    {
        self.norm_sqr().sqrt()
    }

    fn norm_sqr(&self) -> RealNum
    {
        self.x.mul_add(self.x, self.y * self.y)
    }

    fn arg(&self) -> RealNum
    {
        self.y.atan2(self.x)
    }

    fn is_nan(&self) -> bool
    {
        self.x.is_nan() || self.y.is_nan()
    }
}
#[derive(Default, Clone, Copy, Debug, Add, Sub, Display, From, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[display(fmt = "({}, {})", x, y)]
pub struct Point
{
    pub x: RealNum,
    pub y: RealNum,
}
impl Point
{
    fn dot(&self, other: &Self) -> RealNum
    {
        self.x.mul_add(other.x, self.y * other.y)
    }
}
impl From<ComplexNum> for Point
{
    fn from(value: ComplexNum) -> Self
    {
        Self {
            x: value.re,
            y: value.im,
        }
    }
}
impl From<Point> for ComplexNum
{
    fn from(value: Point) -> Self
    {
        Self::new(value.x, value.y)
    }
}

#[derive(Default, Debug, Clone, Copy, Add, Sub, Display, From, PartialEq)]
#[display(fmt = "[{}, {}]", v0, v1)]
pub struct Matrix2x2
{
    pub v0: Point,
    pub v1: Point,
}
impl Matrix2x2
{
    #[must_use]
    pub const fn new(v00: RealNum, v01: RealNum, v10: RealNum, v11: RealNum) -> Self
    {
        let v0 = Point { x: v00, y: v01 };
        let v1 = Point { x: v10, y: v11 };
        Self { v0, v1 }
    }
    #[must_use]
    pub const fn diag(v00: RealNum, v11: RealNum) -> Self
    {
        let v0 = Point { x: v00, y: 0. };
        let v1 = Point { x: 0., y: v11 };
        Self { v0, v1 }
    }
    fn det(&self) -> RealNum
    {
        self.v0.x.mul_add(self.v1.y, -self.v0.y * self.v1.x)
    }
    fn trace(&self) -> RealNum
    {
        self.v0.x + self.v1.y
    }
}
impl From<Matrix2x2> for ComplexNum
{
    fn from(value: Matrix2x2) -> Self
    {
        Self::new(value.v0.x * value.v1.y, value.v0.y * value.v1.x)
    }
}
impl From<RealNum> for Matrix2x2
{
    fn from(value: RealNum) -> Self
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
impl Norm<RealNum> for Matrix2x2
{
    fn norm_sqr(&self) -> RealNum
    {
        let u = self.det();
        u * u
    }
    fn norm(&self) -> RealNum
    {
        self.det().abs()
    }
    fn arg(&self) -> RealNum
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