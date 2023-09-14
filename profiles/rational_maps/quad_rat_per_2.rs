use crate::macros::{cplx_arr, horner, horner_monic, profile_imports};
profile_imports!();

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct QuadRatPer2
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl Default for QuadRatPer2
{
    fractal_impl!(-2.8, 3.2, -2.8, 2.8);
}

impl ParameterPlane for QuadRatPer2
{
    parameter_plane_impl!();
    default_name!();

    fn encode_escaping_point(
        &self,
        iters: Period,
        z: Cplx,
        _base_param: Cplx,
    ) -> PointInfo<Self::Var, Self::Deriv>
    {
        if z.is_nan()
        {
            return PointInfo::Escaping {
                potential: f64::from(iters) - 2.,
            };
        }

        let u = self.escape_radius().log2();
        let v = z.norm_sqr().log2();
        // let q = ((base_param - 1.) / (4. * base_param)).norm().log2();
        let q = -1.;
        let residual = ((u + q) / (v + q)).log2();
        // let residual = ((v - 1.) / (u + u - 1.)).log2() + 1.;
        // (F - M) / (2L - M)
        let potential = (residual as IterCount).mul_add(2., f64::from(iters));
        PointInfo::Escaping { potential }
    }

    #[inline]
    fn map(&self, z: Self::Var, c: Self::Param) -> Self::Var
    {
        let z2 = z * z;
        (z2 + c) / (z2 - 1.)
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, c: Self::Param) -> (Self::Var, Self::Deriv)
    {
        let z2 = z * z;
        let u = z2 - 1.;
        ((c + z2) / u, -2.0 * z * (c + 1.) / (u * u))
    }

    #[inline]
    fn dynamical_derivative(&self, z: Self::Var, c: Self::Param) -> Self::Deriv
    {
        let u = 1. / (z * z - 1.);
        -2.0 * (c + 1.) * z * u * u
    }

    #[inline]
    fn parameter_derivative(&self, z: Self::Var, _c: Self::Param) -> Self::Deriv
    {
        1. / (z * z - 1.)
    }

    #[inline]
    fn start_point(&self, _point: Cplx, c: Self::Param) -> Self::Var
    {
        c
    }

    #[inline]
    fn critical_points_child(&self, _param: Cplx) -> ComplexVec
    {
        vec![(0.).into()]
    }

    fn cycles(&self, period: Period) -> Vec<Self::Var>
    {
        match period
        {
            1 => vec![ZERO],
            2 => vec![],
            3 => solve_quadratic(ONE, -ONE).to_vec(),
            4 =>
            {
                const COEFFS: [Cplx; 5] = cplx_arr!([1, -4, 6, -3, 1]);
                solve_polynomial(&COEFFS)
            }
            5 =>
            {
                const COEFFS: [Cplx; 11] = cplx_arr!([1, -9, 33, -64, 76, -66, 50, -31, 15, -5, 1]);
                solve_polynomial(&COEFFS)
            }
            6 =>
            {
                const COEFFS: [Cplx; 19] = cplx_arr!([
                    1, -19, 162, -822, 2781, -6677, 11858, -16093, 17187, -14858, 10683, -6549,
                    3486, -1617, 645, -213, 55, -10, 1
                ]);
                solve_polynomial(&COEFFS)
            }
            _ => vec![],
        }
    }

