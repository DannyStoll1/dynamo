use crate::macros::{cplx_arr, degree_impl, has_child_impl, horner, horner_monic, profile_imports};
use seq_macro::seq;
profile_imports!();

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct QuadRatMerge2_2
{
    point_grid: PointGrid,
    compute_mode: ComputeMode,
    max_iter: IterCount,
}

impl QuadRatMerge2_2
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -2.8,
        max_x: 3.2,
        min_y: -2.8,
        max_y: 2.8,
    };
}
impl Default for QuadRatMerge2_2
{
    fractal_impl!();
}

type Prm = param::Param;

impl DynamicalFamily for QuadRatMerge2_2
{
    type Var = Cplx;
    type Param = Prm;
    type Deriv = Cplx;
    type MetaParam = NoParam;
    basic_plane_impl!();
    default_name!();

    fn description(&self) -> String
    {
        "The moduli space of quadratic rational maps with a critical 2-cycle, \
            parameterized as $f_c(z) = (z^2 + c)/(z^2 - 1)$. In these coordinates, \
            ∞ <-> 1 is the critical 2-cycle. The plane is colored according to the \
            activity of the free critical point 0."
            .to_owned()
    }

    #[inline]
    fn map(&self, z: Self::Var, Prm { a: _, c }: &Self::Param) -> Self::Var
    {
        let z2 = z.powi(2);
        (z2 + c) / (1. - z2)
    }

    #[inline]
    fn map_and_multiplier(
        &self,
        z: Self::Var,
        Prm { a, c }: &Self::Param,
    ) -> (Self::Var, Self::Deriv)
    {
        let z2 = z.powi(2);
        let u = z2 - c;
        ((z2 + c) / u, -4. * c * z / u.powi(2))
    }

    #[inline]
    fn gradient(
        &self,
        z: Self::Var,
        Prm { a, c }: &Self::Param,
    ) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        let z2 = z.powi(2);
        let u = z2 - c;
        let u2 = u.powi(2);
        ((z2 + c) / u, -4. * c * z / u2, 2. * z2 / u2)
    }

    #[inline]
    fn start_point(&self, _point: Cplx, _c: &Self::Param) -> Self::Var
    {
        ZERO
    }

    #[inline]
    fn start_point_d(&self, _point: Cplx, _c: &Self::Param)
        -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        (ZERO, ZERO, ZERO)
    }

    #[inline]
    fn param_map(&self, point: Cplx) -> Self::Param
    {
        Self::Param::from(point)
    }

    #[inline]
    fn param_map_d(&self, point: Cplx) -> (Self::Param, Self::Deriv)
    {
        (Self::Param::from(point), ONE)
    }
}

default_bounds_impl!(QuadRatMerge2_2);
has_child_impl!(QuadRatMerge2_2, 4.0);

