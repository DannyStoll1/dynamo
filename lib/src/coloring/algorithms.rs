use super::palette::ColorPalette;
use super::types::Hsv;
use crate::consts::*;
use crate::types::*;
use egui::Color32;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ColoringAlgorithm
{
    PeriodMultiplier,
    Period,
    Solid,
    InternalPotential
    {
        periodicity_tolerance: f64,
    },
    Preperiod,
    PreperiodPeriodSmooth
    {
        periodicity_tolerance: f64,
        fill_rate: f64,
    },
    PreperiodPeriod,
    Multiplier,
}
impl ColoringAlgorithm
{
    fn multiplier_coloring_rate(mult_norm: RealNum, fill_rate: RealNum) -> f64
    {
        let scaling_rate = mult_norm;

        if scaling_rate > 1e-10
        {
            -scaling_rate.log2() * fill_rate
        }
        else
        {
            10.
        }
    }

    #[must_use]
    pub fn color_periodic<D>(
        &self,
        palette: ColorPalette,
        period: Period,
        preperiod: Period,
        multiplier: D,
        final_error: RealNum,
    ) -> Color32
    where
        D: Norm<RealNum>,
    {
        match self
        {
            Self::Solid => palette.in_color,
            Self::Period =>
            {
                let hue = period as f32;
                palette.period_coloring.map_color32(hue, 0.75)
            }
            Self::PeriodMultiplier =>
            {
                let hue = period as f32;
                let luminosity = multiplier.norm() as f32;
                palette.period_coloring.map_color32(hue, luminosity)
            }
            Self::Preperiod =>
            {
                let _coloring_rate = 0.02;
                let per = IterCount::from(period);
                let val = IterCount::from(preperiod);

                palette.map_color32(val * val / per)
            }
            Self::PreperiodPeriod =>
            {
                let coloring_rate = 0.02;

                let per = IterCount::from(period);
                let val = IterCount::from(preperiod);

                let potential = (val * coloring_rate / per).tanh();
                palette
                    .period_coloring
                    .map_color32(per as f32, potential as f32)
            }
            Self::InternalPotential {
                periodicity_tolerance,
            } =>
            {
                let per = IterCount::from(period);
                let val: IterCount;

                let mult_norm = multiplier.norm();

                // Superattracting case
                if mult_norm <= 1e-10
                {
                    let potential =
                        2. * (final_error.log(*periodicity_tolerance)).log2() as IterCount;
                    val = per.mul_add(-potential, IterCount::from(preperiod));
                }
                // Parabolic case
                else if 1. - mult_norm <= 1e-5
                {
                    let potential = final_error / periodicity_tolerance;
                    val = per.mul_add(-potential, IterCount::from(preperiod));
                }
                else
                {
                    let mut potential =
                        -(final_error / periodicity_tolerance).log(mult_norm) as IterCount;
                    if potential.is_infinite() || potential.is_nan()
                    {
                        potential = -0.2;
                    }
                    val = per.mul_add(potential, f64::from(preperiod));
                }
                palette.map_color32(val * val / per)
            }
            Self::PreperiodPeriodSmooth {
                periodicity_tolerance,
                fill_rate,
            } =>
            {
                let hue = IterCount::from(period);
                let luminosity: IterCount;

                let mult_norm = multiplier.norm();

                // Superattracting case
                if mult_norm <= 1e-10
                {
                    let w = 2. * (final_error.log(*periodicity_tolerance)).log2() as IterCount;
                    let v = hue.mul_add(-w, IterCount::from(preperiod));
                    luminosity = (0.1 * v / hue).tanh();
                }
                // Parabolic case
                else if 1. - mult_norm <= 1e-5
                {
                    let w = final_error / periodicity_tolerance;
                    let v = hue.mul_add(-w, IterCount::from(preperiod));
                    luminosity = (0.1 * v / hue).tanh();
                }
                else
                {
                    let coloring_rate = Self::multiplier_coloring_rate(mult_norm, *fill_rate);

                    let mut w = -(final_error / periodicity_tolerance).log(mult_norm) as IterCount;
                    if w.is_infinite() || w.is_nan()
                    {
                        w = -0.2;
                    }
                    let v = hue.mul_add(w, f64::from(preperiod));
                    luminosity = (v * coloring_rate / hue).tanh();
                }
                palette
                    .period_coloring
                    .map_color32(hue as f32, luminosity as f32)
            }
            Self::Multiplier => Hsv::new(
                (multiplier.arg() / TAU) as f32 + 0.5,
                1.,
                multiplier.norm() as f32,
            )
            .into(),
        }
    }

    // pub fn encode_periodic(
    //     &self,
    //     period: Period,
    //     preperiod: Period,
    //     multiplier: ComplexNum,
    //     final_error: ComplexNum,
    // ) -> IterCount
    // {
    //     let hue: f64;
    //     let luminosity: f64;
    //     match self
    //     {
    //         Self::Solid => return 0.,
    //         Self::Period =>
    //         {
    //             hue = IterCount::from(period);
    //             luminosity = 1.;
    //         }
    //         Self::PeriodMultiplier =>
    //         {
    //             hue = IterCount::from(period);
    //             luminosity = multiplier.norm();
    //         }
    //         Self::Preperiod =>
    //         {
    //             let coloring_rate = 0.02;
    //
    //             hue = IterCount::from(period);
    //
    //             let v = IterCount::from(preperiod);
    //             luminosity = (v * coloring_rate / hue).tanh();
    //         }
    //         Self::PreperiodSmooth {
    //             periodicity_tolerance,
    //         } =>
    //         {
    //             hue = IterCount::from(period);
    //             let mult_norm = multiplier.norm();
    //
    //             // Superattracting case
    //             if mult_norm <= 1e-10
    //             {
    //                 let w = 2.
    //                     * (final_error.norm_sqr().log2() / periodicity_tolerance.log2()).log2()
    //                         as IterCount;
    //                 let v = preperiod as IterCount - hue * w;
    //                 luminosity = (0.1 * v / hue).tanh();
    //             }
    //             // Parabolic case
    //             else if 1. - mult_norm <= 1e-5
    //             {
    //                 let w = final_error.norm_sqr() / periodicity_tolerance;
    //                 let v = preperiod as IterCount - hue * w;
    //                 luminosity = (0.1 * v / hue).tanh();
    //             }
    //             else
    //             {
    //                 let coloring_rate = multiplier_coloring_rate(multiplier);
    //
    //                 let mut w = -(final_error.norm_sqr() / periodicity_tolerance)
    //                     .log(multiplier.norm()) as IterCount;
    //                 if w.is_infinite() || w.is_nan()
    //                 {
    //                     w = -0.2;
    //                 }
    //                 let v = preperiod as IterCount + hue * w;
    //                 luminosity = (v * coloring_rate / hue).tanh();
    //             }
    //         }
    //         Self::Multiplier =>
    //         {
    //             hue = multiplier.arg() / TAU + 0.5;
    //             luminosity = multiplier.norm();
    //         }
    //     }
    //     -(hue + 0.9999 * luminosity)
    // }
}

impl Default for ColoringAlgorithm
{
    fn default() -> Self
    {
        Self::PeriodMultiplier
    }
}
