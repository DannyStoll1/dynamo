pub use num::integer::binomial;

use crate::consts::{LOG_PI, TAUI, ZERO};
use crate::types::{Cplx, Norm, Real};
use num_complex::ComplexFloat;
pub use spfunc::{
    gamma::{digamma, gamma, polygamma},
    zeta::zeta,
};
use std::f64::consts::PI;
use std::ops::{AddAssign, Div, Sub};

// pub mod erf;
pub mod poly_solve;

#[must_use]
pub fn weierstrass_p(g2: Cplx, g3: Cplx, z: Cplx, tolerance: Real) -> (Cplx, Cplx)
{
    let num_iters = (z.norm() / tolerance).log2().round() as i32 + 1;
    let shrink_scale = (2.0 as Real).powi(-num_iters);
    let z0 = z * shrink_scale;

    let u = z0 * z0;

    let mut p = 1. / u + g2 * u / 20. + g3 * u * u / 28.;
    let mut dp = -2. / (u * z0) + g2 * z0 / 10. + g3 * u * z0 / 7.;

    let mut p_2: Cplx;
    let mut dp_2: Cplx;
    let mut ddp: Cplx;
    let mut ddp_2: Cplx;
    let mut tmp: Cplx;
    let mut four_dp_2: Cplx;

    for _ in 0..num_iters
    {
        p_2 = p * p;
        dp_2 = p * (4. * p_2 - g2) - g3;
        ddp = 6. * p_2 - g2 / 2.;
        ddp_2 = ddp * ddp;
        tmp = ddp_2 / (4. * dp_2) - p - p;
        // dp = (-4. * dp_2 * dp_2 + 12 * p * dp_2 * ddp - ddp * ddp_2) / (4. * dp_2 * dp);
        four_dp_2 = dp_2 + dp_2 + dp_2 + dp_2;
        dp = (four_dp_2 * (3. * p * ddp - dp_2) - ddp * ddp_2) / (four_dp_2 * dp);
        p = tmp;
    }
    (p, dp)
}

#[must_use]
pub fn slog(x: Real) -> Real
{
    if x.is_infinite()
    {
        1000.
    }
    else if x <= 0.
    {
        slog(x.exp()) - 1.
    }
    else if x > 1.
    {
        1. + slog(x.ln())
    }
    else
    {
        x - 1.
    }
}

const fn bernoulli(n: u64) -> f64
{
    match n
    {
        0 => 1.,
        1 => -0.5,
        2 => 0.166_666_666_666_667,
        4 | 8 => -0.033_333_333_333_333_3,
        6 => 0.023_809_523_809_523_8,
        10 => 0.075_757_575_757_575_8,
        12 => -0.253_113_553_113_553,
        14 => 1.166_666_666_666_67,
        16 => -7.092_156_862_745_10,
        18 => 54.971_177_944_862_2,
        20 => -529.124_242_424_242,
        22 => 6_192.123_188_405_80,
        24 => -86_580.253_113_553_1,
        26 => 1.425_517_166_666_67e6,
        28 => -2.729_823_106_781_61e7,
        30 => 6.015_808_739_006_42e8,
        32 => -1.511_631_576_709_22e10,
        34 => 4.296_146_430_611_67e11,
        36 => -1.371_165_520_508_83e13,
        38 => 4.883_323_189_735_93e14,
        40 => -1.929_657_934_194_01e16,
        42 => 8.416_930_475_736_83e17,
        44 => -4.033_807_185_405_95e19,
        46 => 2.115_074_863_808_20e21,
        48 => -1.208_662_652_229_65e23,
        50 => 7.500_866_746_076_96e24,
        52 => -5.038_778_101_481_07e26,
        54 => 3.652_877_648_481_81e28,
        56 => -2.849_876_930_245_09e30,
        58 => 2.386_542_749_968_36e32,
        60 => -2.139_994_925_722_53e34,
        _ => 0.,
    }
}

fn factorial(n: u64) -> f64
{
    match n
    {
        0 | 1 => 1.,
        2 => 2.,
        4 => 24.,
        6 => 720.0,
        _ => factorial(n - 1) * (n as f64),
    }
}

