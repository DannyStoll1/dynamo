use crate::macros::{degree_impl, ext_ray_impl_nonmonic, horner_monic, profile_imports};
use dynamo_common::{horner, math_utils::roots_of_unity};
profile_imports!();

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Unicritical<const D: i32>
{
    point_grid: PointGrid,
    compute_mode: ComputeMode,
    max_iter: Period,
}

impl<const D: i32> Unicritical<D>
{
    const D_FLOAT: Real = D as Real;
    const CRIT: Cplx = Cplx::new(-Self::D_FLOAT, 0.0);
    const DEFAULT_BOUNDS: Bounds =
        Bounds::square(Self::D_FLOAT * 1.2, Cplx::new(-Self::D_FLOAT + 1.0, 0.0));
}

impl<const D: i32> Default for Unicritical<D>
{
    fractal_impl!();
}

#[allow(clippy::suspicious_operation_groupings)]
impl<const D: i32> DynamicalFamily for Unicritical<D>
{
    parameter_plane_impl!();

    #[inline]
    fn map(&self, z: Self::Var, c: &Self::Param) -> Self::Var
    {
        c * (1. + z / Self::D_FLOAT).powi(D)
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv)
    {
        let u = 1. + z / Self::D_FLOAT;
        let df = c * u.powi(D - 1);
        (u * df, df)
    }

    fn gradient(&self, z: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        let u = 1. + z / Self::D_FLOAT;
        let v = u.powi(D - 1);
        let df = c * v;
        (u * df, df, u * v)
    }

    fn start_point(&self, _point: Cplx, _c: &Self::Param) -> Self::Var
    {
        ZERO
    }

    fn periodicity_tolerance(&self) -> Real
    {
        1e-18
    }

    fn name(&self) -> String
    {
        format!("Unicritical({D})")
    }
}

impl<const D: i32> FamilyDefaults for Unicritical<D>
{
    default_bounds!();

    fn default_selection(&self) -> Cplx
    {
        let zeta = (TAUI / Self::D_FLOAT).exp();
        (zeta - 1.) * Self::D_FLOAT
    }
}

impl<const D: i32> HasJulia for Unicritical<D>
{
    fn default_bounds_child(&self, _point: Cplx, _c: &Self::Param) -> Bounds
    {
        Bounds::square(Self::D_FLOAT * 1.618, Self::CRIT)
    }
}

impl<const D: i32> MarkedPoints for Unicritical<D>
{
    #[inline]
    fn critical_points_child(&self, _c: &Self::Param) -> Vec<Self::Var>
    {
        vec![Self::CRIT]
    }

    fn cycles_child(&self, c: &Self::Param, period: Period) -> Vec<Self::Var>
    {
        use dynamo_common::math_utils::binomial;
        match period {
            1 => {
                let mut coeffs: Vec<Cplx> =
                    (0..=D).map(|x| c * Real::from(binomial(D, x))).collect();
                coeffs[1] -= Self::D_FLOAT;
                solve_polynomial(coeffs)
                    .iter()
                    .map(|z| z * Self::D_FLOAT)
                    .collect()
            }
            _ => vec![],
        }
    }
}

impl<const D: i32> InfinityFirstReturnMap for Unicritical<D>
{
    #[inline]
    fn degree(&self) -> AngleNum
    {
        D.into()
    }

    #[inline]
    fn degree_real(&self) -> Real
    {
        Self::D_FLOAT
    }

    #[inline]
    fn escape_coeff(&self, c: &Self::Param) -> Cplx
    {
        c * Self::D_FLOAT.powi(-D)
    }

    #[inline]
    fn escape_coeff_d(&self, c: &Self::Param) -> (Cplx, Cplx)
    {
        let a = Self::D_FLOAT.powi(-D);
        (c * a, a.into())
    }
}

impl<const D: i32> EscapeEncoding for Unicritical<D> {}

impl<const D: i32> ExternalRays for Unicritical<D>
{
    ext_ray_impl_nonmonic!();
}

// const U3_MC_3_POLE_0: Cplx = Cplx::new(
//     -6.3071559227053154449928460559449771172,
//     -4.4052647736416225259941095453003318476,
// );
//
// const U3_MC_3_POLE_1: Cplx = Cplx::new(
//     -5.2340864872432865860914863278284128442,
//     -4.3028610084688658507946609003725230190,
// );

impl HasDynamicalCovers for Unicritical<3>
{
    fn marked_cycle_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> (Cplx, Cplx);
        let bounds: Bounds;

