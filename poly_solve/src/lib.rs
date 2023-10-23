#![allow(dead_code)]

pub mod newton;
pub mod normed;
pub mod poly_traits;
pub mod polynomial;
pub mod solve;
pub mod utils;

#[cfg(test)]
mod tests
{
    use num_complex::Complex64;

    use crate::{
        newton::Newton,
        poly_traits::Eval,
        polynomial::Polynomial,
        solve::{solve_polynomial, JenkinsTraubSolver},
    };
    #[test]
    fn poly_division()
    {
        use crate::poly_traits::DivideByAffine;

        let mut poly = Polynomial::from([24, 2, 13, 5]);
        let target = Polynomial::from([8, -2, 5]);

        let quotient = poly.divide_by_affine(-3);
        poly.divide_by_affine_inplace(-3);
        assert_eq!(quotient, target);
        assert_eq!(poly, target);
    }

    #[test]
    fn poly_addition()
    {
        let poly0: Polynomial<i32> = Polynomial::from([2, 3, 5, 7]);
        let poly1: Polynomial<i32> = Polynomial::from([-2, 1, -5]);
        let poly2: Polynomial<i32> = Polynomial::from([-2, 1, -5, -7]);
        let poly3: Polynomial<i32> = Polynomial::from([-4, -3, 2, 6, 2]);

        assert_eq!(
            poly0.clone() + poly1.clone(),
            Polynomial::from([0, 4, 0, 7])
        );
        assert_eq!(poly0.clone() + poly2.clone(), Polynomial::from([0, 4]));
        assert_eq!(
            poly0.clone() + poly3.clone(),
            Polynomial::from([-2, 0, 7, 13, 2])
        );

        let mut poly4 = poly0.clone();
        poly4 += poly1.clone();
        assert_eq!(poly4, poly0.clone() + poly1);

        let mut poly4 = poly0.clone();
        poly4 += poly2.clone();
        assert_eq!(poly4, poly0.clone() + poly2);

        let mut poly4 = poly0.clone();
        poly4 += poly3.clone();
        assert_eq!(poly4, poly0 + poly3);
    }

    #[test]
    fn newton()
    {
        type Cplx = num_complex::Complex64;

        let poly0: Polynomial<Cplx> = [2., 3., 5., 7.].iter().map(Cplx::from).collect();

        let start = Cplx::new(-2.71, 5.7);

        let sol = poly0
            .find_root_newton(start, 1e-14)
            .expect("Failed to converge to root!");

        assert!(poly0.eval(sol).norm_sqr() < 1e-14);
    }

    #[test]
    fn jtsolve()
    {
        type Cplx = num_complex::Complex64;

        let poly0: Polynomial<Cplx> = [2., 3., 5., 7.].iter().map(Cplx::from).collect();

        let mut solver = JenkinsTraubSolver::new(poly0.clone());

        let sol = solver.find_smallest_root();

        assert!(poly0.eval(sol).norm_sqr() < 1e-14);
    }

    #[test]
    fn jtroots()
    {
        type Cplx = num_complex::Complex64;

        let poly0: Polynomial<Cplx> = [2., 3., 5., 7.].iter().map(Cplx::from).collect();

        let mut solver = JenkinsTraubSolver::new(poly0.clone());

        let sols = solver.find_all_roots();

        for sol in sols.iter()
        {
            assert!(poly0.eval(*sol).norm_sqr() < 1e-14);
        }
    }