fn zeta_t(k: u64, nf: f64, s: Cplx) -> Cplx
{
    let two_k = k + k;
    let t0 = bernoulli(two_k) / factorial(two_k);
    let t1 = nf.powc(1. - s - (two_k as f64));
    let t2: Cplx = (0..two_k - 1).map(|j| s + (j as f64)).product();
    t0 * t1 * t2
}

fn zeta_t_d(k: u64, nf: f64, s: Cplx) -> [Cplx; 2]
{
    let two_k = k + k;
    let t0 = bernoulli(two_k) / factorial(two_k);
    let t1 = nf.powc(1. - s - (two_k as f64));
    let dt1 = -t1 * nf.ln();
    let t2: Cplx = (0..two_k - 1).map(|j| s + (j as f64)).product();
    let dt2: Cplx = (0..two_k - 1).map(|j| t2 / (s + (j as f64))).sum();
    [t0 * t1 * t2, t0 * (t1 * dt2 + dt1 * t2)]
}

fn zeta_t_d2(k: u64, nf: f64, s: Cplx) -> [Cplx; 3]
{
    let two_k = k + k;
    let t0 = bernoulli(two_k) / factorial(two_k);
    let t1d0 = nf.powc(1. - s - (two_k as f64));
    let t1d1 = -t1d0 * nf.ln();
    let t1d2 = -t1d1 * nf.ln();

    let t2d0: Cplx = (0..two_k - 1).map(|j| s + (j as f64)).product();
    let t2d1: Cplx = (0..two_k - 1).map(|j| t2d0 / (s + (j as f64))).sum();
    let t2d2: Cplx = (0..two_k - 1)
        .map(|j| {
            let v = s + (j as f64);
            (t2d1 * v - t2d0) / (v * v)
        })
        .sum();
    [
        t0 * t1d0 * t2d0,
        t0 * (t1d0 * t2d1 + t1d1 * t2d0),
        t0 * (t1d0 * t2d2 + 2. * t1d1 * t2d1 + t1d2 * t2d0),
    ]
}

// The Riemann zeta function
#[must_use]
pub fn riemann_zeta(s: Cplx) -> Cplx
{
    let n = 12;
    let m = 12;
    let u = 1. - s;
    let nf = f64::from(n);
    let s0: Cplx = (1..n).map(|j| f64::from(j).powc(-s)).sum();
    let s1 = 0.5 * nf.powc(-s);
    let s2 = nf.powc(u) / u;
    let s3: Cplx = (1..=m).map(|k| zeta_t(k, nf, s)).sum();

    s0 + s1 - s2 + s3
}

// The Riemann zeta function and its derivative
#[must_use]
pub fn riemann_zeta_d(s: Cplx) -> [Cplx; 2]
{
    let n = 12;
    let m = 12;
    let u = 1. - s;
    let nf = f64::from(n);
    let [s0, ds0]: [Cplx; 2] = (1..n)
        .map(|j| {
            let jf = f64::from(j);
            let term = jf.powc(-s);
            [term, -term * jf.ln()]
        })
        .fold([ZERO, ZERO], |[a, da], [b, db]| [a + b, da + db]);
    let s1 = 0.5 * nf.powc(-s);
    let ds1 = -s1 * nf.ln();
    let s2 = nf.powc(u) / u;
    let ds2 = s2 * (u.inv() - nf.ln());
    let [s3, ds3]: [Cplx; 2] = (1..=m)
        .map(|k| zeta_t_d(k, nf, s))
        .fold([ZERO, ZERO], |[a, da], [b, db]| [a + b, da + db]);

    [s0 + s1 - s2 + s3, ds0 + ds1 - ds2 + ds3]
}

