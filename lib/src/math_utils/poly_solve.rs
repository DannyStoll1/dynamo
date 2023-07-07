use crate::types::Cplx;

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
pub fn solve_polynomial(_coeffs: &[Cplx]) -> Vec<Cplx>
{
    vec![]
}
