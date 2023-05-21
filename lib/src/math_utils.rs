use crate::types::{ComplexNum, RealNum, OMEGA, OMEGA_BAR, ONE_THIRD};

#[must_use]
pub fn weierstrass_p(
    g2: ComplexNum,
    g3: ComplexNum,
    z: ComplexNum,
    tolerance: RealNum,
) -> (ComplexNum, ComplexNum)
{
    let num_iters = (z.norm() / tolerance).log2().round() as i32 + 1;
    let shrink_scale = (2.0 as RealNum).powi(-num_iters);
    let z0 = z * shrink_scale;

    let u = z0 * z0;

    let mut p = 1. / u + g2 * u / 20. + g3 * u * u / 28.;
    let mut dp = -2. / (u * z0) + g2 * z0 / 10. + g3 * u * z0 / 7.;

    let mut p_2: ComplexNum;
    let mut dp_2: ComplexNum;
    let mut ddp: ComplexNum;
    let mut ddp_2: ComplexNum;
    let mut tmp: ComplexNum;
    let mut four_dp_2: ComplexNum;

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
pub fn slog(x: RealNum) -> RealNum
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

// Roots of the polynomial a + bx + cx^2 + x^3
#[must_use]
pub fn solve_cubic(
    a: ComplexNum,
    b: ComplexNum,
    c: ComplexNum,
) -> (ComplexNum, ComplexNum, ComplexNum)
{
    let x0 = -c / 3.;
    let c2 = c * c;
    let c3 = c * c2;
    let bc = b * c;
    let d0 = -3. * b + c2;
    let d1 = 27.*a + 2. * c3 - 9. * bc;
    let disc = (0.5 * (d1 + (d1 * d1 - 4. * d0 * d0 * d0).sqrt())).powf(ONE_THIRD);
    let x5 = -disc * ONE_THIRD;
    let x6 = -d0 / (3. * disc);
    (
        x0 + x5 + x6,
        x0 + OMEGA * x5 + OMEGA_BAR * x6,
        x0 + OMEGA_BAR * x5 + OMEGA * x6,
    )
}
