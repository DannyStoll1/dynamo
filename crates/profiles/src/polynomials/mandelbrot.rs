use dynamo_common::symbolic_dynamics::OrbitSchema;

use crate::macros::{cplx_arr, degree_impl, horner, horner_monic, profile_imports};

profile_imports!();

#[derive(Clone, Debug)]
pub struct Mandelbrot
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl Mandelbrot
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -2.1,
        max_x: 0.55,
        min_y: -1.25,
        max_y: 1.25,
    };
}
impl Default for Mandelbrot
{
    fractal_impl!();
}

impl DynamicalFamily for Mandelbrot
{
    parameter_plane_impl!();
    default_name!();

    fn escape_radius(&self) -> Real
    {
        1e26
    }

    fn start_point(&self, _point: Cplx, _c: &Self::Param) -> Self::Var
    {
        ZERO
    }

    #[inline]
    fn map(&self, z: Self::Var, c: &Self::Param) -> Self::Var
    {
        z.powi(2) + c
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv)
    {
        (z.powi(2) + c, 2. * z)
    }

    fn early_bailout(&self, _start: Cplx, c: &Self::Param) -> Option<EscapeResult<Cplx, Cplx>>
    {
        // Main cardioid
        let four_c = 4. * c;
        let y2 = four_c.im * four_c.im;
        let temp = four_c.re - 1.;
        let mu_norm2 = temp.mul_add(temp, y2);
        let a = mu_norm2 * mu_norm2.mul_add(0.25, temp);

        if a < y2 {
            let multiplier = 1. - (1. - four_c).sqrt();
            let mult_norm2 = multiplier.norm_sqr();
            let fixed_point = 0.5 * multiplier;
            let init_dist = (c - fixed_point).norm_sqr();
            let potential = -2. * (init_dist / self.periodicity_tolerance()).log(mult_norm2);
            return Some(EscapeResult::KnownPotential(PointInfoKnownPotential {
                period: 1,
                multiplier,
                potential,
            }));
        }

        // Basilica bulb
        let mu2 = four_c + 4.;
        let mult_norm2 = mu2.norm_sqr();
        if mult_norm2 < 1. {
            let fixed_point = -0.5 - 0.5 * (-four_c - 3.).sqrt();
            let init_dist = (c - fixed_point).norm_sqr();
            let potential = -4. * (init_dist / self.periodicity_tolerance()).log(mult_norm2);
            return Some(EscapeResult::KnownPotential(PointInfoKnownPotential {
                period: 2,
                potential,
                multiplier: mu2,
            }));
        }

        None
    }

    fn description(&self) -> String
    {
        "The moduli space of quadratic polynomials, \
            parameterized in the coordinates $f_c(z) = z^2 + c$, \
            All such maps have a fixed critical point at infinity \
            and a free critical point at 0. A given parameter $c$ is \
            colored according to the activity of the free critical point \
            under forward iteration of $f_c$."
            .to_owned()
    }
}

impl FamilyDefaults for Mandelbrot
{
    default_bounds!();
}

impl HasJulia for Mandelbrot
{
    #[inline]
    fn default_bounds_child(&self, _point: Cplx, _param: &Cplx) -> Bounds
    {
        Bounds::centered_square(2.2)
    }
}

impl HasDynamicalCovers for Mandelbrot
{
    fn marked_cycle_curve(self, period: Period) -> CoveringMap<Self>
    {
        match period {
            1 => {
                let param_map = |c| (0.25 - c * c, -2. * c);
                let bounds = Bounds {
                    min_x: -1.8,
                    max_x: 1.8,
                    min_y: -1.0,
                    max_y: 1.0,
                };
                CoveringMap::new(self, param_map).with_orig_bounds(bounds)
            }
            3 => {
                let param_map = |t| (-0.25 * (t * t + 7.), -0.5 * t);
                let mult = |t| (horner_monic!(t, 1., 7., -1.), horner!(t, 7., -2., 3.));
                let bounds = Bounds {
                    min_x: -2.1,
                    max_x: 2.1,
                    min_y: -3.5,
                    max_y: 3.5,
                };
                CoveringMap::new(self, param_map)
                    .with_orig_bounds(bounds)
                    .with_multiplier_map(mult)
            }
            4 => {
                let param_map = |t: Cplx| {
                    let t2 = t * t;
                    (-0.25 * t2 - 0.75 - t.inv(), -0.5 * t + t2.inv())
                };
                let mult = |t: Cplx| {
                    let t2 = t.powi(2);
                    let a = horner_monic!(t2, -16., -5., 4.);
                    let b = horner!(t2, -8., 6., 2.);
                    let u = (-8. * t - 32.) / (t * t2);
                    let v = horner!(t, -6., -8., -6., -4.);
                    (-(a + t * b) / t2, u + v)
                };

                let bounds = Bounds {
                    min_x: -2.9,
                    max_x: 2.1,
                    min_y: -3.1,
                    max_y: 3.1,
                };
                CoveringMap::new(self, param_map)
                    .with_orig_bounds(bounds)
                    .with_multiplier_map(mult)
            }
            _ => CoveringMap::from(self),
        }
    }