fn cycles(c: Cplx, period: Period) -> Vec<Cplx>
{
    match period {
        1 => {
            let u = -27. * c;
            let v = u - 11.;
            let x0 = (0.5 * (u + (v.powi(2) - 256.).sqrt() - 11.)).powf(ONE_THIRD);
            let x1 = 4. / x0 * ONE_THIRD;
            let x2 = x0 * ONE_THIRD;
            let r1 = x1 * OMEGA_BAR + x2 * OMEGA - ONE_THIRD;
            let r2 = x1 * OMEGA + x2 * OMEGA_BAR - ONE_THIRD;
            vec![x1 + x2 - ONE_THIRD, r1, r2]
        }
        2 => {
            vec![(-1.).into()]
        }
        3 => {
            let coeffs = [
                horner_monic!(c, 1., -1.),
                c + 1.,
                3. * c - 2.,
                -c - 1.,
                ONE,
                ZERO,
                ONE,
            ];
            solve_polynomial(coeffs)
        }
        4 => {
            let coeffs = [
                horner_monic!(c, 1., -4., 6., -3.),
                c * horner_monic!(c, 2., -3.),
                horner!(c, -6., 19., -20., 7.),
                -horner_monic!(c, -1., 11., -9.),
                horner!(c, 12., -28., 19.),
                -horner!(c, 4., -18., 6.),
                horner_monic!(c, -7., 10.),
                horner!(c, 4., -8.),
                horner!(c, -4., 7.),
                1. - c,
                Cplx::new(4., 0.),
                -TWO,
                ONE,
            ];
            solve_polynomial(coeffs)
        }
        5 => {
            let coeffs = [
                horner_monic!(c, 1., -9., 33., -64., 76., -66., 50., -31., 15., -5.),
                horner_monic!(c, 1., -7., 20., -26., 8., 18., -23., 15., -5.),
                horner!(c, -14., 115., -378., 652., -700., 567., -386., 204., -74., 15.),
                -horner_monic!(c, 13., -81., 202., -208., -7., 205., -188., 84., -19.),
                horner!(c, 85., -636., 1862., -2837., 2738., -2019., 1185., -478., 103.),
                -horner!(c, -72., 398., -855., 633., 357., -879., 550., -158., 14.),
                horner_monic!(c, -291., 1979., -5108., 6786., -5793., 3741., -1712., 408.),
                -horner!(c, 219., -1071., 1936., -786., -1457., 1771., -696., 88.),
                horner!(c, 606., -3735., 8345., -9421., 6869., -3538., 954., 17.),
                -horner_monic!(c, -387., 1671., -2431., -49., 2480., -1666., 295.),
                horner!(c, -751., 4182., -7780., 7018., -3921., 1127., 118.),
                -horner!(c, 364., -1396., 1498., 998., -1783., 493., 16.),
                horner_monic!(c, 437., -2221., 2975., -1496., 50., 425.),
                -horner!(c, -73., 315., -228., -556., 153., 93.),
                horner!(c, 126., -417., 1194., -1410., 807., 15.),
                -horner_monic!(c, -199., 409., 22., -334., 253.),
                horner!(c, -375., 1220., -1464., 675., 83.),
                -horner!(c, 176., -276., -243., 221., 12.),
                horner_monic!(c, 185., -431., 162., 206.),
                -horner!(c, -9., 9., 72., 54.),
                horner!(c, 34., -125., 161., 11.),
                -horner_monic!(c, -43., -7., 37.),
                horner!(c, -51., 66., 42.),
                -horner!(c, 8., 18., 10.),
                horner_monic!(c, 5., 13.),
                -horner!(c, 3., 3.),
                horner!(c, 2., 9.),
                -c - 1.,
                ONE,
                ZERO,
                ONE,
            ];
            solve_polynomial(coeffs)
        }
        6 => {
            let coeffs = [
                horner_monic!(
                    c, 1., -19., 162., -822., 2781., -6677., 11858., -16093., 17187., -14858.,
                    10683., -6549., 3486., -1617., 645., -213., 55., -10.
                ),
                -horner!(
                    c, 1., -19., 158., -764., 2409., -5279., 8409., -10129., 9582., -7340., 4625.,
                    -2411., 1041., -363., 99., -19., 2.
                ),
                horner!(
                    c, -27., 485., -3900., 18618., -59129., 132_998., -220_866., 279_744.,
                    -278_190., 223_353., -148_696., 83944., -40647., 16764., -5708., 1512., -280.,
                    28.
                ),
                -horner!(
                    c, -27., 483., -3768., 17042., -50158., 102_498., -152_240., 170_888.,
                    -150_093., 105_861., -60722., 28380., -10696., 3144., -672., 88., -4.
                ),
                horner!(
                    c,
                    337.,
                    -5719.,
                    43334.,
                    -194_405.,
                    578_641.,
                    -1_216_500.,
                    1_882_943.,
                    -2_216_106.,
                    2_040_958.,
                    -1_511_772.,
                    923_504.,
                    -473_398.,
                    203_842.,
                    -71966.,
                    19682.,
                    -3738.,
                    378.
                ),
                -horner!(
                    c,
                    337.,
                    -5671.,
                    41461.,
                    -175_199.,
                    480_666.,
                    -914_206.,
                    1_262_034.,
                    -1_312_870.,
                    1_061_802.,
                    -682_088.,
                    350_924.,
                    -143_806.,
                    45870.,
                    -10758.,
                    1638.,
                    -118.,
                    2.
                ),
                horner!(
                    c,
                    -2575.,
                    41258.,
                    -294_310.,
                    1_239_024.,
                    -3_448_984.,
                    6_756_464.,
                    -9_706_528.,
                    10_558_044.,
                    -8_944_088.,
                    6_057_758.,
                    -3_350_958.,
                    1_526_584.,
                    -563_744.,
                    160_268.,
                    -31448.,
                    3248.
                ),
                -horner!(
                    c,
                    -2576.,
                    40746.,
                    -278_886.,
                    1_099_544.,
                    -2_806_724.,
                    4_954_112.,
                    -6_326_464.,
                    6_054_604.,
                    -4_461_812.,
                    2_573_162.,
                    -1_163_106.,
                    405_280.,
                    -104_168.,
                    17872.,
                    -1632.,
                    48.
                ),
                horner!(
                    c,
                    13433.,
                    -203_103.,
                    1_362_566.,
                    -5_373_426.,
                    13_949_591.,
                    -25_362_021.,
                    33_638_139.,
                    -33_584_651.,
                    25_937_032.,
                    -15_859_612.,
                    7_784_665.,
                    -3_044_429.,
                    907_591.,
                    -185_397.,
                    19717.,
                    7.
                ),
                -horner_monic!(
                    c,
                    13457.,
                    -199_943.,
                    1_279_710.,
                    -4_699_338.,
                    11_131_207.,
                    -18_157_443.,
                    21_309_749.,
                    -18_579_477.,
                    12_302_580.,
                    -6_243_754.,
                    2_407_279.,
                    -680_595.,
                    130_003.,
                    -13943.,
                    543.
                ),
                horner!(
                    c,
                    -50439.,
                    719_365.,
                    -4_533_398.,
                    16_708_778.,
                    -40_307_288.,
                    67_666_978.,
                    -82_294_604.,
                    74_744_256.,
                    -51_971_031.,
                    28_138_639.,
                    -11_855_580.,
                    3_749_752.,
                    -804_114.,
                    88508.,
                    172.
                ),
                -horner!(
                    c,
                    -50703.,
                    707_245.,
                    -4_227_246.,
                    14_424_942.,
                    -31_587_018.,
                    47_337_718.,
                    -50_600_488.,
                    39_660_976.,
                    -23_143_141.,
                    10_041_159.,
                    -3_154_684.,
                    668_984.,
                    -81488.,
                    3760.,
                    24.
                ),
                horner!(
                    c,
                    139_641.,
                    -1_878_295.,
                    11_104_345.,
                    -38_140_597.,
                    85_101_800.,
                    -131_050_880.,
                    144_844_712.,
                    -118_200_524.,
                    72_611_280.,
                    -33_709_464.,
                    11_491_084.,
                    -2_611_776.,
                    297_488.,
                    2004.
                ),
                -horner!(
                    c,
                    141_401.,
                    -1_850_969.,
                    10_314_875.,
                    -32_605_039.,
                    65_657_036.,
                    -89_653_108.,
                    86_188_948.,
                    -59_590_172.,
                    29_772_648.,
                    -10_548_860.,
                    2_494_988.,
                    -338_960.,
                    17184.,
                    288.
                ),
                horner!(
                    c,
                    -285_574.,
                    3_624_260.,
                    -20_073_712.,
                    64_026_500.,
                    -131_353_940.,
                    183_941_448.,
                    -182_486_140.,
                    131_308_964.,
                    -69_029_016.,
                    25_879_256.,
                    -6_300_076.,
                    733_560.,
                    14560.,
                    4.
                ),
                -horner!(
                    c,
                    -293_488.,
                    3_605_312.,
                    -18_695_532.,
                    54_507_452.,
                    -100_184_300.,
                    123_151_932.,
                    -104_501_212.,
                    61_891_956.,
                    -25_246_296.,
                    6_715_228.,
                    -1_004_100.,
                    50544.,
                    2248.
                ),
                horner!(
                    c,
                    419_837.,
                    -5_038_483.,
                    26_130_440.,
                    -77_087_130.,
                    144_302_895.,
                    -181_540_022.,
                    158_593_082.,
                    -97_346_508.,
                    41_069_514.,
                    -10_789_359.,
                    1_218_904.,
                    72924.,
                    101.
                ),
                -horner!(
                    c,
                    445_061.,
                    -5_136_521.,
                    24_733_624.,
                    -66_118_370.,
                    109_681_320.,
                    -119_092_022.,
                    86_455_412.,
                    -41_606_568.,
                    12_566_637.,
                    -2_012_227.,
                    73258.,
                    12452.,
                    6.
                ),
                horner_monic!(
                    c,
                    -402_715.,
                    4_615_085.,
                    -22_501_084.,
                    61_217_820.,
                    -103_519_915.,
                    114_706_902.,
                    -84_967_524.,
                    41_093_432.,
                    -11_426_896.,
                    922_371.,
                    261_348.,
                    1192.
                ),
                -horner!(
                    c,
                    -460_771.,
                    5_024_075.,
                    -22_444_406.,
                    54_542_208.,
                    -80_082_814.,
                    74_036_870.,
                    -42_960_604.,
                    14_624_736.,
                    -2_234_007.,
                    -95285.,
                    50182.,
                    132.
                ),
                horner!(
                    c,
                    152_401.,
                    -1_817_536.,
                    8_759_138.,
                    -22_340_230.,
                    33_670_078.,
                    -30_969_556.,
                    16_390_134.,
                    -3_202_564.,
                    -1_293_752.,
                    668_578.,
                    8670.,
                    20.
                ),
                -horner!(
                    c,
                    248_016.,
                    -2_648_812.,
                    11_106_684.,
                    -24_169_070.,
                    29_710_276.,
                    -20_418_024.,
                    6_485_256.,
                    340_644.,
                    -799_272.,
                    145_702.,
                    1350.
                ),
                horner!(
                    c,
                    199_211.,
                    -1_652_290.,
                    5_825_590.,
                    -11_825_516.,
                    15_524_212.,
                    -14_447_008.,
                    10_070_320.,
                    -4_899_132.,
                    1_162_496.,
                    42170.,
                    210.
                ),
                -horner!(
                    c,
                    92656.,
                    -614_634.,
                    1_818_002.,
                    -3_746_476.,
                    6_291_636.,
                    -7_519_688.,
                    5_441_480.,
                    -2_057_340.,
                    286_284.,
                    8330.,
                    6.
                ),
                horner!(
                    c,
                    -372_527.,
                    3_148_021.,
                    -10_933_094.,
                    20_475_234.,
                    -22_656_438.,
                    15_554_390.,
                    -6_474_468.,
                    1_173_696.,
                    139_859.,
                    1555.
                ),
                -horner!(
                    c,
                    -308_207.,
                    2_407_525.,
                    -7_578_118.,
                    12_872_510.,
                    -13_170_834.,
                    8_205_790.,
                    -2_781_056.,
                    315_800.,
                    34103.,
                    111.
                ),
                horner!(
                    c,
                    241_617.,
                    -1_983_435.,
                    6_358_228.,
                    -10_253_492.,
                    8_717_842.,
                    -3_653_198.,
                    253_936.,
                    307_328.,
                    8285.,
                    5.
                ),
                -horner_monic!(
                    c,
                    253_889.,
                    -1_878_479.,
                    5_253_372.,
                    -7_242_876.,
                    5_215_986.,
                    -1_693_638.,
                    -1384.,
                    91440.,
                    969.
                ),
                horner!(
                    c, 10866., 53830., -332_366., 330_178., 464_512., -876_588., 402_536., 30828.,
                    120.
                ),
                -horner!(
                    c, -52906., 402_450., -916_814., 630_094., 271_396., -499_108., 153_604.,
                    5372., 16.
                ),
                horner!(
                    c,
                    -129_360.,
                    735_268.,
                    -1_597_204.,
                    1_720_788.,
                    -1_055_996.,
                    254_540.,
                    75268.,
                    1100.
                ),
                -horner!(
                    c, -74908., 389_012., -825_772., 977_428., -608_156., 124_924., 17588., 140.
                ),
                horner!(c, 79287., -425_021., 757_246., -468_595., 10509., 99000., 5635., 10.),
                -horner!(c, 66679., -309_305., 471_531., -276_799., 10364., 35028., 964.),
                horner!(c, -1205., 27224., -32900., -66694., 54594., 17432., 130.),
                -horner!(c, -14150., 52290., -20798., -49522., 28956., 3536., 4.),
                horner!(c, -19285., 54161., -32830., -2459., 23599., 854., 1.),
                -horner!(c, -7951., 22373., -24851., 4499., 8236., 108.),
                horner!(c, 7563., -16232., -7724., 11956., 3604., 12.),
                -horner!(c, 4936., -6652., -5112., 6068., 504.),
                horner!(c, -313., 3271., -1381., 4625., 92.),
                -horner!(c, -881., 393., 629., 1611., 12.),
                horner!(c, -216., -2526., 2254., 658.),
                -horner!(c, 210., -1222., 998., 54.),
                horner!(c, 197., 145., 649., 3.),
                -horner!(c, -7., 203., 255., 1.),
                horner!(c, -194., 232., 104.),
                -horner!(c, -92., 88., 4.),
                horner!(c, 27., 58.),
                -horner!(c, 18., 32.),
                horner!(c, 9., 14.),
                Cplx::new(-4., 0.),
                Cplx::new(3., 0.),
                -TWO,
                ONE,
            ];
            solve_polynomial(coeffs)
        }
        _ => vec![],
    }
}