    #[test]
    fn high_degree()
    {
        let poly = Polynomial::from([
            Complex64 {
                re: -5566639.898816645,
                im: -3057559.8744177306,
            },
            Complex64 {
                re: -1850933.2371058222,
                im: -5936871.660945967,
            },
            Complex64 {
                re: 2799352.2152970033,
                im: 26422838.313466772,
            },
            Complex64 {
                re: -23805474.092002384,
                im: 30757416.232245553,
            },
            Complex64 {
                re: 53843432.47763345,
                im: -57481536.27574396,
            },
            Complex64 {
                re: 143464715.08684674,
                im: -3027185.062396222,
            },
            Complex64 {
                re: -178981199.92823133,
                im: 7777010.625380026,
            },
            Complex64 {
                re: -263696215.70402354,
                im: -276805328.47771144,
            },
            Complex64 {
                re: 258497054.957991,
                im: 198528518.72904414,
            },
            Complex64 {
                re: -11064764.964479223,
                im: 790495977.587424,
            },
            Complex64 {
                re: -139925323.02681014,
                im: -466041069.90167046,
            },
            Complex64 {
                re: 882030516.4337372,
                im: -973933050.5346684,
            },
            Complex64 {
                re: -154906140.74653327,
                im: 585492175.6985052,
            },
            Complex64 {
                re: -1762871138.9732978,
                im: 293372509.2590186,
            },
            Complex64 {
                re: 426647031.1842258,
                im: -469212474.2609504,
            },
            Complex64 {
                re: 1801648743.6623375,
                im: 881165981.2092729,
            },
            Complex64 {
                re: -513503138.16161776,
                im: 227224477.1401333,
            },
            Complex64 {
                re: -962144269.7302469,
                im: -1594645947.568012,
            },
            Complex64 {
                re: 420863668.3718518,
                im: -21228874.53406405,
            },
            Complex64 {
                re: 16611330.892319413,
                im: 1431272250.7322295,
            },
            Complex64 {
                re: -257288784.80338785,
                im: -74208454.45120491,
            },
            Complex64 {
                re: 423012142.0254443,
                im: -805267600.8973821,
            },
            Complex64 {
                re: 119088336.69906557,
                im: 79900144.32092628,
            },
            Complex64 {
                re: -387733344.67396134,
                im: 278052418.78071177,
            },
            Complex64 {
                re: -40391065.47839718,
                im: -49923586.13009964,
            },
            Complex64 {
                re: 202569662.9428278,
                im: -37356144.89876827,
            },
            Complex64 {
                re: 8929351.55317001,
                im: 22060509.736583628,
            },
            Complex64 {
                re: -70827598.66874005,
                im: -17069243.062905703,
            },
            Complex64 {
                re: -645312.2357679964,
                im: -7183867.924887203,
            },
            Complex64 {
                re: 16838185.077967968,
                im: 12483342.235453077,
            },
            Complex64 {
                re: -360680.9458111378,
                im: 1724843.2524770517,
            },
            Complex64 {
                re: -2519673.26832485,
                im: -4147678.7386794826,
            },
            Complex64 {
                re: 164188.12061241185,
                im: -298562.3831245087,
            },
            Complex64 {
                re: 151641.2866419518,
                im: 876898.0268756633,
            },
            Complex64 {
                re: -36474.21016888501,
                im: 35418.019284098555,
            },
            Complex64 {
                re: 24204.81276788791,
                im: -123854.013897651,
            },
            Complex64 {
                re: 5018.163826520113,
                im: -2569.246853472343,
            },
            Complex64 {
                re: -6952.001360253172,
                im: 11383.676140434778,
            },
            Complex64 {
                re: -430.22444497484145,
                im: 76.46881572915355,
            },
            Complex64 {
                re: 771.8483844581996,
                im: -617.9366018454182,
            },
            Complex64 {
                re: 20.957471656259973,
                im: 2.543445495286715,
            },
            Complex64 {
                re: -43.299444296983005,
                im: 15.049822067622886,
            },
            Complex64 {
                re: -0.4374027312331896,
                im: -0.19351060158698433,
            },
            Complex64 { re: 1.0, im: 0.0 },
        ]);
        let roots = solve_polynomial(poly.clone());
        dbg!(poly.eval(roots[0]).norm_sqr());
    }

    #[test]
    fn zero_coeffs()
    {
        let poly: Polynomial<Complex64> = [0., 0., 1., 0.].iter().map(Complex64::from).collect();
        let roots = solve_polynomial(poly);
        dbg!(&roots);
    }
}
