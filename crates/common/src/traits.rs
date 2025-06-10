use crate::globals::DISPLAY_PREC;
use crate::prelude::{RationalAngle, Real, TAUI};
use crate::types::Cplx;

pub trait Norm<R>: Copy
{
    fn norm(&self) -> R;
    fn norm_sqr(&self) -> R;
}
pub trait Arg<R>
{
    fn arg(self) -> R;
}
pub trait Conj: Clone
{
    #[must_use]
    fn conj(&self) -> Self
    {
        self.clone()
    }
}

pub trait Polar<R>: Norm<R> + Arg<R> {}
impl<T, R> Polar<R> for T where T: Norm<R> + Arg<R> {}

pub trait MaybeNan
{
    fn is_nan(&self) -> bool;
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

pub trait Named
{
    fn name(&self) -> &'static str
    {
        "c"
    }
}

#[derive(Clone, Debug)]
pub struct DescriptionConf
{
    pub is_enabled: bool,
    pub precision: usize,
}
impl DescriptionConf
{
    #[must_use]
    pub const fn new() -> Self
    {
        Self {
            is_enabled: false,
            precision: DISPLAY_PREC,
        }
    }
    #[must_use]
    pub const fn enabled(self) -> Self
    {
        self.with_visibility(true)
    }
    #[must_use]
    pub const fn with_visibility(mut self, visible: bool) -> Self
    {
        self.is_enabled = visible;
        self
    }
    #[must_use]
    pub const fn with_precision(mut self, prec: usize) -> Self
    {
        self.precision = prec;
        self
    }
}
impl Default for DescriptionConf
{
    fn default() -> Self
    {
        Self::new()
    }
}

/// Used to print the value of a variable in the GUI's format
pub trait Describe: std::fmt::Display
{
    fn describe(&self, desc_conf: &DescriptionConf) -> Option<String>
    {
        desc_conf.is_enabled.then(|| self.to_string())
    }
}

/// Used to print the name and value of a variable in the GUI's format
pub trait Summarize: std::fmt::Display
{
    fn summarize(&self) -> Option<String>
    {
        let s = self.to_string();
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    }
}

impl<T> Summarize for T
where
    T: Named + Describe,
{
    fn summarize(&self) -> Option<String>
    {
        Some(format!(
            "{} = {}",
            self.name(),
            self.describe(&DescriptionConf::new())?
        ))
    }
}

pub trait FloatLike: std::fmt::Display {}
impl FloatLike for f32 {}
impl FloatLike for f64 {}
impl FloatLike for Cplx {}

impl Named for f32 {}
impl Named for f64 {}
impl Named for Cplx {}

impl<T> Describe for T
where
    T: FloatLike,
{
    fn describe(&self, params: &DescriptionConf) -> Option<String>
    {
        params
            .is_enabled
            .then(|| format!("{self:.prec$}", prec = params.precision))
    }
}

impl Named for i32
{
    fn name(&self) -> &'static str
    {
        "n"
    }
}

impl Describe for i32
{
    fn describe(&self, params: &DescriptionConf) -> Option<String>
    {
        params.is_enabled.then(|| format!("{self}"))
    }
}

pub trait ToCircle
{
    fn to_circle(self) -> Cplx;
}
impl ToCircle for Real
{
    fn to_circle(self) -> Cplx
    {
        (TAUI * self).exp()
    }
}
impl ToCircle for RationalAngle
{
    fn to_circle(self) -> Cplx
    {
        (TAUI * Real::from(self)).exp()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FloatToIntError;

impl std::fmt::Display for FloatToIntError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "Error converting float to integer")
    }
}

impl std::error::Error for FloatToIntError {}

pub trait TryRound<T>
{
    fn try_round(self) -> Result<T, FloatToIntError>;
}

macro_rules! try_round_impl {
    ($float: ty, $int: ty) => {
        impl TryRound<$int> for $float
        {
            #[allow(clippy::cast_lossless)]
            fn try_round(self) -> Result<$int, FloatToIntError>
            {
                if self.is_finite() && self > <$int>::MIN as $float && self < <$int>::MAX as $float
                {
                    Ok(self.round() as $int)
                } else {
                    Err(FloatToIntError)
                }
            }
        }
    };
}

try_round_impl!(f64, i64);
try_round_impl!(f64, i32);
try_round_impl!(f32, i64);
try_round_impl!(f32, i32);

use num_traits::{One, Zero};
use std::fmt::Display;
use std::ops::{Add, AddAssign, Mul, MulAssign, Sub};

pub trait Variable:
    Norm<Real>
    + Sub<Output = Self>
    + MaybeNan
    + Clone
    + Send
    + Default
    + From<Cplx>
    + Into<Cplx>
    + Describe
{
}
pub trait Parameter: Clone + Send + Sync + Default + PartialEq + Describe + Summarize {}
pub trait Derivative:
    Polar<Real>
    + Send
    + Sync
    + Default
    + Zero
    + One
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + AddAssign
    + MulAssign
    + Display
    + Into<Cplx>
{
}

impl<V> Variable for V where
    V: Norm<Real>
        + Sub<Output = Self>
        + MaybeNan
        + Clone
        + Send
        + Default
        + From<Cplx>
        + Into<Cplx>
        + Describe
{
}
impl<P> Parameter for P where P: Clone + Send + Sync + Default + PartialEq + Describe + Summarize {}
impl<D> Derivative for D where
    D: Polar<Real>
        + Send
        + Sync
        + Default
        + Zero
        + One
        + Add<Output = Self>
        + Sub<Output = Self>
        + Mul<Output = Self>
        + AddAssign
        + MulAssign
        + Display
        + Into<Cplx>
{
}

macro_rules! impl_polar {
    ($t: ty) => {
        impl Norm<Real> for $t
        {
            #[inline]
            fn norm(&self) -> Real
            {
                Real::from(*self)
            }
            #[inline]
            fn norm_sqr(&self) -> Real
            {
                Real::from(self * self)
            }
        }
        impl Arg<Real> for $t
        {
            #[inline]
            fn arg(self) -> Real
            {
                if self >= 0.0 {
                    0.0
                } else {
                    std::f64::consts::PI
                }
            }
        }
        impl Conj for $t
        {
            #[inline]
            fn conj(&self) -> Self
            {
                *self
            }
        }
    };
}

impl_polar!(f64);
impl_polar!(f32);
