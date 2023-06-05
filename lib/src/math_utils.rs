use crate::consts::*;
use crate::types::{Cplx, Real};
use num_complex::ComplexFloat;
use spfunc::gamma::{digamma, gamma};
use std::f64::consts::PI;

#[must_use]
pub fn weierstrass_p(
    g2: Cplx,
    g3: Cplx,
    z: Cplx,
    tolerance: Real,
) -> (Cplx, Cplx)
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

// Roots of the polynomial a + bx + x^2
#[must_use]
pub fn solve_quadratic(a: Cplx, b: Cplx) -> [Cplx; 2]
{
    let disc = (b * b - 4. * a).sqrt();
    [-0.5 * (b + disc), 0.5 * (disc - b)]
}

// Roots of the polynomial a + bx + cx^2 + x^3
#[must_use]
pub fn solve_cubic(a: Cplx, b: Cplx, c: Cplx) -> [Cplx; 3]
{
    let x0 = -c / 3.;
    let c2 = c * c;
    let c3 = c * c2;
    let bc = b * c;
    let d0 = -3. * b + c2;
    let d1 = 27. * a + 2. * c3 - 9. * bc;
    let disc = (0.5 * (d1 + (d1 * d1 - 4. * d0 * d0 * d0).sqrt())).powf(ONE_THIRD);
    let x5 = -disc * ONE_THIRD;
    let x6 = -d0 / (3. * disc);
    [
        x0 + x5 + x6,
        x0 + OMEGA * x5 + OMEGA_BAR * x6,
        x0 + OMEGA_BAR * x5 + OMEGA * x6,
    ]
}

// Roots of the polynomial a + bx + cx^2 + dx^3 + x^4
#[must_use]
pub fn solve_quartic(a: Cplx, b: Cplx, c: Cplx, d: Cplx)
    -> [Cplx; 4]
{
    let c2 = c * c;
    let d2 = d * d;
    let bd = b * d;

    let disc_0 = c2 - 3. * bd + 12. * a;
    let disc_1 = c * (c2 + c2 - 9. * bd - 72. * a) + 27. * (d2 * a + b * b);

    let p = c - 0.375 * d2;
    let q = 0.5 * d * (0.25 * d2 - c) + b;

    let q1 = (0.5 * (disc_1 + (disc_1 * disc_1 - 4. * disc_0.powi(3)).sqrt())).powf(ONE_THIRD);
    let s = 0.5 * (ONE_THIRD * (q1 + disc_0 / q1 - p - p)).sqrt();

    let x0 = -0.25 * d;
    let u = -4. * s * s - p - p;
    let v = q / s;

    let disc_2 = 0.5 * (u + v).sqrt();
    let disc_3 = 0.5 * (u - v).sqrt();

    [
        x0 - s + disc_2,
        x0 - s - disc_2,
        x0 + s + disc_3,
        x0 + s - disc_3,
    ]
}

fn bernoulli(n: u64) -> f64
{
    match n
    {
        0 => 1.,
        1 => -0.5,
        2 => 1. / 6.,
        4 | 8 => -1. / 30.,
        6 => 1. / 42.,
        10 => 5. / 66.,
        12 => -691. / 2730.,
        14 => 7. / 6.,
        16 => -3617. / 510.,
        18 => 43867. / 798.,
        20 => -174611. / 330.,
        22 => 854513. / 138.,
        24 => -236364091. / 2730.,
        26 => 8553103. / 6.,
        28 => -23749461029. / 870.,
        30 => 8615841276005. / 14322.,
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

fn zeta_t_d(k: u64, nf: f64, s: Cplx) -> (Cplx, Cplx)
{
    let two_k = k + k;
    let t0 = bernoulli(two_k) / factorial(two_k);
    let t1 = nf.powc(1. - s - (two_k as f64));
    let dt1 = -t1 * nf.ln();
    let t2: Cplx = (0..two_k - 1).map(|j| s + (j as f64)).product();
    let dt2: Cplx = (0..two_k - 1).map(|j| t2 / (s + (j as f64))).sum();
    (t0 * t1 * t2, t0 * (t1 * dt2 + dt1 * t2))
}

pub fn riemann_zeta(s: Cplx) -> Cplx
{
    let n = 20;
    let m = 19;
    let u = 1. - s;
    let nf = n as f64;
    let s0: Cplx = (1..n).map(|j| (j as f64).powc(-s)).sum();
    let s1 = 0.5 * nf.powc(-s);
    let s2 = nf.powc(u) / u;
    let s3: Cplx = (1..=m).map(|k| zeta_t(k, nf, s)).sum();

    s0 + s1 - s2 + s3
}

pub fn riemann_zeta_d(s: Cplx) -> (Cplx, Cplx)
{
    let n = 10;
    let m = 9;
    let u = 1. - s;
    let nf = n as f64;
    let (s0, ds0): (Cplx, Cplx) = (1..n)
        .map(|j| {
            let jf = j as f64;
            let term = jf.powc(-s);
            (term, -term * jf.ln())
        })
        .fold((ZERO, ZERO), |(a, da), (b, db)| (a + b, da + db));
    let s1 = 0.5 * nf.powc(-s);
    let ds1 = -s1 * nf.ln();
    let s2 = nf.powc(u) / u;
    let ds2 = s2 * (u.inv() - nf.ln());
    let (s3, ds3): (Cplx, Cplx) = (1..=m)
        .map(|k| zeta_t_d(k, nf, s))
        .fold((ZERO, ZERO), |(a, da), (b, db)| (a + b, da + db));

    (s0 + s1 - s2 + s3, ds0 + ds1 - ds2 + ds3)
}

pub fn riemann_xi(s: Cplx) -> Cplx
{
    let u = s * 0.5;
    u * (s - 1.) * PI.powc(-u) * gamma(u) * riemann_zeta(s)
}

pub fn riemann_xi_d(s: Cplx) -> (Cplx, Cplx)
{
    let x0 = s * 0.5;
    let x1 = s - 1.;
    let x2 = PI.powc(-x0);
    let dx2 = -x2 * PI.ln();
    let x3 = gamma(x0);
    let dx3 = x3 * digamma(x0);
    let (x4, dx4) = riemann_zeta_d(s);
    let x01 = x0 * x1;
    (
        x01 * x2 * x3 * x4,
        x2 * x3 * x4 * (s - 0.5) + x01 * (x2 * x3 * dx4 + 0.5 * (dx2 * x3 * x4 + x2 * dx3 * x4)),
    )
}

pub fn roots_of_unity(degree: i32) -> impl Iterator<Item=Cplx>
{
    let theta = TAUI / (degree as Real);
    (0..degree)
        .map(move|k| (theta.clone() * (k as Real)).exp())
}