    fn dynatomic_curve(self, period: Period) -> CoveringMap<Self>
    {
        match period {
            1 => {
                let param_map = |t: Cplx| (0.25 - t.powi(2), -2. * t);
                let bounds = Bounds {
                    min_x: -1.8,
                    max_x: 1.8,
                    min_y: -1.0,
                    max_y: 1.0,
                };
                CoveringMap::new(self, param_map).with_orig_bounds(bounds)
            }
            2 => {
                let param_map = |t: Cplx| {
                    let u = 9. / t.powi(2);
                    ((t - 1.) * u - 3., (2. - t) * u / t)
                };
                let bounds = Bounds {
                    min_x: 0.5,
                    max_x: 8.3,
                    min_y: -2.7,
                    max_y: 2.7,
                };
                CoveringMap::new(self, param_map).with_orig_bounds(bounds)
            }
            3 => {
                let param_map = |t: Cplx| {
                    let t2 = t * t;

                    let v = t2 * (t2 - 3. * t + 6.) - 2. * t + 2.;
                    let dv_dt = horner!(t, -2., 12., -9., 4.);

                    let w = (t2 - t).inv();
                    let dw_dt = (1. - 2. * t) * w * w;

                    let u = v + w;
                    let du_dt = dv_dt + dw_dt;
                    (-0.25 * u * w, -0.25 * (du_dt * w + u * dw_dt))
                };
                let mult = |t: Cplx| {
                    let t2 = t.powi(2);
                    let t3 = t * t2;
                    let a = t3 - t2 + 2. * t - 1.;
                    let b = t3 - t2 + 1.;
                    let c = 1. - t3 + 3. * t2 - 2. * t;

                    let u = t2 - t;
                    let u2 = u.powi(2);

                    let mu = a * b * c / (u * u2);
                    let dmu =
                        -((t2 - t + 1.) / u2).powi(2) * horner!(t, 3., -8., 2., 6., 7., -10., 3.);
                    (mu, dmu)
                };
                let bounds = Bounds {
                    min_x: -2.5,
                    max_x: 3.5,
                    min_y: -3.,
                    max_y: 3.,
                };
                CoveringMap::new(self, param_map)
                    .with_orig_bounds(bounds)
                    .with_multiplier_map(mult)
            }
            _ => CoveringMap::from(self),
        }
    }
    fn misiurewicz_curve(self, preperiod: Period, period: Period) -> CoveringMap<Self>
    {
        match (preperiod, period) {
            (2, 1) => {
                let param_map = |t: Cplx| {
                    let t2 = t.powi(2);
                    let u = (t2 - 1.).inv();
                    let u2 = u.powi(2);
                    (-2. * (t2 + 1.) * u2, 4. * t * (t2 + 3.) * u2 * u)
                };
                let bounds = Bounds {
                    min_x: -3.5,
                    max_x: 3.5,
                    min_y: -3.0,
                    max_y: 3.0,
                };
                CoveringMap::new(self, param_map).with_orig_bounds(bounds)
            }
            (2, 2) => {
                let param_map = |c: Cplx| {
                    let c2 = c.powi(2);
                    (
                        -0.25 * (c2 * (c2 + 2. * c + 2.) - 2. * c + 1.) / c2,
                        -0.5 * (c2 + c - 1.) * (c2 + 1.) / (c2 * c),
                    )
                };
                let bounds = Bounds {
                    min_x: -4.,
                    max_x: 2.4,
                    min_y: -2.5,
                    max_y: 2.5,
                };
                CoveringMap::new(self, param_map).with_orig_bounds(bounds)
            }
            (_, _) => CoveringMap::from(self),
        }
    }
}

impl MarkedPoints for Mandelbrot
{
    #[inline]
    fn critical_points_child(&self, _param: &Cplx) -> ComplexVec
    {
        vec![Cplx::new(0., 0.)]
    }

    fn cycles(&self, period: Period) -> Vec<Self::Var>
    {
        match period {
            1 => vec![ZERO],
            2 => vec![-ONE],
            3 => solve_cubic(ONE, ONE, TWO).to_vec(),
            4 => {
                const COEFFS: [Cplx; 6] = cplx_arr!([1, 2, 3, 3, 3, 1]);
                solve_polynomial(COEFFS)
            }
            5 => {
                const COEFFS: [Cplx; 16] =
                    cplx_arr!([1, 1, 2, 5, 14, 26, 44, 69, 94, 114, 116, 94, 60, 28, 8, 1]);
                solve_polynomial(COEFFS)
            }
            _ => vec![],
        }
    }