    fn cycles_child(&self, c: Cplx, period: Period) -> ComplexVec
    {
        match period
        {
            1 =>
            {
                let u = -27. * c;
                let v = u - 11.;
                let x0 = (0.5 * (u + (v * v - 256.).sqrt() - 11.)).powf(ONE_THIRD);
                let x1 = 4. / x0 * ONE_THIRD;
                let x2 = x0 * ONE_THIRD;
                let r1 = -x1 * OMEGA_BAR - x2 * OMEGA + ONE_THIRD;
                let r2 = -x1 * OMEGA - x2 * OMEGA_BAR + ONE_THIRD;
                vec![-x1 - x2 + ONE_THIRD, r1, r2]
            }
            2 =>
            {
                vec![(1.).into()]
            }
            3 =>
            {
                let coeffs = [
                    horner_monic!(c, 1., -1.),
                    -c - 1.,
                    3. * c - 2.,
                    c + 1.,
                    ONE,
                    ZERO,
                    ONE,
                ];
                solve_polynomial(&coeffs)
            }
            4 =>
            {
                let coeffs = [
                    horner_monic!(c, 1., -4., 6., -3.),
                    -c * horner_monic!(c, 2., -3.),
                    horner!(c, -6., 19., -20., 7.),
                    horner_monic!(c, -1., 11., -9.),
                    horner!(c, 12., -28., 19.),
                    horner!(c, 4., -18., 6.),
                    horner_monic!(c, -7., 10.),
                    horner!(c, -4., 8.),
                    horner!(c, -4., 7.),
                    c - 1.,
                    Cplx::new(4., 0.),
                    TWO,
                    ONE,
                ];
                solve_polynomial(&coeffs)
            }
            5 =>
            {
                let coeffs = [
                    horner_monic!(c, 1., -9., 33., -64., 76., -66., 50., -31., 15., -5.),
                    -horner_monic!(c, 1., -7., 20., -26., 8., 18., -23., 15., -5.),
                    horner!(c, -14., 115., -378., 652., -700., 567., -386., 204., -74., 15.),
                    horner_monic!(c, 13., -81., 202., -208., -7., 205., -188., 84., -19.),
                    horner!(c, 85., -636., 1862., -2837., 2738., -2019., 1185., -478., 103.),
                    horner!(c, -72., 398., -855., 633., 357., -879., 550., -158., 14.),
                    horner_monic!(c, -291., 1979., -5108., 6786., -5793., 3741., -1712., 408.),
                    horner!(c, 219., -1071., 1936., -786., -1457., 1771., -696., 88.),
                    horner!(c, 606., -3735., 8345., -9421., 6869., -3538., 954., 17.),
                    horner_monic!(c, -387., 1671., -2431., -49., 2480., -1666., 295.),
                    horner!(c, -751., 4182., -7780., 7018., -3921., 1127., 118.),
                    horner!(c, 364., -1396., 1498., 998., -1783., 493., 16.),
                    horner_monic!(c, 437., -2221., 2975., -1496., 50., 425.),
                    horner!(c, -73., 315., -228., -556., 153., 93.),
                    horner!(c, 126., -417., 1194., -1410., 807., 15.),
                    horner_monic!(c, -199., 409., 22., -334., 253.),
                    horner!(c, -375., 1220., -1464., 675., 83.),
                    horner!(c, 176., -276., -243., 221., 12.),
                    horner_monic!(c, 185., -431., 162., 206.),
                    horner!(c, -9., 9., 72., 54.),
                    horner!(c, 34., -125., 161., 11.),
                    horner_monic!(c, -43., -7., 37.),
                    horner!(c, -51., 66., 42.),
                    horner!(c, 8., 18., 10.),
                    horner_monic!(c, 5., 13.),
                    horner!(c, 3., 3.),
                    horner!(c, 2., 9.),
                    c + 1.,
                    ONE,
                    ZERO,
                    ONE,
                ];
                solve_polynomial(&coeffs)
            }
            6 =>
            {
                let coeffs = [
                    horner_monic!(
                        c, 1., -19., 162., -822., 2781., -6677., 11858., -16093., 17187., -14858.,
                        10683., -6549., 3486., -1617., 645., -213., 55., -10.
                    ),
                    horner!(
                        c, 1., -19., 158., -764., 2409., -5279., 8409., -10129., 9582., -7340.,
                        4625., -2411., 1041., -363., 99., -19., 2.
                    ),
                    horner!(
                        c, -27., 485., -3900., 18618., -59129., 132998., -220866., 279744.,
                        -278190., 223353., -148696., 83944., -40647., 16764., -5708., 1512., -280.,
                        28.
                    ),
                    horner!(
                        c, -27., 483., -3768., 17042., -50158., 102498., -152240., 170888.,
                        -150093., 105861., -60722., 28380., -10696., 3144., -672., 88., -4.
                    ),
                    horner!(
                        c, 337., -5719., 43334., -194405., 578641., -1216500., 1882943., -2216106.,
                        2040958., -1511772., 923504., -473398., 203842., -71966., 19682., -3738.,
                        378.
                    ),
                    horner!(
                        c, 337., -5671., 41461., -175199., 480666., -914206., 1262034., -1312870.,
                        1061802., -682088., 350924., -143806., 45870., -10758., 1638., -118., 2.
                    ),
                    horner!(
                        c, -2575., 41258., -294310., 1239024., -3448984., 6756464., -9706528.,
                        10558044., -8944088., 6057758., -3350958., 1526584., -563744., 160268.,
                        -31448., 3248.
                    ),
                    horner!(
                        c, -2576., 40746., -278886., 1099544., -2806724., 4954112., -6326464.,
                        6054604., -4461812., 2573162., -1163106., 405280., -104168., 17872.,
                        -1632., 48.
                    ),
                    horner!(
                        c, 13433., -203103., 1362566., -5373426., 13949591., -25362021., 33638139.,
                        -33584651., 25937032., -15859612., 7784665., -3044429., 907591., -185397.,
                        19717., 7.
                    ),
                    horner_monic!(
                        c, 13457., -199943., 1279710., -4699338., 11131207., -18157443., 21309749.,
                        -18579477., 12302580., -6243754., 2407279., -680595., 130003., -13943.,
                        543.
                    ),
                    horner!(
                        c, -50439., 719365., -4533398., 16708778., -40307288., 67666978.,
                        -82294604., 74744256., -51971031., 28138639., -11855580., 3749752.,
                        -804114., 88508., 172.
                    ),
                    horner!(
                        c, -50703., 707245., -4227246., 14424942., -31587018., 47337718.,
                        -50600488., 39660976., -23143141., 10041159., -3154684., 668984., -81488.,
                        3760., 24.
                    ),
                    horner!(
                        c,
                        139641.,
                        -1878295.,
                        11104345.,
                        -38140597.,
                        85101800.,
                        -131050880.,
                        144844712.,
                        -118200524.,
                        72611280.,
                        -33709464.,
                        11491084.,
                        -2611776.,
                        297488.,
                        2004.
                    ),
                    horner!(
                        c, 141401., -1850969., 10314875., -32605039., 65657036., -89653108.,
                        86188948., -59590172., 29772648., -10548860., 2494988., -338960., 17184.,
                        288.
                    ),
                    horner!(
                        c,
                        -285574.,
                        3624260.,
                        -20073712.,
                        64026500.,
                        -131353940.,
                        183941448.,
                        -182486140.,
                        131308964.,
                        -69029016.,
                        25879256.,
                        -6300076.,
                        733560.,
                        14560.,
                        4.
                    ),
                    horner!(
                        c,
                        -293488.,
                        3605312.,
                        -18695532.,
                        54507452.,
                        -100184300.,
                        123151932.,
                        -104501212.,
                        61891956.,
                        -25246296.,
                        6715228.,
                        -1004100.,
                        50544.,
                        2248.
                    ),
                    horner!(
                        c,
                        419837.,
                        -5038483.,
                        26130440.,
                        -77087130.,
                        144302895.,
                        -181540022.,
                        158593082.,
                        -97346508.,
                        41069514.,
                        -10789359.,
                        1218904.,
                        72924.,
                        101.
                    ),
                    horner!(
                        c,
                        445061.,
                        -5136521.,
                        24733624.,
                        -66118370.,
                        109681320.,
                        -119092022.,
                        86455412.,
                        -41606568.,
                        12566637.,
                        -2012227.,
                        73258.,
                        12452.,
                        6.
                    ),
                    horner_monic!(
                        c,
                        -402715.,
                        4615085.,
                        -22501084.,
                        61217820.,
                        -103519915.,
                        114706902.,
                        -84967524.,
                        41093432.,
                        -11426896.,
                        922371.,
                        261348.,
                        1192.
                    ),
                    horner!(
                        c, -460771., 5024075., -22444406., 54542208., -80082814., 74036870.,
                        -42960604., 14624736., -2234007., -95285., 50182., 132.
                    ),
                    horner!(
                        c, 152401., -1817536., 8759138., -22340230., 33670078., -30969556.,
                        16390134., -3202564., -1293752., 668578., 8670., 20.
                    ),
                    horner!(
                        c, 248016., -2648812., 11106684., -24169070., 29710276., -20418024.,
                        6485256., 340644., -799272., 145702., 1350.
                    ),
                    horner!(
                        c, 199211., -1652290., 5825590., -11825516., 15524212., -14447008.,
                        10070320., -4899132., 1162496., 42170., 210.
                    ),
                    horner!(
                        c, 92656., -614634., 1818002., -3746476., 6291636., -7519688., 5441480.,
                        -2057340., 286284., 8330., 6.
                    ),
                    horner!(
                        c, -372527., 3148021., -10933094., 20475234., -22656438., 15554390.,
                        -6474468., 1173696., 139859., 1555.
                    ),
                    horner!(
                        c, -308207., 2407525., -7578118., 12872510., -13170834., 8205790.,
                        -2781056., 315800., 34103., 111.
                    ),
                    horner!(
                        c, 241617., -1983435., 6358228., -10253492., 8717842., -3653198., 253936.,
                        307328., 8285., 5.
                    ),
                    horner_monic!(
                        c, 253889., -1878479., 5253372., -7242876., 5215986., -1693638., -1384.,
                        91440., 969.
                    ),
                    horner!(
                        c, 10866., 53830., -332366., 330178., 464512., -876588., 402536., 30828.,
                        120.
                    ),
                    horner!(
                        c, -52906., 402450., -916814., 630094., 271396., -499108., 153604., 5372.,
                        16.
                    ),
                    horner!(
                        c, -129360., 735268., -1597204., 1720788., -1055996., 254540., 75268.,
                        1100.
                    ),
                    horner!(
                        c, -74908., 389012., -825772., 977428., -608156., 124924., 17588., 140.
                    ),
                    horner!(c, 79287., -425021., 757246., -468595., 10509., 99000., 5635., 10.),
                    horner!(c, 66679., -309305., 471531., -276799., 10364., 35028., 964.),
                    horner!(c, -1205., 27224., -32900., -66694., 54594., 17432., 130.),
                    horner!(c, -14150., 52290., -20798., -49522., 28956., 3536., 4.),
                    horner!(c, -19285., 54161., -32830., -2459., 23599., 854., 1.),
                    horner!(c, -7951., 22373., -24851., 4499., 8236., 108.),
                    horner!(c, 7563., -16232., -7724., 11956., 3604., 12.),
                    horner!(c, 4936., -6652., -5112., 6068., 504.),
                    horner!(c, -313., 3271., -1381., 4625., 92.),
                    horner!(c, -881., 393., 629., 1611., 12.),
                    horner!(c, -216., -2526., 2254., 658.),
                    horner!(c, 210., -1222., 998., 54.),
                    horner!(c, 197., 145., 649., 3.),
                    horner!(c, -7., 203., 255., 1.),
                    horner!(c, -194., 232., 104.),
                    horner!(c, -92., 88., 4.),
                    horner!(c, 27., 58.),
                    horner!(c, 18., 32.),
                    horner!(c, 9., 14.),
                    Cplx::new(4., 0.),
                    Cplx::new(3., 0.),
                    TWO,
                    ONE,
                ];
                solve_polynomial(&coeffs)
            }
            _ => vec![],
        }
    }

