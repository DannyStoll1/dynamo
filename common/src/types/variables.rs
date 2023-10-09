use super::{Cplx, Real};
use crate::consts::ZERO;
use crate::traits::{Arg, Describe, MaybeNan, Norm, Summarize};
use derive_more::{Add, Display, From};

pub mod matrix;
pub use matrix::{Matrix2x2, Point};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Copy, Debug, Add, From, PartialEq, Eq, Display)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[display(fmt = "[ a: {a}, b: {b} ] ")]
pub struct Pair<T>
where
    T: std::fmt::Display,
{
    pub a: T,
    pub b: T,
}

impl<T> Describe for Pair<T>
where
    T: std::fmt::Display,
{
    fn describe(&self) -> Option<String>
    {
        Some(self.to_string())
    }
}
impl<T> Summarize for Pair<T> where T: std::fmt::Display {}

impl<T> From<Cplx> for Pair<T>
where
    T: std::fmt::Display,
{
    fn from(_z: Cplx) -> Self
    {
        unimplemented!()
    }
}

impl<T> From<Pair<T>> for Cplx
where
    T: std::fmt::Display,
{
    fn from(_value: Pair<T>) -> Self
    {
        unimplemented!()
    }
}

pub type RealPair = Pair<Real>;
pub type CplxPair = Pair<Cplx>;

#[derive(Default, Clone, Copy, Debug, Add, From, PartialEq, Display)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[display(fmt = "[ a: {a}, b: {b}, c: {c}, d: {d} ] ")]
pub struct ComplexQuad
{
    pub a: Cplx,
    pub b: Cplx,
    pub c: Cplx,
    pub d: Cplx,
}

impl Summarize for ComplexQuad {}

impl From<Cplx> for ComplexQuad
{
    fn from(_z: Cplx) -> Self
    {
        unimplemented!()
    }
}

impl From<ComplexQuad> for Cplx
{
    fn from(_value: ComplexQuad) -> Self
    {
        unimplemented!()
    }
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
}
impl Arg<Real> for Cplx
{
    #[inline]
    fn arg(&self) -> Real
    {
        <Self>::arg(*self)
    }
}
impl MaybeNan for Cplx
{
    #[inline]
    fn is_nan(&self) -> bool
    {
        <Self>::is_nan(*self)
    }
}

#[derive(Copy, Clone, Debug, Display, PartialEq, Eq)]
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

#[derive(Copy, Clone, Debug, Display, PartialEq)]
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
}
impl Arg<Real> for Bicomplex
{
    fn arg(&self) -> Real
    {
        match self
        {
            Self::PlaneA(z) | Self::PlaneB(z) => z.arg(),
        }
    }
}
impl MaybeNan for Bicomplex
{
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

// impl Dist<Real> for Bicomplex
// {
//     fn dist(&self, rhs: Self) -> Real
//     {
//         match self
//         {
//             Self::PlaneA(z) => match rhs
//             {
//                 Self::PlaneA(w) => (z - w).norm(),
//                 Self::PlaneB(_) => Real::INFINITY,
//             },
//             Self::PlaneB(z) => match rhs
//             {
//                 Self::PlaneA(_) => Real::INFINITY,
//                 Self::PlaneB(w) => (z - w).norm(),
//             },
//         }
//     }
//     fn dist_sqr(&self, rhs: Self) -> Real
//     {
//         match self
//         {
//             Self::PlaneA(z) => match rhs
//             {
//                 Self::PlaneA(w) => (z - w).norm_sqr(),
//                 Self::PlaneB(_) => Real::INFINITY,
//             },
//             Self::PlaneB(z) => match rhs
//             {
//                 Self::PlaneA(_) => Real::INFINITY,
//                 Self::PlaneB(w) => (z - w).norm_sqr(),
//             },
//         }
//     }
// }

impl std::ops::Sub for Bicomplex
{
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output
    {
        match self
        {
            Self::PlaneA(z) => match rhs
            {
                Self::PlaneA(w) => Self::PlaneA(z - w),
                Self::PlaneB(_) => Self::PlaneA(crate::consts::NAN),
            },
            Self::PlaneB(z) => match rhs
            {
                Self::PlaneA(_) => Self::PlaneB(crate::consts::NAN),
                Self::PlaneB(w) => Self::PlaneB(z - w),
            },
        }
    }
}