    fn cycles_child(&self, c: &Cplx, period: Period) -> ComplexVec
    {
        use dynamo_common::math_utils::polynomial_roots::solve_polynomial;
        match period {
            1 => {
                let u = (1. - 4. * c).sqrt();
                vec![0.5 * (1. + u), 0.5 * (1. - u)]
            }
            2 => {
                let u = (-3. - 4. * c).sqrt();
                vec![0.5 * (-1. + u), -0.5 * (1. + u)]
            }
            3 => {
                let c2 = c * c;
                let coeffs = vec![
                    1. + c + (2. + c) * c2,
                    1. + c + c + c2,
                    1. + 3. * (c + c2),
                    1. + c + c,
                    1. + 3. * c,
                    ONE,
                    ONE,
                ];
                solve_polynomial(coeffs)
            }
            4 => {
                let c2 = c * c;
                let coeffs = vec![
                    1. + c2 * horner_monic!(c, 2., 3., 3., 3.),
                    c * horner_monic!(c, 2., 1., 2.),
                    c * horner!(c, 1., 5., 6., 12., 6.),
                    1. + 4. * c2 * (1. + c),
                    c * horner!(c, 4., 3., 18., 15.),
                    c * horner!(c, 2., 6.),
                    1. + c2 * (12. + 20. * c),
                    4. * c,
                    3. * c + 15. * c2,
                    ONE,
                    6. * c,
                    ZERO,
                    ONE,
                ];
                solve_polynomial(coeffs)
            }
            5 => {
                let v = horner_monic!(
                    c, 1., 2., 5., 14., 26., 44., 69., 94., 114., 116., 94., 60., 28., 8.
                );
                let u = 14. * c + 1.;
                let coeffs = [
                    v * c + 1.,
                    v,
                    horner!(
                        c, 1., 3., 9., 28., 66., 137., 265., 436., 642., 794., 766., 576., 316.,
                        105., 15.
                    ),
                    horner!(
                        c, 1., 4., 14., 40., 93., 196., 342., 528., 678., 672., 516., 288., 97.,
                        14.
                    ),
                    horner!(
                        c, 1., 5., 20., 67., 179., 437., 876., 1572., 2398., 2790., 2496., 1629.,
                        637., 105.
                    ),
                    horner!(
                        c, 1., 6., 27., 86., 241., 534., 1044., 1720., 2118., 1980., 1341., 540.,
                        91.
                    ),
                    horner!(
                        c, 1., 7., 35., 126., 401., 1000., 2196., 4200., 5990., 6445., 5071.,
                        2366., 455.
                    ),
                    horner!(
                        c, 1., 8., 40., 160., 466., 1152., 2480., 3872., 4465., 3730., 1826., 364.
                    ),
                    horner!(
                        c, 1., 9., 50., 221., 712., 1932., 4712., 8415., 11025., 10615., 6006.,
                        1365.
                    ),
                    horner!(c, 1., 10., 61., 246., 780., 2232., 4543., 6560., 6885., 4180., 1001.),
                    horner!(
                        c, 1., 11., 73., 324., 1116., 3527., 8113., 13140., 15741., 11011., 3003.
                    ),
                    horner!(c, 1., 12., 78., 336., 1295., 3570., 6580., 8856., 6831., 2002.),
                    horner!(c, 1., 13., 92., 427., 1779., 5467., 11172., 16962., 15015., 5005.),
                    horner!(c, 1., 14., 91., 484., 1897., 4592., 8106., 8184., 3003.),
                    horner!(c, 1., 15., 105., 598., 2565., 6822., 13398., 15444., 6435.),
                    horner!(c, 1., 14., 114., 668., 2230., 5292., 7260., 3432.),
                    horner!(c, 1., 15., 130., 815., 2970., 7722., 12012., 6435.),
                    horner!(c, 1., 16., 147., 740., 2430., 4752., 3003.),
                    horner!(c, 1., 17., 165., 900., 3190., 7007., 5005.),
                    horner!(c, 1., 18., 160., 760., 2255., 2002.),
                    horner!(c, 1., 19., 180., 913., 3003., 3003.),
                    horner!(c, 1., 20., 153., 748., 1001.),
                    horner!(c, 1., 21., 171., 910., 1365.),
                    horner!(c, 1., 18., 162., 364.),
                    horner!(c, 1., 19., 182., 455.),
                    horner!(c, 1., 20., 91.),
                    horner!(c, 1., 21., 105.),
                    u,
                    u + c,
                    ONE,
                    ONE,
                ];
                solve_polynomial(coeffs)
            }
            6 => {
                let c2 = c * c;
                let coeffs = [
                    horner_monic!(
                        c, 1., -1., 1., 3., 7., 17., 35., 76., 155., 298., 536., 927., 1525.,
                        2331., 3310., 4346., 5258., 5843., 5892., 5313., 4219., 2892., 1672., 792.,
                        293., 78., 13.
                    ),
                    -horner_monic!(
                        c, 1., -2., -1., -2., -1., -8., -16., -18., -26., -32., -31., 16., 149.,
                        384., 730., 1164., 1635., 2032., 2201., 2050., 1614., 1052., 554., 226.,
                        66., 12.
                    ),
                    c * horner!(
                        c, -1., 1., 2., 20., 38., 118., 336., 830., 1789., 3675., 7278., 13177.,
                        21870., 33146., 45768., 57763., 65879., 66831., 59408., 45336., 29012.,
                        15146., 6156., 1793., 325., 27.
                    ),
                    horner!(
                        c, 1., 0., 4., -4., 8., 50., 62., 112., 181., 342., 426., 12., -1202.,
                        -3608., -7396., -12632., -18711., -23728., -25494., -22864., -16778.,
                        -9862., -4466., -1440., -287., -26.
                    ),
                    horner!(
                        c, -1., 2., -3., 26., 33., 84., 364., 1166., 2932., 6726., 15798., 33828.,
                        65816., 115_718., 182_688., 262_556., 340_280., 390_387., 390_678.,
                        333_800., 237_612., 137_324., 61590., 19710., 3900., 351.
                    ),
                    c * horner!(
                        c, 2., -2., -10., 62., 90., 190., 296., 764., 1772., 1900., -148., -6726.,
                        -19956., -42692., -77016., -116_249., -145_938., -150_666., -125_470.,
                        -82824., -41866., -14982., -3288., -325.
                    ),
                    horner!(
                        c, 1., -4., 14., 28., 6., 202., 966., 3128., 7588., 20584., 52104.,
                        119_292., 245_646., 444_356., 728_148., 1_077_208., 1_405_041., 1_592_556.,
                        1_533_190., 1_220_740., 784_784., 390_258., 137_862., 29900., 2925.
                    ),
                    c * horner!(
                        c, 2., -14., 36., 80., 212., 328., 672., 3104., 5500., 5416., -3512.,
                        -28760., -81704., -185_472., -340_960., -508_344., -612_114., -584_050.,
                        -435_980., -247_500., -98868., -24012., -2600.
                    ),
                    horner!(
                        c, -1., 3., 21., -17., 63., 463., 2390., 6152., 18021., 53705., 144_586.,
                        354_856., 738_994., 1_379_504., 2_338_720., 3_486_132., 4_503_660.,
                        4_920_750., 4_409_715., 3_170_035., 1_756_986., 688_666., 164_450., 17550.
                    ),
                    horner!(
                        c,
                        1.,
                        -6.,
                        7.,
                        46.,
                        131.,
                        368.,
                        76.,
                        2940.,
                        8045.,
                        13180.,
                        8944.,
                        -20888.,
                        -94332.,
                        -285_488.,
                        -662_360.,
                        -1_198_620.,
                        -1_712_196.,
                        -1_894_170.,
                        -1_611_725.,
                        -1_034_550.,
                        -464_310.,
                        -125_488.,
                        -14950.
                    ),
                    c * horner!(
                        c,
                        7.,
                        -7.,
                        2.,
                        97.,
                        1220.,
                        4040.,
                        11356.,
                        39435.,
                        122_452.,
                        368_668.,
                        894_960.,
                        1_895_936.,
                        3_694_012.,
                        6_322_848.,
                        9_368_184.,
                        11_712_477.,
                        11_898_351.,
                        9_619_643.,
                        5_976_432.,
                        2_613_996.,
                        690_690.,
                        80730.
                    ),
                    horner!(
                        c,
                        -1.,
                        0.,
                        16.,
                        36.,
                        348.,
                        -268.,
                        1424.,
                        7344.,
                        16828.,
                        23760.,
                        592.,
                        -61208.,
                        -283_360.,
                        -886_396.,
                        -2_009_148.,
                        -3_484_176.,
                        -4_538_121.,
                        -4_442_796.,
                        -3_249_114.,
                        -1_650_264.,
                        -499_422.,
                        -65780.
                    ),
                    horner!(
                        c,
                        1.,
                        0.,
                        -6.,
                        10.,
                        308.,
                        2266.,
                        5484.,
                        21820.,
                        73304.,
                        281_624.,
                        821_580.,
                        1_961_932.,
                        4_397_120.,
                        8_677_956.,
                        14_827_548.,
                        21_411_444.,
                        24_864_744.,
                        22_751_949.,
                        15_943_774.,
                        7_831_362.,
                        2_302_300.,
                        296_010.
                    ),
                    c * horner!(
                        c,
                        2.,
                        2.,
                        188.,
                        -142.,
                        12.,
                        4540.,
                        13504.,
                        30788.,
                        19872.,
                        -10472.,
                        -162_932.,
                        -814_772.,
                        -2_436_252.,
                        -5_304_348.,
                        -8_306_280.,
                        -9_462_183.,
                        -7_951_614.,
                        -4_608_450.,
                        -1_572_648.,
                        -230_230.
                    ),
                    c * horner!(
                        c,
                        -2.,
                        6.,
                        -36.,
                        998.,
                        2204.,
                        9676.,
                        30960.,
                        156_468.,
                        586_996.,
                        1_569_376.,
                        4_035_152.,
                        9_213_996.,
                        18_219_084.,
                        30_707_820.,
                        41_178_216.,
                        42_945_111.,
                        34_177_770.,
                        18_987_650.,
                        6_249_100.,
                        888_030.
                    ),
                    c2 * horner!(
                        c,
                        48.,
                        40.,
                        -456.,
                        2016.,
                        6928.,
                        26416.,
                        27624.,
                        17396.,
                        -17732.,
                        -480_040.,
                        -2_112_240.,
                        -6_098_352.,
                        -11_838_720.,
                        -15_916_896.,
                        -15_519_504.,
                        -10_360_548.,
                        -4_018_652.,
                        -657_800.
                    ),
                    c * horner!(
                        c,
                        2.,
                        -38.,
                        282.,
                        826.,
                        3525.,
                        9658.,
                        59106.,
                        330_417.,
                        991_776.,
                        2_898_731.,
                        7_687_317.,
                        17_576_754.,
                        34_973_940.,
                        54_838_260.,
                        65_702_076.,
                        59_828_967.,
                        37_898_388.,
                        14_060_475.,
                        2_220_075.
                    ),
                    c * horner!(
                        c,
                        4.,
                        52.,
                        -302.,
                        667.,
                        2132.,
                        15620.,
                        25434.,
                        16512.,
                        56936.,
                        -125_345.,
                        -1_228_656.,
                        -5_239_728.,
                        -13_253_280.,
                        -21_432_240.,
                        -24_515_700.,
                        -19_058_292.,
                        -8_479_548.,
                        -1_562_275.
                    ),
                    c * horner!(
                        c,
                        -6.,
                        28.,
                        302.,
                        955.,
                        2930.,
                        11652.,
                        145_028.,
                        505_753.,
                        1_646_205.,
                        5_107_179.,
                        13_387_088.,
                        31_794_360.,
                        59_241_260.,
                        82_287_480.,
                        86_437_384.,
                        63_005_349.,
                        26_558_675.,
                        4_686_825.
                    ),
                    c * horner!(
                        c,
                        14.,
                        -88.,
                        148.,
                        340.,
                        6058.,
                        17970.,
                        7176.,
                        54549.,
                        58630.,
                        -366_080.,
                        -3_212_352.,
                        -11_638_120.,
                        -23_264_500.,
                        -31_621_700.,
                        -28_992_480.,
                        -14_954_577.,
                        -3_124_550.
                    ),
                    c * horner!(
                        c,
                        -5.,
                        92.,
                        136.,
                        1180.,
                        -1198.,
                        47134.,
                        212_514.,
                        742_148.,
                        2_740_947.,
                        8_069_776.,
                        23_034_804.,
                        52_124_176.,
                        84_859_852.,
                        103_733_388.,
                        87_929_644.,
                        42_493_880.,
                        8_436_285.
                    ),
                    horner!(
                        c,
                        1.,
                        -10.,
                        6.,
                        46.,
                        1274.,
                        9578.,
                        2598.,
                        25256.,
                        79871.,
                        75504.,
                        -1_189_760.,
                        -7_897_396.,
                        -20_394_764.,
                        -33_440_836.,
                        -36_709_596.,
                        -22_227_568.,
                        -5_311_735.
                    ),
                    horner!(
                        c,
                        -1.,
                        16.,
                        -6.,
                        440.,
                        -1474.,
                        9726.,
                        74522.,
                        264_788.,
                        1_207_569.,
                        3_868_332.,
                        13_175_656.,
                        37_340_160.,
                        72_239_596.,
                        103_725_636.,
                        103_500_828.,
                        57_946_200.,
                        13_037_895.
                    ),
                    c * horner!(
                        c,
                        -6.,
                        30.,
                        -4.,
                        3636.,
                        1756.,
                        5404.,
                        43240.,
                        136_968.,
                        -9204.,
                        -3_967_652.,
                        -14_389_752.,
                        -28_989_896.,
                        -38_798_760.,
                        -27_992_472.,
                        -7_726_160.
                    ),
                    horner!(
                        c,
                        1.,
                        -3.,
                        99.,
                        -375.,
                        488.,
                        21490.,
                        74074.,
                        441_546.,
                        1_502_124.,
                        5_827_640.,
                        21_663_746.,
                        50_749_114.,
                        86_415_420.,
                        102_965_940.,
                        67_603_900.,
                        17_383_860.
                    ),
                    horner!(
                        c,
                        -1.,
                        10.,
                        -75.,
                        922.,
                        1198.,
                        -616.,
                        14882.,
                        63804.,
                        311_844.,
                        -1_290_380.,
                        -8_091_694.,
                        -20_507_916.,
                        -34_213_452.,
                        -29_953_728.,
                        -9_657_700.
                    ),
                    c * horner!(
                        c,
                        9.,
                        -23.,
                        -384.,
                        4788.,
                        16198.,
                        132_426.,
                        497_400.,
                        1_912_410.,
                        10_054_650.,
                        29_340_674.,
                        59_798_928.,
                        86_532_992.,
                        67_603_900.,
                        20_058_300.
                    ),
                    horner!(
                        c,
                        1.,
                        -16.,
                        142.,
                        496.,
                        -742.,
                        3640.,
                        11340.,
                        218_736.,
                        -92910.,
                        -3_555_040.,
                        -11_724_900.,
                        -25_069_968.,
                        -27_249_572.,
                        -10_400_600.
                    ),
                    c * horner!(
                        c,
                        7.,
                        -126.,
                        734.,
                        2860.,
                        30822.,
                        152_436.,
                        425_820.,
                        3_641_400.,
                        13_892_230.,
                        34_151_436.,
                        61_244_676.,
                        57_946_200.,
                        20_058_300.
                    ),
                    horner!(
                        c,
                        -1.,
                        10.,
                        118.,
                        -188.,
                        590.,
                        -1860.,
                        81084.,
                        180_120.,
                        -1_170_450.,
                        -5_329_500.,
                        -15_135_780.,
                        -21_038_928.,
                        -9_657_700.
                    ),
                    horner!(
                        c,
                        1.,
                        -18.,
                        66.,
                        428.,
                        4962.,
                        44436.,
                        49620.,
                        974_712.,
                        5_350_818.,
                        15_931_652.,
                        36_283_236.,
                        42_493_880.,
                        17_383_860.
                    ),
                    c * horner!(
                        c,
                        16.,
                        -16.,
                        16.,
                        -1248.,
                        15552.,
                        126_288.,
                        -257_856.,
                        -1_875_984.,
                        -7_418_664.,
                        -13_728_792.,
                        -7_726_160.
                    ),
                    horner!(
                        c,
                        -1.,
                        3.,
                        47.,
                        444.,
                        11067.,
                        -576.,
                        165_036.,
                        1_662_804.,
                        5_979_699.,
                        17_814_742.,
                        26_558_675.,
                        13_037_895.
                    ),
                    horner!(
                        c,
                        1.,
                        0.,
                        -16.,
                        -174.,
                        30.,
                        47724.,
                        -19278.,
                        -490_314.,
                        -2_877_930.,
                        -7_518_148.,
                        -5_311_735.
                    ),
                    c * horner!(
                        c,
                        2.,
                        12.,
                        1968.,
                        120.,
                        4392.,
                        412_566.,
                        1_768_140.,
                        7_138_395.,
                        14_060_475.,
                        8_436_285.
                    ),
                    c * horner!(
                        c,
                        -2.,
                        6.,
                        -720.,
                        11370.,
                        11754.,
                        -89034.,
                        -842_688.,
                        -3_417_777.,
                        -3_124_550.
                    ),
                    c2 * horner!(
                        c, 202., 660., -6620., 80256., 400_862., 2_278_518., 6_249_100., 4_686_825.
                    ),
                    c * horner!(
                        c,
                        2.,
                        -150.,
                        1570.,
                        6220.,
                        -10450.,
                        -166_782.,
                        -1_269_048.,
                        -1_562_275.
                    ),
                    c * horner!(
                        c, 8., 210., -2020., 11660., 67914., 556_094., 2_302_300., 2_220_075.
                    ),
                    c * horner!(c, -10., 70., 1700., -1100., -13860., -375_452., -657_800.),
                    c * horner!(c, 26., -285., 1067., 8778., 95634., 690_690., 888_030.),
                    c * horner!(c, -10., 291., -286., 3234., -85008., -230_230.),
                    horner!(c, 1., -17., 11., 1056., 9108., 164_450., 296_010.),
                    horner!(c, -1., 28., -66., 1320., -13662., -65780.),
                    c * horner!(c, -11., 150., -230., 29900., 80730.),
                    horner!(c, 1., -6., 198., -1288., -14950.),
                    horner!(c, -1., 18., -198., 3900., 17550.),
                    c * horner!(c, 12., -12., -2600.),
                    horner!(c, 1., -24., 325., 2925.),
                    c * horner!(c, 12., -325.),
                    horner!(c, -1., 13., 351.),
                    1. - 26. * c,
                    c * 27.,
                    -ONE,
                    ONE,
                ];
                solve_polynomial(coeffs)
            }
            _ => vec![],
        }
    }

