use super::{Cplx, Real};
use crate::consts::ZERO;
use crate::prelude::{Conj, OMEGA};
use crate::traits::{Arg, Describe, DescriptionConf, MaybeNan, Norm, Summarize};
use derive_more::{Add, AddAssign, Display, From, Sub, SubAssign};

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
    fn describe(&self, desc_params: &DescriptionConf) -> Option<String>
    {
        desc_params.is_enabled.then(|| self.to_string())
    }
}
impl<T> Summarize for Pair<T> where T: std::fmt::Display {}

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
    fn arg(self) -> Real
    {
        <Self>::arg(self)
    }
}
impl Conj for Cplx
{
    #[inline]
    fn conj(&self) -> Self
    {
        Self::conj(self)
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
        match self {
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
        match value {
            PlaneA(z) | PlaneB(z) => z,
        }
    }
}
impl Norm<Real> for Bicomplex
{
    fn norm(&self) -> Real
    {
        match self {
            Self::PlaneA(z) | Self::PlaneB(z) => z.norm(),
        }
    }
    fn norm_sqr(&self) -> Real
    {
        match self {
            Self::PlaneA(z) | Self::PlaneB(z) => z.norm_sqr(),
        }
    }
}
impl Arg<Real> for Bicomplex
{
    fn arg(self) -> Real
    {
        match self {
            Self::PlaneA(z) | Self::PlaneB(z) => z.arg(),
        }
    }
}
impl MaybeNan for Bicomplex
{
    fn is_nan(&self) -> bool
    {
        match self {
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

impl std::ops::Mul<Cplx> for Bicomplex
{
    type Output = Self;
    fn mul(self, z: Cplx) -> Self
    {
        match self {
            Self::PlaneA(w) => Self::PlaneA(w * z),
            Self::PlaneB(w) => Self::PlaneB(w * z),
        }
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
        match self {
            Self::PlaneA(z) => match rhs {
                Self::PlaneA(w) => Self::PlaneA(z - w),
                Self::PlaneB(_) => Self::PlaneA(crate::consts::NAN),
            },
            Self::PlaneB(z) => match rhs {
                Self::PlaneA(_) => Self::PlaneB(crate::consts::NAN),
                Self::PlaneB(w) => Self::PlaneB(z - w),
            },
        }
    }
}

impl Describe for Bicomplex
{
    fn describe(&self, desc_conf: &DescriptionConf) -> Option<String>
    {
        desc_conf.is_enabled.then(|| self.to_string())
    }
}

#[derive(Clone, Copy, Default, Debug, Hash, PartialEq, Eq, Add, Sub, AddAssign, SubAssign)]
pub struct EisensteinInteger
{
    pub a: i64,
    pub b: i64,
}
impl EisensteinInteger
{
    #[must_use]
    pub const fn new(a: i64, b: i64) -> Self
    {
        Self { a, b }
    }
}

impl std::ops::Mul for EisensteinInteger
{
    type Output = Self;
    fn mul(self, Self { a: c, b: d }: Self) -> Self::Output
    {
        let a = self.a;
        let b = self.b;
        Self::new(a * c - b * d, (a - b) * d + b * c)
    }
}

impl std::ops::MulAssign for EisensteinInteger
{
    fn mul_assign(&mut self, rhs: Self)
    {
        *self = *self * rhs;
    }
}

impl std::ops::Mul<EisensteinInteger> for i64
{
    type Output = EisensteinInteger;
    fn mul(self, rhs: EisensteinInteger) -> Self::Output
    {
        EisensteinInteger::new(self * rhs.a, self * rhs.b)
    }
}

impl std::ops::Mul<i64> for EisensteinInteger
{
    type Output = Self;
    fn mul(self, rhs: i64) -> Self::Output
    {
        Self::new(rhs * self.a, rhs * self.b)
    }
}

impl std::ops::MulAssign<i64> for EisensteinInteger
{
    fn mul_assign(&mut self, rhs: i64)
    {
        self.a *= rhs;
        self.b *= rhs;
    }
}

impl std::ops::Div for EisensteinInteger
{
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output
    {
        use num_traits::Zero;
        assert!(
            !rhs.is_zero(),
            "Attempt to divide Eisenstein integer by zero"
        );

        let quot_approx = Cplx::from(self) / Cplx::from(rhs);
        quot_approx.into()
    }
}

impl std::ops::Rem for EisensteinInteger
{
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output
    {
        let quot = self / rhs;
        self - quot * rhs
    }
}

impl num_traits::Zero for EisensteinInteger
{
    fn zero() -> Self
    {
        Self { a: 0, b: 0 }
    }

    fn is_zero(&self) -> bool
    {
        self.a == 0 && self.b == 0
    }

    fn set_zero(&mut self)
    {
        self.a = 0;
        self.b = 0;
    }
}

impl num_traits::One for EisensteinInteger
{
    fn one() -> Self
    {
        Self { a: 1, b: 0 }
    }

    fn is_one(&self) -> bool
    {
        self.a == 1 && self.b == 0
    }

    fn set_one(&mut self)
    {
        self.a = 1;
        self.b = 0;
    }
}

impl From<Cplx> for EisensteinInteger
{
    fn from(z: Cplx) -> Self
    {
        let y = (z.im / OMEGA.im).floor();
        let x = y.mul_add(0.5, z.re).floor();

        // Nearest corner of the fundamental parallelogram
        let mut corner = x + OMEGA * y;
        let mut best_dist = (corner - z).norm_sqr();
        let mut closest = (0, 0);

        corner += 1.;
        let dist = (corner - z).norm_sqr();
        if dist < best_dist {
            best_dist = dist;
            closest = (1, 0);
        }

        corner += OMEGA;
        let dist = (corner - z).norm_sqr();
        if dist < best_dist {
            best_dist = dist;
            closest = (1, 1);
        }

        corner -= 1.;
        let dist = (corner - z).norm_sqr();
        if dist < best_dist {
            closest = (0, 1);
        }

        Self {
            a: x as i64 + closest.0,
            b: y as i64 + closest.1,
        }
    }
}

impl From<i64> for EisensteinInteger
{
    fn from(a: i64) -> Self
    {
        Self { a, b: 0 }
    }
}

impl From<EisensteinInteger> for Cplx
{
    fn from(eis: EisensteinInteger) -> Self
    {
        (eis.a as Real) + (eis.b as Real) * OMEGA
    }
}

impl MaybeNan for EisensteinInteger
{
    #[inline]
    fn is_nan(&self) -> bool
    {
        false
    }
}

impl Norm<Real> for EisensteinInteger
{
    #[inline]
    fn norm_sqr(&self) -> Real
    {
        (self.a.pow(2) + self.b.pow(2) - self.a * self.b) as Real
    }

    #[inline]
    fn norm(&self) -> Real
    {
        self.norm_sqr().sqrt()
    }
}

impl Arg<Real> for EisensteinInteger
{
    fn arg(self) -> Real
    {
        Cplx::from(self).arg()
    }
}

impl std::fmt::Display for EisensteinInteger
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "{} + {}ω", self.a, self.b)
    }
}

impl Describe for EisensteinInteger {}
impl crate::traits::Named for EisensteinInteger
{
    fn name(&self) -> &str
    {
        "c"
    }
}

#[derive(Clone, Copy, Default, Debug, Hash, PartialEq, Eq, Add, Sub, AddAssign, SubAssign)]
pub struct GaussianInteger
{
    pub a: i64,
    pub b: i64,
}
impl GaussianInteger
{
    #[must_use]
    pub const fn new(a: i64, b: i64) -> Self
    {
        Self { a, b }
    }
}

impl std::ops::Mul for GaussianInteger
{
    type Output = Self;
    fn mul(self, Self { a: c, b: d }: Self) -> Self::Output
    {
        let a = self.a;
        let b = self.b;
        Self::new(a * c - b * d, a * d + b * c)
    }
}

impl std::ops::MulAssign for GaussianInteger
{
    fn mul_assign(&mut self, rhs: Self)
    {
        *self = *self * rhs;
    }
}

impl std::ops::Mul<GaussianInteger> for i64
{
    type Output = GaussianInteger;
    fn mul(self, rhs: GaussianInteger) -> Self::Output
    {
        GaussianInteger::new(self * rhs.a, self * rhs.b)
    }
}

impl std::ops::Mul<i64> for GaussianInteger
{
    type Output = Self;
    fn mul(self, rhs: i64) -> Self::Output
    {
        Self::new(rhs * self.a, rhs * self.b)
    }
}

impl std::ops::MulAssign<i64> for GaussianInteger
{
    fn mul_assign(&mut self, rhs: i64)
    {
        self.a *= rhs;
        self.b *= rhs;
    }
}

impl std::ops::Div for GaussianInteger
{
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output
    {
        use num_traits::Zero;
        assert!(!rhs.is_zero(), "Attempt to divide Gaussian integer by zero");

        let quot_approx = Cplx::from(self) / Cplx::from(rhs);
        quot_approx.into()
    }
}

impl std::ops::Rem for GaussianInteger
{
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output
    {
        let quot = self / rhs;
        self - quot * rhs
    }
}

impl num_traits::Zero for GaussianInteger
{
    fn zero() -> Self
    {
        Self { a: 0, b: 0 }
    }

