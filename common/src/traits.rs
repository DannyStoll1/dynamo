use crate::globals::DISPLAY_PREC;
use crate::types::Cplx;

pub trait Norm<R>: Copy
{
    fn norm(&self) -> R;
    fn norm_sqr(&self) -> R;
}
pub trait Arg<R>: Copy
{
    fn arg(&self) -> R;
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
    fn name(&self) -> &str
    {
        "c"
    }
}

/// Used to print the value of a variable in the GUI's format
pub trait Describe: std::fmt::Display
{
    fn describe(&self) -> Option<String>
    {
        Some(self.to_string())
    }
}

/// Used to print the name and value of a variable in the GUI's format
pub trait Summarize: std::fmt::Display
{
    fn summarize(&self) -> Option<String>
    {
        Some(self.to_string())
    }
}

impl<T> Summarize for T
where
    T: Named + Describe,
{
    fn summarize(&self) -> Option<String>
    {
        Some(format!("{} = {}", self.name(), self.describe()?))
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
    fn describe(&self) -> Option<String>
    {
        Some(format!("{self:.DISPLAY_PREC$}"))
    }
}

impl Named for i32
{
    fn name(&self) -> &str
    {
        "n"
    }
}

impl Describe for i32
{
    fn describe(&self) -> Option<String>
    {
        Some(format!("{self}"))
    }
}