    #[inline]
    fn default_julia_bounds(&self, _point: Cplx, _param: Cplx) -> Bounds
    {
        Bounds::centered_square(4.)
    }
}

const A0: Cplx = Cplx::new(-5448., 6_051.300_686_629_28);
const A1: Cplx = Cplx::new(-29_961.795_134_443_0, 43_861.639_473_933_7);
const A2: Cplx = Cplx::new(-65_413.655_299_273_2, 128_711.643_030_672);
const A3: Cplx = Cplx::new(-70_918.940_786_376_0, 196_781.349_743_989);
const A4: Cplx = Cplx::new(-38_246.235_127_179_3, 165_912.340_564_512);
const A5: Cplx = Cplx::new(-8_271.848_132_127_45, 73_334.197_922_255_2);
const A6: Cplx = Cplx::new(-44.432_836_932_486_6, 13_302.145_857_037_4);

const B0: Cplx = Cplx::new(-6174., 0.);
const B1: Cplx = Cplx::new(-38_914.156_209_987_2, 1_067.791_134_284_38);
const B2: Cplx = Cplx::new(-102_108.377_281_498, 5_375.650_615_514_38);
const B3: Cplx = Cplx::new(-142_796.822_391_875, 10_800.604_008_295_7);
const B4: Cplx = Cplx::new(-112_272.282_050_380, 10_824.434_074_704_7);
const B5: Cplx = Cplx::new(-47_060.675_356_870_1, 5_410.564_894_838_89);
const B6: Cplx = Cplx::new(-8_216.992_738_080_66, 1_078.880_698_179_05);