    fn is_zero(&self) -> bool
    {
        self.a == 0 && self.b == 0
    }

    fn set_zero(&mut self)
    {
        self.a = 0;
        self.b = 0;
    }
}

impl num_traits::One for GaussianInteger
{
    fn one() -> Self
    {
        Self { a: 1, b: 0 }
    }

    fn is_one(&self) -> bool
    {
        self.a == 1 && self.b == 0
    }

    fn set_one(&mut self)
    {
        self.a = 1;
        self.b = 0;
    }
}

impl From<Cplx> for GaussianInteger
{
    fn from(z: Cplx) -> Self
    {
        let x = z.re.round();
        let y = z.im.round();

        Self {
            a: x as i64,
            b: y as i64,
        }
    }
}

impl From<i64> for GaussianInteger
{
    fn from(a: i64) -> Self
    {
        Self { a, b: 0 }
    }
}

impl From<GaussianInteger> for Cplx
{
    fn from(eis: GaussianInteger) -> Self
    {
        Self::new(eis.a as Real, eis.b as Real)
    }
}

impl MaybeNan for GaussianInteger
{
    #[inline]
    fn is_nan(&self) -> bool
    {
        false
    }
}

impl Norm<Real> for GaussianInteger
{
    #[inline]
    fn norm_sqr(&self) -> Real
    {
        (self.a.pow(2) + self.b.pow(2)) as Real
    }

    #[inline]
    fn norm(&self) -> Real
    {
        self.norm_sqr().sqrt()
    }
}

impl Arg<Real> for GaussianInteger
{
    fn arg(self) -> Real
    {
        Cplx::from(self).arg()
    }
}

impl std::fmt::Display for GaussianInteger
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "{} + {}i", self.a, self.b)
    }
}

impl Describe for GaussianInteger {}
impl crate::traits::Named for GaussianInteger
{
    fn name(&self) -> &str
    {
        "c"
    }
}

impl Conj for GaussianInteger
{
    fn conj(&self) -> Self
    {
        Self {
            a: self.a,
            b: -self.b,
        }
    }
}
impl Conj for EisensteinInteger
{
    fn conj(&self) -> Self
    {
        Self {
            a: self.a - self.b,
            b: -self.b,
        }
    }
}