// The Riemann zeta function and its first two derivatives
#[must_use]
pub fn riemann_zeta_d2(s: Cplx) -> [Cplx; 3]
{
    let n = 14;
    let m = 10;
    let u = 1. - s;
    let nf = f64::from(n);
    let (s0d0, s0d1, s0d2): (Cplx, Cplx, Cplx) = (1..n)
        .map(|j| {
            let jf = f64::from(j);
            let term = jf.powc(-s);
            let log_j = jf.ln();
            let dterm = term * log_j;
            (term, -dterm, dterm * log_j)
        })
        .fold((ZERO, ZERO, ZERO), |(a0, a1, a2), (b0, b1, b2)| {
            (a0 + b0, a1 + b1, a2 + b2)
        });

    let log_n = nf.ln();
    let s1d0 = 0.5 * nf.powc(-s);
    let s1d1 = -s1d0 * log_n;
    let s1d2 = -s1d1 * log_n;

    let u_inv = u.inv();
    let s2d0 = nf.powc(u) * u_inv;
    let alpha = u_inv - nf.ln();
    let s2d1 = s2d0 * alpha;
    let s2d2 = s2d1 * alpha + s2d0 * u_inv * u_inv;

    let [s3d0, s3d1, s3d2]: [Cplx; 3] = (1..=m)
        .map(|k| zeta_t_d2(k, nf, s))
        .fold([ZERO, ZERO, ZERO], |[a0, a1, a2], [b0, b1, b2]| {
            [a0 + b0, a1 + b1, a2 + b2]
        });

    [
        s0d0 + s1d0 - s2d0 + s3d0,
        s0d1 + s1d1 - s2d1 + s3d1,
        s0d2 + s1d2 - s2d2 + s3d2,
    ]
}

#[must_use]
pub fn riemann_xi(s: Cplx) -> Cplx
{
    let u = s * 0.5;
    u * (s - 1.) * PI.powc(-u) * gamma(u) * riemann_zeta(s)
}

#[must_use]
pub fn riemann_xi_d(s: Cplx) -> [Cplx; 2]
{
    if s.re < -5.
    {
        // avoid underflow issues for large neative s
        let [z0, z1] = riemann_xi_d(1.0 - s);
        return [z0, -z1];
    }
    let x0 = s * 0.5;
    let x1 = s - 1.;
    let x2 = PI.powc(-x0);
    let dx2 = -x2 * PI.ln();
    let x3 = gamma(x0);
    let dx3 = x3 * digamma(x0);
    let [x4, dx4] = riemann_zeta_d(s);
    let x01 = x0 * x1;
    [
        x01 * x2 * x3 * x4,
        x2 * x3 * x4 * (s - 0.5) + x01 * (x2 * x3 * dx4 + 0.5 * (dx2 * x3 * x4 + x2 * dx3 * x4)),
    ]
}

#[must_use]
pub fn riemann_xi_d2(s: Cplx) -> [Cplx; 3]
{
    if s.re < -5.
    {
        // avoid underflow issues for large neative s
        let [z0, z1, z2] = riemann_xi_d2(1.0 - s);
        return [z0, -z1, z2];
    }
    let [z0, z1, z2] = riemann_zeta_d2(s);

    let x0 = s - 1.;
    let x1 = 0.5 * s;

    let h = digamma(x1);
    let k = polygamma(x1, 1);
    let x3 = gamma(x1) * PI.powc(-x1);

    let x2 = z0 * x1;
    let x4 = z0 * x0;
    let x5 = 0.5 * x4;
    let x6 = x0 * z1;
    let x7 = x1 * x6;
    let x8 = s * x4;
    let x9 = 0.25 * x8;
    let x10 = h * x9;
    let x12 = 0.125 * x8;
    let y = x2 + x5 + x7;
    [
        x0 * x2 * x3,
        x3 * ((h - LOG_PI) * x9 + y),
        x3 * (h * (y + h * x12)
            + k * x12
            + s * z1
            + z0
            + x0 * x1 * z2
            + LOG_PI * (-x10 + x12 * LOG_PI - y)
            + x6),
    ]
}

pub fn roots_of_unity(degree: i32) -> impl Iterator<Item = Cplx>
{
    let theta = TAUI / f64::from(degree);
    (0..degree).map(move |k| (theta * f64::from(k)).exp())
}

pub fn nth_roots(x: Cplx, degree: i32) -> impl Iterator<Item = Cplx>
{
    let u = x.powf(1. / Real::from(degree));
    let theta = TAUI / f64::from(degree);
    (0..degree).map(move |k| u * (theta * f64::from(k)).exp())
}

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
        error = (z - z_old).norm_sqr();
        z_old = z;
    }
    z
}

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
        error = (z - z_old).norm_sqr();
        z_old = z;
    }
    (z, f, df)
}