    fn precycles_child(&self, c: &Cplx, orbit_schema: OrbitSchema) -> ComplexVec
    {
        use dynamo_common::math_utils::polynomial_roots::solve_polynomial;
        match (orbit_schema.preperiod, orbit_schema.period) {
            (2, 1) => {
                let u = 0.5 * (1. - 4. * c).sqrt();
                let v0 = (-0.5 + u - c).sqrt();
                let v1 = (-0.5 - u - c).sqrt();
                vec![v0, -v0, v1, -v1]
            }
            (2, 2) => {
                let u = 0.5 * (-3. - 4. * c).sqrt();
                let v0 = (0.5 + u - c).sqrt();
                let v1 = (0.5 - u - c).sqrt();
                vec![v0, -v0, v1, -v1]
            }
            (2, 3) => {
                let coeffs = [
                    horner_monic!(c, 1., 0., 1., 2., 2., 2.),
                    horner!(c, -1., 0., 2., 4., 7., 6.),
                    horner!(c, 1., 0., 3., 8., 15.),
                    horner!(c, -1., 2., 2., 20.),
                    horner!(c, 1., -2., 15.),
                    horner!(c, -1., 6.),
                    ONE,
                ];
                let zs = solve_polynomial(coeffs);
                zs.iter().map(|z| z.sqrt()).flat_map(|w| [w, -w]).collect()
            }
            (2, 4) => {
                let coeffs = [
                    horner_monic!(c, 1., 0., 0., 2., 6., 8., 11., 18., 23., 22., 15., 6.),
                    c * horner!(c, -2., -2., 8., 15., 20., 54., 104., 135., 120., 60., 12.),
                    c * horner!(c, -2., 5., 18., 13., 54., 186., 348., 420., 270., 66.),
                    horner!(c, -1., 0., 12., 8., 12., 160., 484., 840., 720., 220.),
                    c * horner!(c, 4., 8., -12., 55., 384., 1050., 1260., 495.),
                    c * horner!(c, 4., -6., -12., 162., 840., 1512., 792.),
                    horner!(c, 1., 0., -16., 20., 420., 1260., 924.),
                    c * horner!(c, -4., -12., 120., 720., 792.),
                    c * horner!(c, -6., 15., 270., 495.),
                    horner!(c, -1., 0., 60., 220.),
                    c * horner!(c, 6., 66.),
                    c * 12.,
                    ONE,
                ];
                let zs = solve_polynomial(coeffs);
                zs.iter().map(|z| z.sqrt()).flat_map(|w| [w, -w]).collect()
            }
            (3, 3) => {
                let coeffs = [
                    horner!(c, 1., 0., 0., 2., 5., 6., 10., 16., 18., 18., 14., 6., 1.),
                    c * horner!(c, -2., 0., 8., 8., 20., 56., 80., 104., 110., 60., 12.),
                    horner!(c, -1., 0., 8., 4., 10., 84., 148., 244., 375., 270., 66.),
                    c * horner!(c, 4., 0., -8., 72., 156., 288., 720., 720., 220.),
                    horner!(c, 1., 0., -12., 38., 115., 160., 840., 1260., 495.),
                    c * horner!(c, -6., 12., 68., 8., 588., 1512., 792.),
                    horner!(c, -1., 2., 30., -36., 210., 1260., 924.),
                    c * horner!(c, 8., -16., 0., 720., 792.),
                    horner!(c, 1., -2., -30., 270., 495.),
                    c * horner!(c, -10., 60., 220.),
                    horner!(c, -1., 6., 66.),
                    c * 12.,
                    ONE,
                ];
                let zs = solve_polynomial(coeffs);
                zs.iter().map(|z| z.sqrt()).flat_map(|w| [w, -w]).collect()
            }
            (3, 4) => {
                let c2 = c * c;
                let coeffs = [
                    horner!(
                        c, 1., 0., 0., 0., 4., 14., 30., 56., 102., 192., 356., 626., 1015., 1494.,
                        1982., 2336., 2415., 2166., 1658., 1062., 555., 226., 66., 12., 1.
                    ),
                    c2 * horner!(
                        c, -4., -4., 8., 44., 112., 232., 504., 1150., 2536., 5096., 9096., 14412.,
                        19984., 23952., 24624., 21390., 15408., 9000., 4080., 1320., 264., 24.
                    ),
                    c * horner!(
                        c, -2., -2., -4., 30., 128., 312., 660., 1655., 4452., 11176., 24444.,
                        46830., 77336., 108_528., 129_000., 128_151., 104_472., 68580., 34800.,
                        12540., 2772., 276.
                    ),
                    c2 * horner!(
                        c, -8., 0., 72., 292., 600., 1432., 4400., 13940., 37944., 89384.,
                        178_704., 297_192., 412_560., 472_944., 439_488., 326_160., 186_360.,
                        75240., 18480., 2024.
                    ),
                    c * horner!(
                        c, -2., -10., 18., 193., 454., 926., 2748., 10765., 37230., 110_466.,
                        273_796., 548_058., 899_220., 1_202_340., 1_284_192., 1_084_500., 702_270.,
                        319_770., 87780., 10626.
                    ),
                    c * horner!(
                        c, -6., 0., 72., 272., 520., 1296., 5264., 23520., 91272., 291_392.,
                        717_360., 1_411_536., 2_230_200., 2_764_944., 2_676_240., 1_977_984.,
                        1_023_264., 316_008., 42504.
                    ),
                    horner!(
                        c, -1., 0., 12., 120., 236., 664., 1688., 9072., 49356., 218_368.,
                        683_424., 1_642_872., 3_117_492., 4_540_536., 5_077_800., 4_316_640.,
                        2_558_160., 895_356., 134_596.
                    ),
                    c2 * horner!(
                        c, 32., 64., 384., 512., 1632., 15168., 112_992., 476_856., 1_436_160.,
                        3_342_624., 5_806_944., 7_572_240., 7_466_400., 5_116_320., 2_046_528.,
                        346_104.
                    ),
                    c * horner!(
                        c,
                        4.,
                        8.,
                        168.,
                        280.,
                        -156.,
                        420.,
                        36948.,
                        240_399.,
                        942_480.,
                        2_768_832.,
                        5_853_276.,
                        8_996_130.,
                        10_382_580.,
                        8_314_020.,
                        3_837_240.,
                        735_471.
                    ),
                    c2 * horner!(
                        c,
                        40.,
                        160.,
                        -120.,
                        -1900.,
                        4792.,
                        83496.,
                        456_720.,
                        1_767_700.,
                        4_674_384.,
                        8_580_000.,
                        11_704_160.,
                        11_085_360.,
                        5_969_040.,
                        1_307_504.
                    ),
                    c * horner!(
                        c,
                        4.,
                        60.,
                        -12.,
                        -894.,
                        -1844.,
                        17220.,
                        156_024.,
                        858_066.,
                        2_954_952.,
                        6_589_440.,
                        10_735_296.,
                        12_193_896.,
                        7_759_752.,
                        1_961_256.
                    ),
                    c * horner!(
                        c,
                        12.,
                        0.,
                        -192.,
                        -1216.,
                        672.,
                        32592.,
                        306_864.,
                        1_467_648.,
                        4_071_600.,
                        8_009_040.,
                        11_085_360.,
                        8_465_184.,
                        2_496_144.
                    ),
                    horner!(
                        c, 1., 0., -16., -344., -672., 1260., 75180., 563_472., 2_014_740.,
                        4_839_900., 8_314_020., 7_759_752., 2_704_156.
                    ),
                    c2 * horner!(
                        c, 0., -56., -168., -1680., 9912., 162_288., 791_280., 2_350_080.,
                        5_116_320., 5_969_040., 2_496_144.
                    ),
                    c * horner!(
                        c, -4., -12., -600., -540., 33192., 243_000., 905_760., 2_558_160.,
                        3_837_240., 1_961_256.
                    ),
                    c2 * horner!(
                        c, -96., -576., 4320., 56880., 272_544., 1_023_264., 2_046_528., 1_307_504.
                    ),
                    c * horner!(c, -6., -138., 270., 9675., 62730., 319_770., 895_356., 735_471.),
                    c * horner!(c, -18., 0., 1080., 10800., 75240., 316_008., 346_104.),
                    horner!(c, -1., 0., 60., 1360., 12540., 87780., 134_596.),
                    c2 * horner!(c, 120., 1320., 18480., 42504.),
                    c * horner!(c, 6., 66., 2772., 10626.),
                    c2 * horner!(c, 264., 2024.),
                    c * horner!(c, 12., 276.),
                    c * 24.,
                    ONE,
                ];
                let zs = solve_polynomial(coeffs);
                zs.iter().map(|z| z.sqrt()).flat_map(|w| [w, -w]).collect()
            }
            _ => vec![],
        }
    }
}