impl MarkedPoints for QuadRatMerge2_2
{
    #[inline]
    fn critical_points_child(&self, Self::Param { a: _, c }: &Self::Param) -> ComplexVec
    {
        let u = ((1. - c) / 2.).sqrt();
        vec![ZERO, u, -u]
        // vec![(0.).into(), ]
    }

    #[allow(clippy::match_same_arms)]
    fn cycles(&self, period: Period) -> Vec<Self::Var>
    {
        match period {
            1 => vec![ZERO],
            2 => vec![],
            3 => solve_quadratic(ONE, -ONE).to_vec(),
            4 => {
                const COEFFS: [Cplx; 5] = cplx_arr!([1, -4, 6, -3, 1]);
                solve_polynomial(COEFFS)
            }
            5 => {
                const COEFFS: [Cplx; 11] = cplx_arr!([1, -9, 33, -64, 76, -66, 50, -31, 15, -5, 1]);
                solve_polynomial(COEFFS)
            }
            6 => {
                const COEFFS: [Cplx; 19] = cplx_arr!([
                    1, -19, 162, -822, 2781, -6677, 11858, -16093, 17187, -14858, 10683, -6549,
                    3486, -1617, 645, -213, 55, -10, 1
                ]);
                solve_polynomial(COEFFS)
            }
            _ => vec![],
        }
    }

