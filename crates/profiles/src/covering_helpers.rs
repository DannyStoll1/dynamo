use dynamo_common::prelude::*;
use dynamo_core::prelude::*;

use crate::macros::{horner, horner_monic};

const PERIOD_3_DEN_A0: Cplx = Cplx::new(15.019_639_247_721_374, 48.282_356_214_136_12);
const PERIOD_3_DEN_A1: Cplx = Cplx::new(11.411_649_536_823_681, 8.425_252_873_580_56);
const PERIOD_3_DEN_B0: Cplx = Cplx::new(14.056_957_561_484_392, 50.196_352_118_588_65);
const PERIOD_3_DEN_B1: Cplx = Cplx::new(11.541_242_409_948_602, 8.708_125_782_110_49);

const PERIOD_3_NUM_0: Cplx = Cplx::new(-1_744.408_589_013_732_4, 1_473.740_602_292_486_8);
const PERIOD_3_NUM_1: Cplx = Cplx::new(-343.849_951_900_078, 1_273.100_690_510_641);
const PERIOD_3_NUM_2: Cplx = Cplx::new(96.708_321_954_359_62, 269.238_722_028_364_3);
const PERIOD_3_NUM_3: Cplx = Cplx::new(22.564_029_832_221_34, 15.916_865_188_953_581);
const PERIOD_3_NUM_COEF: Cplx = Cplx::new(-2.216_531_263_344_174, 0.388_928_257_728_017_26);

const PERIOD_3_DNUM_2: Cplx = Cplx::new(2. * PERIOD_3_NUM_2.re, 2. * PERIOD_3_NUM_2.im);
const PERIOD_3_DNUM_3: Cplx = Cplx::new(3. * PERIOD_3_NUM_3.re, 3. * PERIOD_3_NUM_3.im);

const PERIOD_3_POLE_0: Cplx = Cplx::new(-5.914_015_205_273_233, -3.709_866_341_074_397_5);
const PERIOD_3_POLE_1: Cplx = Cplx::new(-5.497_634_331_550_449, -4.715_386_532_506_162);
const PERIOD_3_ANGLE: Cplx = Cplx::new(0.5 * SQRT_3, -0.5);
const PERIOD_3_VECT: Cplx = Cplx::new(-0.142_163_681_421_990_37, -1.078_996_466_659_493_8);

const PERIOD_3_BOUNDS: Bounds = Bounds {
    min_x: -2.,
    max_x: 5.8,
    min_y: -2.,
    max_y: 3.5,
};

const DEGREE_3_DYNATOMIC_PERIOD_2_BOUNDS: Bounds = Bounds {
    min_x: -3.5,
    max_x: 5.5,
    min_y: -4.5,
    max_y: 4.5,
};

fn period_3_marked_cycle_map(t: Cplx) -> (Cplx, Cplx)
{
    let u = t * PERIOD_3_ANGLE;
    let v = u + 1.;
    let w = (PERIOD_3_POLE_1 * u + PERIOD_3_POLE_0) / v;

    let path_deriv = PERIOD_3_VECT / v.powi(2);

    let numerator_base = horner_monic!(w, PERIOD_3_NUM_0, PERIOD_3_NUM_1, PERIOD_3_NUM_2, PERIOD_3_NUM_3);
    let numerator_base_deriv = horner!(w, PERIOD_3_NUM_1, PERIOD_3_DNUM_2, PERIOD_3_DNUM_3, 4.);

    let primary_denominator = horner_monic!(w, PERIOD_3_DEN_A0, PERIOD_3_DEN_A1);
    let secondary_denominator = horner_monic!(w, PERIOD_3_DEN_B0, PERIOD_3_DEN_B1);
    let primary_denominator_deriv = horner!(w, PERIOD_3_DEN_A1, 2.);
    let secondary_denominator_deriv = horner!(w, PERIOD_3_DEN_B1, 2.);

    let primary_denominator_sq = primary_denominator * primary_denominator;
    let primary_denominator_cu = primary_denominator_sq * primary_denominator;

    let numerator = PERIOD_3_NUM_COEF * numerator_base.powi(2);
    let numerator_deriv = 2. * PERIOD_3_NUM_COEF * numerator_base * numerator_base_deriv;

    let denominator = primary_denominator_cu * secondary_denominator;
    let denominator_deriv =
        3. * primary_denominator_sq * primary_denominator_deriv * secondary_denominator
            + primary_denominator_cu * secondary_denominator_deriv;

    (
        numerator / denominator,
        path_deriv * (denominator * numerator_deriv - numerator * denominator_deriv)
            / denominator.powi(2),
    )
}

pub fn degree_3_marked_cycle_curve_period_3<C>(base_curve: C) -> CoveringMap<C>
where
    C: DynamicalFamily<Param = Cplx, Deriv = Cplx>,
{
    CoveringMap::new(base_curve, period_3_marked_cycle_map).with_orig_bounds(PERIOD_3_BOUNDS)
}

fn degree_3_dynatomic_curve_period_2_map(t: Cplx) -> (Cplx, Cplx)
{
    let t2 = t.powi(2);
    let numerator_left = t - 0.5;
    let numerator_right = t2 + t + 1.25;
    let denominator_base = t2 + 0.75;
    let numerator = numerator_left * numerator_right;

    let denominator_base_deriv = 2. * t;
    let numerator_deriv = numerator_left * (denominator_base_deriv + 1.) + numerator_right;
    let denominator_base_sq = denominator_base * denominator_base;

    let value = -3. * numerator * numerator;
    let denominator = denominator_base * denominator_base_sq;
    let value_deriv = -3. * numerator * numerator_deriv;
    let denominator_deriv = 3. * denominator_base_sq * denominator_base_deriv;

    (
        value / denominator,
        (denominator * value_deriv - value * denominator_deriv) / denominator.powi(2),
    )
}

pub fn degree_3_dynatomic_curve_period_2<C>(base_curve: C) -> CoveringMap<C>
where
    C: DynamicalFamily<Param = Cplx, Deriv = Cplx>,
{
    CoveringMap::new(base_curve, degree_3_dynatomic_curve_period_2_map)
        .with_orig_bounds(DEGREE_3_DYNATOMIC_PERIOD_2_BOUNDS)
}