impl HasDynamicalCovers for QuadRatPer2
{
    fn dynatomic_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> Cplx;
        let bounds: Bounds;

        match period
        {
            1 =>
            {
                param_map = |c| (4. - c * (c + 2.)) * c / 8.;
                bounds = Bounds {
                    min_x: -5.0,
                    max_x: 3.0,
                    min_y: -3.0,
                    max_y: 3.0,
                };
            }
            3 =>
            {
                param_map = |t| -horner_monic!(t, -OMEGA, -1., 2. * OMEGA + 1.) / (OMEGA + 1.);
                bounds = Bounds {
                    min_x: -1.8,
                    max_x: 1.8,
                    min_y: -2.3,
                    max_y: 1.2,
                };
            }
            4 =>
            {
                const A0: Cplx = Cplx::new(-23.0000000000000, -14.0000000000000);
                const A1: Cplx = Cplx::new(-43.5952148437500, -198.812988281250);
                const A2: Cplx = Cplx::new(393.670501828194, -585.061190664768);
                const A3: Cplx = Cplx::new(1459.26865290843, -251.041057291222);
                const A4: Cplx = Cplx::new(1689.73657634438, 1253.11836853702);
                const A5: Cplx = Cplx::new(263.170605532593, 2124.30689546928);
                const A6: Cplx = Cplx::new(-1022.56887552264, 1244.03619728333);
                const A7: Cplx = Cplx::new(-907.580743841250, 62.5812248360001);
                const A8: Cplx = Cplx::new(-282.801070639887, -262.556139911993);
                const A9: Cplx = Cplx::new(-0.301795185485805, -120.425962376276);
                const A10: Cplx = Cplx::new(19.1705165819150, -17.9211712972486);
                const A11: Cplx = Cplx::new(3.55738991578644, 0.247670436437753);
                const A12: Cplx = Cplx::new(0.142742524186783, 0.175662128168037);

                const B0: Cplx = Cplx::new(25.0000000000000, 50.0000000000000);
                const B1: Cplx = Cplx::new(-142.712402343750, 397.741699218750);
                const B2: Cplx = Cplx::new(-1337.90185153484, 594.505678117275);
                const B3: Cplx = Cplx::new(-2843.31086660859, -1168.74148176612);
                const B4: Cplx = Cplx::new(-1585.98927368805, -4058.84188445190);
                const B5: Cplx = Cplx::new(1856.77078333192, -3982.88918800431);
                const B6: Cplx = Cplx::new(3064.65024283614, -1032.41304011642);
                const B7: Cplx = Cplx::new(1547.93271225278, 818.021881782741);
                const B8: Cplx = Cplx::new(185.634744083957, 667.196208176713);
                const B9: Cplx = Cplx::new(-101.261202462199, 167.059013936565);
                const B10: Cplx = Cplx::new(-36.5500089943826, 7.62187524951790);
                const B11: Cplx = Cplx::new(-3.53720886972747, -2.52453753137727);
                const B12: Cplx = Cplx::new(-0.0276521249511676, -0.231588268174843);

                // Mobius transformation to frame the image
                const POLE: Cplx = Cplx::new(-0.93856601763702075, 2.1250254224644328);
                const SHIFT: Cplx = Cplx::new(0.006285758096917293865, 0.69546218693638377);
                const ANGLE: Cplx = Cplx::new(0.3016938919708282662, 0.1676310038253636);

                param_map = |t| {
                    let t = (t * ANGLE + SHIFT).inv() + POLE;
                    let numer = horner!(t, A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12);
                    let denom = horner!(t, B0, B1, B2, B3, B4, B5, B6, B7, B8, B9, B10, B11, B12);
                    -numer / denom
                };
                bounds = Bounds {
                    min_x: -4.3,
                    max_x: 3.4,
                    min_y: -4.,
                    max_y: 4.,
                }
            }
            _ =>
            {
                param_map = |c| c;
                bounds = self.point_grid.bounds.clone();
            }
        }
        let grid = self.point_grid.new_with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }
    fn marked_cycle_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> Cplx;
        let bounds: Bounds;

        match period
        {
            1 =>
            {
                param_map = |c| (4. - c * (c + 2.)) * c / 8.;
                bounds = Bounds {
                    min_x: -5.0,
                    max_x: 3.0,
                    min_y: -3.0,
                    max_y: 3.0,
                };
            }
            4 =>
            {
                param_map = |c| {
                    let u = c * c;
                    u * c - 2. * u + 4. * c - 1.
                };
                bounds = Bounds {
                    min_x: -1.,
                    max_x: 1.4,
                    min_y: -2.2,
                    max_y: 2.2,
                };
            }
            5 =>
            {
                param_map = |c| {
                    // t = sqrt(-2235)
                    // ((-2043332879690812551104*t + 322671215001188162496)*c^6 + (-7211787718815174272*t + 38457203855637713472)*c^5 + (-10445615819508480*t + 113836835145028800)*c^4 + (-7931553616080*t + 135137329840080)*c^3 + (-3321323160*t + 79799557200)*c^2 + (-724598*t + 23400162)*c + (-64*t + 2724))/((-165726073638468871360*t + 59671792608719217337728)*c^6 + (-532082528560799520*t + 218792941658814953376)*c^5 + (-681491680626360*t + 334169395252260120)*c^4 + (-435333784880*t + 272101938829200)*c^3 + (-138715290*t + 124564255830)*c^2 + (-17640*t + 30391956)*c + 3087)
                    let pole = Cplx::new(-1.029_131_872_704_64, 0.051_564_155_271_414_3);
                    let angle = Cplx::new(1., 0.);

                    let c = angle / c + pole;

                    let numer = A0 + c * (A1 + c * (A2 + c * (A3 + c * (A4 + c * (A5 + c * A6)))));
                    let denom = B0 + c * (B1 + c * (B2 + c * (B3 + c * (B4 + c * (B5 + c * B6)))));

                    -numer / denom
                };
                bounds = Bounds {
                    min_x: -8.,
                    max_x: 5.5,
                    min_y: -1.5,
                    max_y: 8.,
                };
            }
            _ =>
            {
                param_map = |c| c;
                bounds = self.point_grid.bounds.clone();
            }
        };
        let grid = self.point_grid.new_with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }

    fn misiurewicz_curve(self, preperiod: Period, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> Cplx;
        let bounds: Bounds;

        match (preperiod, period)
        {
            (2, 1) =>
            {
                param_map = |c| {
                    let c2 = c * c;
                    // -25*(131*t^4 - 102*t^3 - 106*t^2 - 8*t - 4)*t^2/(13*t^2 + 2*t + 2)^3
                    let denom = 13. * c2 + c + c + 2.;
                    let numer = c2 * (131. * c2 - 102. * c - 106.) - 8. * c - 4.;
                    25. * c2 * numer / (denom * denom * denom)
                };
                bounds = Bounds {
                    min_x: -3.4,
                    max_x: 3.4,
                    min_y: -5.1,
                    max_y: 5.1,
                };
            }
            (2, 2) =>
            {
                param_map = |c| {
                    //(-t^4 + 2*t^2 + 1)/(2*t^4)
                    let c2 = c * c;
                    0.5 - (c2 + 0.5) / (c2 * c2)
                };
                bounds = Bounds {
                    min_x: -4.,
                    max_x: 4.,
                    min_y: -4.,
                    max_y: 4.,
                };
            }
            (_, _) =>
            {
                param_map = |c| c;
                bounds = self.point_grid.bounds.clone();
            }
        };
        let grid = self.point_grid.new_with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }
}