    #[inline]
    fn cycles_child(&self, Prm { a: _, c }: &Self::Param, period: Period) -> ComplexVec
    {
        cycles(*c, period)
    }

    fn other_marked_points(&self) -> Vec<Cplx>
    {
        vec![-ONE]
    }
}

impl EscapeEncoding for QuadRatMerge2_2
{
    basic_escape_encoding!(None, 2);
}
impl ExternalRays for QuadRatMerge2_2 {}

mod param
{
    use dynamo_common::prelude::*;

    #[derive(Clone, Copy, PartialEq, Debug)]
    pub struct Param
    {
        pub a: Cplx, // -2c - 2
        pub c: Cplx,
    }

    impl Default for Param
    {
        fn default() -> Self
        {
            Self {
                a: Cplx::new(-2., 0.),
                c: ZERO,
            }
        }
    }

    impl From<Cplx> for Param
    {
        #[inline]
        fn from(c: Cplx) -> Self
        {
            let a = -2. * c - 2.;
            Self { a, c }
        }
    }

    impl From<Param> for Cplx
    {
        #[inline]
        fn from(param: Param) -> Self
        {
            param.c
        }
    }

    impl std::fmt::Display for Param
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
        {
            self.c.fmt(f)
        }
    }

    impl Describe for Param {}
    impl Named for Param
    {
        fn name(&self) -> &str
        {
            "c"
        }
    }
}
