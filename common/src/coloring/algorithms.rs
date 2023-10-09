use super::palette::ColorPalette;
use super::types::Hsv;
use crate::consts::TAU;
use crate::orbit_info::PointInfoPeriodic;
use crate::traits::Polar;
use crate::types::{IterCount, Real};
use egui::Color32;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum IncoloringAlgorithm
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
    // PointBased
    // {
    //     points: Vec<Cplx>,
    //     tolerance: Real,
    // },
}
impl IncoloringAlgorithm
{
    fn multiplier_coloring_rate(mult_norm: Real, fill_rate: Real) -> f64
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
    pub fn color_periodic<V, D>(
        &self,
        palette: ColorPalette,
        point_info: PointInfoPeriodic<V, D>,
    ) -> Color32
    where
        D: Polar<Real>,
    {
        match self
        {
            Self::Solid => palette.in_color,
            Self::Period =>
            {
                let hue_id = point_info.period as f32;
                palette.period_coloring.map(hue_id, 0.75)
            }
            Self::PeriodMultiplier =>
            {
                let hue_id = point_info.period as f32;
                let luminosity_modifier = point_info.multiplier.norm() as f32;
                palette.period_coloring.map(hue_id, luminosity_modifier)
            }
            Self::Preperiod =>
            {
                let per = IterCount::from(point_info.period);
                let val = IterCount::from(point_info.preperiod);

                palette.map_color32(val * val / per)
            }
            Self::PreperiodPeriod =>
            {
                let coloring_rate = 0.02;

                let per = IterCount::from(point_info.period);
                let val = IterCount::from(point_info.preperiod);

                let potential = (val * coloring_rate / per).tanh();
                palette
                    .period_coloring
                    .map(per as f32, potential as f32)
            }
            Self::InternalPotential {
                periodicity_tolerance,
            } =>
            {
                let per = IterCount::from(point_info.period);
                let val: IterCount;

                let mult_norm = point_info.multiplier.norm();

                // Superattracting case
                if mult_norm <= 1e-10
                {
                    let potential = 2.
                        * (point_info.final_error.log(*periodicity_tolerance)).log2() as IterCount;
                    val = per.mul_add(-potential, IterCount::from(point_info.preperiod));
                }
                // Parabolic case
                else if 1. - mult_norm <= 1e-5
                {
                    let potential = point_info.final_error / periodicity_tolerance;
                    val = per.mul_add(-potential, IterCount::from(point_info.preperiod));
                }
                else
                {
                    let mut potential = -(point_info.final_error / periodicity_tolerance)
                        .log(mult_norm) as IterCount;
                    if potential.is_infinite() || potential.is_nan()
                    {
                        potential = -0.2;
                    }
                    val = per.mul_add(potential, f64::from(point_info.preperiod));
                }
                palette.map_color32(val * val / per)
            }
            Self::PreperiodPeriodSmooth {
                periodicity_tolerance,
                fill_rate,
            } =>
            {
                let hue_id = IterCount::from(point_info.period);
                let luminosity_modifier: IterCount;

                let mult_norm = point_info.multiplier.norm();

                // Superattracting case
                if mult_norm <= 1e-12
                {
                    let w = 2.
                        * (point_info.final_error.log(*periodicity_tolerance)).log2() as IterCount;
                    let v = hue_id.mul_add(-w, IterCount::from(point_info.preperiod));
                    luminosity_modifier = (0.1 * v / hue_id).tanh();
                }
                // Parabolic case
                else if 1. - mult_norm <= 1e-5
                {
                    let w = point_info.final_error / periodicity_tolerance;
                    let v = hue_id.mul_add(-w, IterCount::from(point_info.preperiod));
                    luminosity_modifier = (0.1 * v / hue_id).tanh();
                }
                else
                {
                    let coloring_rate = Self::multiplier_coloring_rate(mult_norm, *fill_rate);

                    let mut w = -(point_info.final_error / periodicity_tolerance).log(mult_norm)
                        as IterCount;
                    if w.is_infinite() || w.is_nan()
                    {
                        w = -0.2;
                    }
                    let v = hue_id.mul_add(w, f64::from(point_info.preperiod));
                    luminosity_modifier = (v * coloring_rate / hue_id).tanh();
                }
                palette
                    .period_coloring
                    .map(hue_id as f32, luminosity_modifier as f32)
            }
            Self::Multiplier => Hsv::new(
                (point_info.multiplier.arg() / TAU) as f32 + 0.5,
                1.,
                point_info.multiplier.norm() as f32,
            )
            .into(),
            // Self::PointBased { points, tolerance } =>
            // {
            //     for (i, pt) in points.iter().enumerate()
            //     {
            //         if (point_info.value - pt).norm_sqr() < *tolerance
            //         {
            //             let hue = (i as f32) / (points.len() as f32);
            //             return Hsv {
            //                 hue,
            //                 saturation: 0.8,
            //                 luminosity: 1.0,
            //             }.into();
            //         }
            //     }
            //     palette.in_color
            // }
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

impl Default for IncoloringAlgorithm
{
    fn default() -> Self
    {
        Self::PeriodMultiplier
    }
}
