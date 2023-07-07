#![feature(const_fn_floating_point_arithmetic)]
#![allow(dead_code)]

pub mod coloring;
pub mod consts;
pub mod iter_plane;
pub mod macros;
pub mod math_utils;
pub mod point_grid;
pub mod types;

#[cfg(feature = "mpsolve")]
mod bindings;
#[cfg(not(feature = "mpsolve"))]
mod bindings
{}

#[cfg(test)]
mod tests
{
    use crate::types::Cplx;

    #[test]
    fn quartic_roots()
    {
        use crate::macros::horner_monic;
        use crate::math_utils::poly_solve::solve_quartic;

        let a = Cplx::new(2., 0.);
        let b = Cplx::new(3., 0.);
        let c = Cplx::new(5., 0.);
        let d = Cplx::new(7., 0.);

        let roots = solve_quartic(a, b, c, d);
        for r in roots.iter()
        {
            let val = horner_monic!(r, a, b, c, d);
            assert!(val.norm() < 1e-13);
        }
    }

    #[test]
    fn zeta_d()
    {
        use crate::math_utils::riemann_zeta_d;
        let s = Cplx::new(0.5, 14.134_725_141_734_695);
        let mut val = Cplx::default();
        let mut dval = Cplx::default();
        for _ in 0..50000
        {
            [val, dval] = riemann_zeta_d(s);
        }
        let err = val.norm();
        let dval_true = Cplx::new(0.783_296_511_867_031, 0.124_699_829_748_171_09);
        let derr = (dval - dval_true).norm();
        dbg!(err, derr);
        assert!(err < 1e-14); // 1.6274073245768128e-15
        assert!(derr < 1e-14); // 3.234090342272182e-15
    }

    #[test]
    fn zeta_d2()
    {
        use crate::math_utils::riemann_zeta_d2;
        let s = Cplx::new(0.5, 14.134_725_141_734_695);
        let mut val0 = Cplx::default();
        let mut val1 = Cplx::default();
        let mut val2 = Cplx::default();

        for _ in 0..50000
        {
            [val0, val1, val2] = riemann_zeta_d2(s);
        }

        let val1_true = Cplx::new(0.783_296_511_867_031, 0.124_699_829_748_171_09);
        let val2_true = Cplx::new(-0.614409794662293, -0.229783642987604);

        let err0 = val0.norm();
        let err1 = (val1 - val1_true).norm();
        let err2 = (val2 - val2_true).norm();
        dbg!(err0, err1, err2);
        assert!(err0 < 1e-14); // 1.6274073245768128e-15
        assert!(err1 < 1e-14); // 3.234090342272182e-15
        assert!(err2 < 1e-14); // 3.234090342272182e-15
    }

    #[test]
    fn xi_d2()
    {
        use crate::math_utils::riemann_xi_d2;
        let s = Cplx::new(0.5, 14.134_725_141_734_695);
        let mut val0 = Cplx::default();
        let mut val1 = Cplx::default();
        let mut val2 = Cplx::default();

        for _ in 0..50000
        {
            [val0, val1, val2] = riemann_xi_d2(s);
        }

        let val1_true = Cplx::new(0., 0.00138271908921625);
        let val2_true = Cplx::new(-0.00160293252834422, 0.);

        let err0 = val0.norm();
        let err1 = (val1 - val1_true).norm();
        let err2 = (val2 - val2_true).norm();
        dbg!(err0, err1, err2);
        assert!(err0 < 5e-14); // 3.5713649459283646e-18
        assert!(err1 < 5e-14); // 5.4935093103301e-14
        assert!(err2 < 5e-14); // 6.368821534010004e-14
    }

    #[test]
    fn zeta()
    {
        use crate::math_utils::riemann_zeta;
        let s = Cplx::new(0.5, 14.134_725_141_734_695);
        let mut val = Cplx::default();
        for _ in 0..10000
        {
            val = riemann_zeta(s);
        }
        let err = val.norm();
        dbg!(err); // 1.6274073245768128e-15
        assert!(err < 1e-14);
    }

    #[test]
    fn zeta_spfunc()
    {
        use spfunc::zeta::zeta;
        let s = Cplx::new(0.5, 14.134_725_141_734_695);
        let mut val = Cplx::default();
        for _ in 0..10000
        {
            val = zeta(s);
        }
        let err = val.norm();
        dbg!(err); // 5.90604839085258e-16,
        assert!(err < 1e-14);
    }

    #[test]
    fn gamma_spfunc()
    {
        use spfunc::gamma::polygamma;
        let s = Cplx::new(0.5, 14.134_725_141_734_695);
        let mut val = Cplx::default();
        for _ in 0..10000
        {
            val = polygamma(s, 1);
        }
        dbg!(val);
    }

    #[test]
    fn xi()
    {
        use crate::math_utils::riemann_xi_d;
        use std::f64::consts::FRAC_PI_6;
        let s = Cplx::new(2., 0.);
        let [val, dval] = riemann_xi_d(s);
        let val_true = Cplx::from(FRAC_PI_6);
        let dval_true = Cplx::new(0.036_162_994_264_296_92, 0.);
        let err = (val - val_true).norm();
        let derr = (dval - dval_true).norm();
        dbg!(err, derr);
        assert!(err < 1e-11);
        assert!(derr < 1e-11);
    }
}