        match period {
            1 => {
                param_map = |t| (3. * (t + 1.) * t.powi(2), 3. * t * (3. * t + 2.));
                bounds = Bounds {
                    min_x: -1.8,
                    max_x: 0.9,
                    min_y: -1.2,
                    max_y: 1.2,
                };
            }
            2 => {
                param_map = |t| (3. * (t - 2.) * t.powi(2), 3. * t * (3. * t - 4.));
                bounds = Bounds {
                    min_x: -1.,
                    max_x: 2.5,
                    min_y: -1.,
                    max_y: 1.,
                };
            }
            3 => {
                const DEN_0_0: Cplx = Cplx::new(15.019_639_247_721_374, 48.282_356_214_136_12);
                const DEN_0_1: Cplx = Cplx::new(11.411_649_536_823_681, 8.425_252_873_580_56);
                const DEN_1_0: Cplx = Cplx::new(14.056_957_561_484_392, 50.196_352_118_588_65);
                const DEN_1_1: Cplx = Cplx::new(11.541_242_409_948_602, 8.708_125_782_110_49);

                const NUM_0: Cplx = Cplx::new(-1_744.408_589_013_732_4, 1_473.740_602_292_486_8);
                const NUM_1: Cplx = Cplx::new(-343.849_951_900_078, 1_273.100_690_510_641);
                const NUM_2: Cplx = Cplx::new(96.708_321_954_359_62, 269.238_722_028_364_3);
                const NUM_3: Cplx = Cplx::new(22.564_029_832_221_34, 15.916_865_188_953_581);
                const NUM_COEF: Cplx = Cplx::new(-2.216_531_263_344_174, 0.388_928_257_728_017_26);

                const DNUM_2: Cplx = Cplx::new(2. * NUM_2.re, 2. * NUM_2.im);
                const DNUM_3: Cplx = Cplx::new(3. * NUM_3.re, 3. * NUM_3.im);

                const POLE_0: Cplx = Cplx::new(-5.914_015_205_273_233, -3.709_866_341_074_397_5);

                const POLE_1: Cplx = Cplx::new(-5.497_634_331_550_449, -4.715_386_532_506_162);

                const ANGLE: Cplx = Cplx::new(0.5 * SQRT_3, -0.5);

                // ANGLE * (POLE_1 - POLE_0)
                const VECT: Cplx = Cplx::new(-0.142_163_681_421_990_37, -1.078_996_466_659_493_8);

                param_map = |t| {
                    let u = t * ANGLE;
                    let v = u + 1.;
                    let w = (POLE_1 * u + POLE_0) / v;

                    let dw = VECT / v.powi(2);

                    let num0 = horner_monic!(w, NUM_0, NUM_1, NUM_2, NUM_3);
                    let num0_d = horner!(w, NUM_1, DNUM_2, DNUM_3, 4.);

                    let den0 = horner_monic!(w, DEN_0_0, DEN_0_1);
                    let den1 = horner_monic!(w, DEN_1_0, DEN_1_1);
                    let den0_d = horner!(w, DEN_0_1, 2.);
                    let den1_d = horner!(w, DEN_1_1, 2.);

                    let den0_2 = den0 * den0;
                    let den0_3 = den0_2 * den0;

                    let num = NUM_COEF * num0.powi(2);
                    let num_d = 2. * NUM_COEF * num0 * num0_d;

                    let den = den0_3 * den1;
                    let den_d = 3. * den0_2 * den0_d * den1 + den0_3 * den1_d;

                    (num / den, dw * (den * num_d - num * den_d) / den.powi(2))
                };
                bounds = Bounds {
                    min_x: -2.,
                    max_x: 5.8,
                    min_y: -2.,
                    max_y: 3.5,
                };
            }
            _ => {
                param_map = |t| (t, ONE);
                bounds = self.point_grid.bounds.clone();
            }
        };
        CoveringMap::new(self, param_map).with_orig_bounds(bounds)
    }

    #[allow(clippy::suspicious_operation_groupings)]
    fn dynatomic_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> (Cplx, Cplx);
        let bounds: Bounds;

        match period {
            2 => {
                param_map = |t| {
                    let t2 = t.powi(2);
                    let num0 = t - 0.5;
                    let num1 = t2 + t + 1.25;
                    let den0 = t2 + 0.75;
                    let num01 = num0 * num1;

                    let d_den0 = 2. * t;
                    let d_num01 = num0 * (d_den0 + 1.) + num1;

                    let den0_2 = den0 * den0;

                    let num = -3. * num01 * num01;
                    let den = den0 * den0_2;

                    let d_num = -3. * num01 * d_num01;
                    let d_den = 3. * den0_2 * d_den0;

                    (num / den, (den * d_num - num * d_den) / (den * den))
                };
                bounds = Bounds {
                    min_x: -3.5,
                    max_x: 5.5,
                    min_y: -4.5,
                    max_y: 4.5,
                };
            }
            _ => {
                param_map = |t| (t, ONE);
                bounds = self.point_grid.bounds.clone();
            }
        };
        CoveringMap::new(self, param_map).with_orig_bounds(bounds)
    }
}
