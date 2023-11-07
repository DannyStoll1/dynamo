#![feature(const_fn_floating_point_arithmetic)]
#![allow(dead_code)]

pub mod consts;
pub mod globals;
pub mod iter_plane;
pub mod macros;
pub mod math_utils;
pub mod point_grid;
pub mod point_info;
pub mod prelude;
pub mod rational_angle;
pub mod symbolic_dynamics;
pub mod traits;
pub mod types;

pub mod cache
{
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    #[derive(Clone, Debug)]
    pub struct Cache<K, V>
    where
        K: std::hash::Hash + Eq + Clone,
        V: Clone,
    {
        data: Arc<Mutex<HashMap<K, V>>>,
    }

    impl<K, V> Cache<K, V>
    where
        K: std::hash::Hash + Eq + Clone,
        V: Clone,
    {
        #[must_use]
        pub fn new() -> Self
        {
            Self {
                data: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        pub fn get(&self, key: &K) -> Option<V>
        {
            match self.data.lock() {
                Ok(lock) => lock.get(key).cloned(),
                Err(poisoned) => poisoned.into_inner().get(key).cloned(),
            }
        }

        pub fn insert(&self, key: K, value: V)
        {
            match self.data.lock() {
                Ok(mut lock) => {
                    lock.insert(key, value);
                }
                Err(poisoned) => {
                    poisoned.into_inner().insert(key, value);
                }
            }
        }
    }

    impl<K, V> Default for Cache<K, V>
    where
        K: std::hash::Hash + Eq + Clone,
        V: Clone,
    {
        fn default() -> Self
        {
            Self::new()
        }
    }
}

#[cfg(test)]
mod tests
{
    use crate::{
        prelude::{OrbitSchema, RationalAngle},
        types::Cplx,
    };

    #[test]
    fn quartic_roots()
    {
        use crate::macros::horner_monic;
        use crate::math_utils::polynomial_roots::solve_quartic;

        let a = Cplx::new(2., 0.);
        let b = Cplx::new(3., 0.);
        let c = Cplx::new(5., 0.);
        let d = Cplx::new(7., 0.);

        let roots = solve_quartic(a, b, c, d);
        for r in &roots {
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
        for _ in 0..50000 {
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

        for _ in 0..50000 {
            [val0, val1, val2] = riemann_zeta_d2(s);
        }

        let val1_true = Cplx::new(0.783_296_511_867_031, 0.124_699_829_748_171_09);
        let val2_true = Cplx::new(-0.614_409_794_662_293, -0.229_783_642_987_604);

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

        for _ in 0..50000 {
            [val0, val1, val2] = riemann_xi_d2(s);
        }

        let val1_true = Cplx::new(0., 0.001_382_719_089_216_25);
        let val2_true = Cplx::new(-0.001_602_932_528_344_22, 0.);

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
        for _ in 0..10000 {
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
        for _ in 0..10000 {
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
        for _ in 0..10000 {
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

    #[test]
    fn sort_circ()
    {
        use crate::symbolic_dynamics::sort_circularly_ordered;
        use std::collections::VecDeque;
        let mut values = VecDeque::from([3, 4, 8, 8, 9, 0, 2]);
        sort_circularly_ordered(&mut values);

        assert_eq!(values, [0, 2, 3, 4, 8, 8, 9]);
    }

    #[test]
    fn active_angles()
    {
        let o = OrbitSchema {
            preperiod: 0,
            period: 3,
        };
        let angles = o.with_degree(-2).child_angles();
        dbg!(angles);
    }

    #[test]
    fn fmt_angle()
    {
        let angle = RationalAngle::new(7, 17);
        let s0 = format!("{angle:>10}");
        assert_eq!(s0, "      7/17");

        let it = angle.with_degree(2);
        let s = format!("{it:>13}");
        assert_eq!(s, "    p01101001");
    }
}
