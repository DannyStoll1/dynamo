use num_complex::Complex;
use num_traits::Num;

/// A positive definite map.
pub trait TopologicalNorm
{
    type Norm: Clone + Num + PartialOrd;
    fn topological_norm(&self) -> Self::Norm;
}

/// A symmetric, positive definite function
pub trait Semimetric
{
    type Dist: Clone + Num + PartialOrd;
    fn dist_semi(&self, other: Self) -> Self::Dist;
}

impl<T> Semimetric for T
where
    T: Copy + TopologicalNorm + std::ops::Sub<Output = T>,
{
    type Dist = <Self as TopologicalNorm>::Norm;
    fn dist_semi(&self, other: Self) -> Self::Dist
    {
        (*self - other).topological_norm()
    }
}

impl<T: Clone + Num + PartialOrd> TopologicalNorm for Complex<T>
{
    type Norm = T;
    #[inline]
    fn topological_norm(&self) -> T
    {
        Self::norm_sqr(self)
    }
}

macro_rules! top_norm_abs_impl {
    ($t:ty) => {
        impl TopologicalNorm for $t
        {
            type Norm = Self;
            fn topological_norm(&self) -> Self::Norm
            {
                self.abs()
            }
        }
    };
}

top_norm_abs_impl!(f32);
top_norm_abs_impl!(f64);
top_norm_abs_impl!(i32);
top_norm_abs_impl!(i64);
