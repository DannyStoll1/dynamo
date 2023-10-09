use crate::consts::{OMEGA, OMEGA_BAR, ONE_THIRD};
use crate::types::Cplx;

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
#[allow(clippy::suspicious_operation_groupings)]
#[must_use]
pub fn solve_quartic(a: Cplx, b: Cplx, c: Cplx, d: Cplx) -> [Cplx; 4]
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

#[cfg(feature = "mpsolve")]
#[allow(unused_mut)]
pub fn solve_polynomial(coeffs: &[Cplx]) -> Vec<Cplx>
{
    if coeffs.iter().any(|x| x.is_nan())
    {
        return vec![];
    }

    unsafe {
        use crate::bindings::*;
        let mut ctx = mps_context_new();
        let degree = coeffs.len() - 1;
        let mut poly = mps_monomial_poly_new(ctx, degree.try_into().unwrap());

        coeffs.iter().enumerate().for_each(|(d, a)| {
            mps_monomial_poly_set_coefficient_d(ctx, poly, d as i64, a.re as f64, a.im as f64);
        });

        mps_context_set_input_poly(ctx, poly as *mut mps_polynomial);
        mps_context_select_algorithm(ctx, 1);
        mps_context_set_output_prec(ctx, 64);
        mps_context_set_output_goal(ctx, 1);
        mps_mpsolve(ctx);

        // Initialize to null as MPSolve will do the allocation
        let mut c_roots: *mut [__cplx_struct; 1] = std::ptr::null_mut();
        let c_roots_ptr: *mut *mut [__cplx_struct; 1] = &mut c_roots;

        let mut radius: *mut f64 = std::ptr::null_mut();
        let radius_ptr: *mut *mut f64 = &mut radius;

        mps_context_get_roots_d(ctx, c_roots_ptr, radius_ptr);

        let mut roots: Vec<Cplx> = Vec::with_capacity(degree);
        for i in 0..degree
        {
            let c_root = (*c_roots.add(i))[0];
            let root = Cplx::new(c_root.r, c_root.i);
            roots.push(root);
        }

        // Don't forget to free the memory allocated by MPSolve
        libc::free(c_roots as *mut libc::c_void);
        libc::free(radius as *mut libc::c_void);

        roots
    }
}

#[cfg(not(feature = "mpsolve"))]
#[must_use]
pub const fn solve_polynomial(_coeffs: &[Cplx]) -> Vec<Cplx>
{
    Vec::new()
}