degree_impl!(Mandelbrot, 2);

#[derive(Clone, Debug)]
pub struct MandelbrotMC3Mult
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl MandelbrotMC3Mult
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -1.55,
        max_x: 0.55,
        min_y: -1.75,
        max_y: 1.75,
    };
}
impl Default for MandelbrotMC3Mult
{
    fractal_impl!();
}

impl DynamicalFamily for MandelbrotMC3Mult
{
    type Var = Cplx;
    type Param = CplxPair;
    type MetaParam = NoParam;
    type Deriv = Cplx;
    basic_plane_impl!();
    default_name!();

    fn escape_radius(&self) -> Real
    {
        1e26
    }

    fn start_point(&self, _point: Cplx, _c: &Self::Param) -> Self::Var
    {
        ZERO
    }

    fn param_map(&self, t: Cplx) -> Self::Param
    {
        CplxPair {
            a: -t.powi(2) - t - 2.,
            b: horner_monic!(t, 1., 2., 1.),
        }
    }

    #[inline]
    fn map(&self, z: Self::Var, CplxPair { a, b: _ }: &Self::Param) -> Self::Var
    {
        z.powi(2) + a
    }

    #[inline]
    fn map_and_multiplier(
        &self,
        z: Self::Var,
        CplxPair { a, b: _ }: &Self::Param,
    ) -> (Self::Var, Self::Deriv)
    {
        (z.powi(2) + a, 2. * z)
    }

    fn early_bailout(
        &self,
        _start: Cplx,
        CplxPair { a, b }: &Self::Param,
    ) -> Option<EscapeResult<Cplx, Cplx>>
    {
        Some(EscapeResult::Periodic {
            info: PointInfoPeriodic {
                preperiod: 0,
                period: 3,
                multiplier: *b,
                final_error: 0.0,
            },
            final_value: *a,
        })
    }
}

impl FamilyDefaults for MandelbrotMC3Mult
{
    default_bounds!();
}

impl HasJulia for MandelbrotMC3Mult
{
    #[inline]
    fn default_bounds_child(&self, _point: Cplx, _param: &Self::Param) -> Bounds
    {
        Bounds::centered_square(2.2)
    }
}

impl MarkedPoints for MandelbrotMC3Mult {}

degree_impl!(MandelbrotMC3Mult, 2);
