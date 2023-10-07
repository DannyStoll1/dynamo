use crate::{
    globals::{NEWTON_MAX_ERR, NEWTON_MAX_ITERS, NEWTON_MIN_ERR},
    traits::{Dist, MaybeNan, Norm},
    types::Real,
};
use std::ops::{AddAssign, Div, Sub, SubAssign};

pub fn newton_fixed_iter<T, F, G>(f_and_df: F, start: T, target: T, iters: usize) -> T
where
    F: Fn(T) -> (T, T),
    T: Sub<Output = T> + Div<Output = T> + AddAssign + Copy,
{
    let mut z = start;
    for _ in 0..iters
    {
        let (f, df) = f_and_df(z);
        z += (target - f) / df;
    }
    z
}

/// Apply Newton's method until we converge to within `tolerance`.
/// Will loop forever if Newton's method fails to converge.
pub fn newton_until_convergence<T, F>(f_and_df: F, start: T, target: T, tolerance: Real) -> T
where
    F: Fn(T) -> (T, T),
    T: Sub<Output = T> + Div<Output = T> + AddAssign + Norm<Real> + Copy,
{
    let mut z = start;
    let mut z_old = start;

    let mut error = Real::INFINITY;

    while error > tolerance
    {
        let (f, df) = f_and_df(z);
        z += (target - f) / df;
        error = z.dist_sqr(z_old);
        z_old = z;
    }
    z
}

/// Apply Newton's method until we converge to within `tolerance`.
/// Will loop forever if Newton's method fails to converge.
/// Returns root together with value and derivative of function.
pub fn newton_until_convergence_d<T, F>(
    f_and_df: F,
    start: T,
    target: T,
    tolerance: Real,
) -> (T, T, T)
where
    F: Fn(T) -> (T, T),
    T: Sub<Output = T> + Div<Output = T> + AddAssign + Norm<Real> + Copy + Default,
{
    let mut z = start;
    let mut z_old = start;

    let mut error = Real::INFINITY;

    let f: T = T::default();
    let df: T = T::default();

    while error > tolerance
    {
        let (f, df) = f_and_df(z);
        z += (target - f) / df;
        error = z.dist_sqr(z_old);
        z_old = z;
    }
    (z, f, df)
}

/// Apply Newton's method until we converge to within `tolerance`.
/// Will loop forever if Newton's method fails to converge.
/// Returns the approximate root, together with the value and derivative of the function there.
pub fn find_root_newton_d<T, F>(mut f_and_df: F, start: T) -> Option<(T, T, T)>
where
    F: FnMut(T) -> (T, T),
    T: Div<Output = T> + SubAssign + Dist<Real> + MaybeNan + Copy,
{
    let mut z = start;
    let mut z_old = start;
    let mut f = start;
    let mut df = start;

    for _ in 0..NEWTON_MAX_ITERS
    {
        z_old = z;
        (f, df) = f_and_df(z);
        z -= f / df;

        // Terminate early if we are below min error threshold
        if z.dist_sqr(z_old) < NEWTON_MIN_ERR
        {
            return Some((z, f, df));
        }
        else if z.is_nan()
        {
            return None;
        }
    }
    if z.dist_sqr(z_old) < NEWTON_MAX_ERR
    {
        Some((z, f, df))
    }
    else
    {
        None
    }
}

/// Apply Newton's method until we converge to within `tolerance`.
/// Will loop forever if Newton's method fails to converge.
pub fn find_root_newton<T, F>(f_and_df: F, start: T) -> Option<T>
where
    F: FnMut(T) -> (T, T),
    T: Div<Output = T> + SubAssign + Dist<Real> + MaybeNan + Copy,
{
    find_root_newton_d(f_and_df, start).map(|(z, _f, _d)| z)
}
