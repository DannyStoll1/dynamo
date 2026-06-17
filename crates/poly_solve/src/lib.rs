pub mod newton;
pub mod normed;
pub mod poly_traits;
pub mod polynomial;
pub mod solve;
pub mod utils;

use num_complex::Complex64;
use polynomial::Polynomial;
use solve::JenkinsTraubSolver;

/// Find all roots of a given polynomial.
pub fn solve_polynomial<P>(poly: P) -> Vec<Complex64>
where
    P: Into<Polynomial<Complex64>>,
{
    let mut solver = JenkinsTraubSolver::new(poly.into());
    solver.find_all_roots()
}

#[cfg(test)]
mod tests
{
    use num_complex::Complex64;

    use crate::newton::Newton as _;
    use crate::poly_traits::Eval as _;
    use crate::polynomial::Polynomial;
    use crate::solve::JenkinsTraubSolver;
    use crate::solve_polynomial;
    #[test]
    fn poly_division()
    {
        use crate::poly_traits::DivideByAffine as _;

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

        for sol in &sols {
            assert!(poly0.eval(*sol).norm_sqr() < 1e-14);
        }
    }

    const HIGH_DEGREE_COEFFS: [(f64, f64); 44] = [
        (-5_566_639.898_816_645, -3_057_559.874_417_730_6),
        (-1_850_933.237_105_822_2, -5_936_871.660_945_967),
        (2_799_352.215_297_003_3, 26_422_838.313_466_772),
        (-23_805_474.092_002_384, 30_757_416.232_245_553),
        (53_843_432.477_633_45, -57_481_536.275_743_96),
        (143_464_715.086_846_74, -3_027_185.062_396_222),
        (-178_981_199.928_231_33, 7_777_010.625_380_026),
        (-263_696_215.704_023_54, -276_805_328.477_711_44),
        (258_497_054.957_991, 198_528_518.729_044_14),
        (-11_064_764.964_479_223, 790_495_977.587_424),
        (-139_925_323.026_810_14, -466_041_069.901_670_46),
        (882_030_516.433_737_2, -973_933_050.534_668_4),
        (-154_906_140.746_533_27, 585_492_175.698_505_2),
        (-1_762_871_138.973_297_8, 293_372_509.259_018_6),
        (426_647_031.184_225_8, -469_212_474.260_950_4),
        (1_801_648_743.662_337_5, 881_165_981.209_272_9),
        (-513_503_138.161_617_76, 227_224_477.140_133_3),
        (-962_144_269.730_246_9, -1_594_645_947.568_012),
        (420_863_668.371_851_8, -21_228_874.534_064_05),
        (16_611_330.892_319_413, 1_431_272_250.732_229_5),
        (-257_288_784.803_387_85, -74_208_454.451_204_91),
        (423_012_142.025_444_3, -805_267_600.897_382_1),
        (119_088_336.699_065_57, 79_900_144.320_926_28),
        (-387_733_344.673_961_34, 278_052_418.780_711_77),
        (-40_391_065.478_397_18, -49_923_586.130_099_64),
        (202_569_662.942_827_8, -37_356_144.898_768_27),
        (8_929_351.553_170_01, 22_060_509.736_583_628),
        (-70_827_598.668_740_05, -17_069_243.062_905_703),
        (-645_312.235_767_996_4, -7_183_867.924_887_203),
        (16_838_185.077_967_968, 12_483_342.235_453_077),
        (-360_680.945_811_137_8, 1_724_843.252_477_051_7),
        (-2_519_673.268_324_85, -4_147_678.738_679_482_6),
        (164_188.120_612_411_85, -298_562.383_124_508_7),
        (151_641.286_641_951_8, 876_898.026_875_663_3),
        (-36_474.210_168_885_01, 35_418.019_284_098_555),
        (24_204.812_767_887_91, -123_854.013_897_651),
        (5_018.163_826_520_113, -2_569.246_853_472_343),
        (-6_952.001_360_253_172, 11_383.676_140_434_778),
        (-430.224_444_974_841_45, 76.468_815_729_153_55),
        (771.848_384_458_199_6, -617.936_601_845_418_2),
        (20.957_471_656_259_973, 2.543_445_495_286_715),
        (-43.299_444_296_983_005, 15.049_822_067_622_886),
        (-0.437_402_731_233_189_6, -0.193_510_601_586_984_33),
        (1.0, 0.0),
    ];

    #[test]
    fn high_degree()
    {
        let poly: Polynomial<Complex64> = HIGH_DEGREE_COEFFS
            .iter()
            .map(|&(re, im)| Complex64::new(re, im))
            .collect();
        let roots = solve_polynomial(poly.clone());
        assert!(poly.eval(roots[0]).norm_sqr().is_finite());
    }

    #[test]
    fn zero_coeffs()
    {
        let poly: Polynomial<Complex64> = [0., 0., 1., 0.].iter().map(Complex64::from).collect();
        let roots = solve_polynomial(poly);
        assert!(!roots.is_empty());
    }
}
