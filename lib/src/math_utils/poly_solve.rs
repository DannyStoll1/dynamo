use crate::types::Cplx;

#[cfg(feature = "mpsolve")]
pub fn solve_polynomial(coeffs: &[Cplx]) -> Vec<Cplx>
{
    unsafe {
        use crate::bindings::*;
        let mut ctx = mps_context_new();
        let degree = coeffs.len();
        let mut poly = mps_monomial_poly_new(ctx, degree.try_into().unwrap());

        coeffs.into_iter().enumerate().for_each(|(d, a)| {
            mps_monomial_poly_set_coefficient_d(ctx, poly, d as i64, a.re as f64, a.im as f64);
        });

        mps_mpsolve(ctx);
        let mut c_roots: *mut [__cplx_struct; 1] = Vec::with_capacity(degree).as_mut_ptr();
        let roots_ptr = std::ptr::addr_of_mut!(c_roots);

        let mut radius = 1e-8f64;
        let mut radius_ptr = std::ptr::addr_of_mut!(radius);
        let radius_ptr_ptr = std::ptr::addr_of_mut!(radius_ptr);

        mps_context_get_roots_d(ctx, roots_ptr, radius_ptr_ptr);

        let mut roots: Vec<Cplx> = Vec::with_capacity(degree);
        for i in 0..degree
        {
            let c_root = (*c_roots)[0];
            let root = Cplx::new(c_root.r, c_root.i);
            roots.push(root);
            c_roots = c_roots.wrapping_add(1);
        }
        roots
    }
}

#[cfg(not(feature = "mpsolve"))]
#[must_use] pub fn solve_polynomial(_coeffs: &[Cplx]) -> Vec<Cplx>
{
    vec![]
}
