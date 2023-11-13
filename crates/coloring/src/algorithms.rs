use crate::palette::Palette;
use crate::types::Hsv;
use dynamo_common::prelude::*;
use egui::Color32;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

const PERIOD_LUMA_MODIFIER: f32 = 1.0;

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
        crit_degree: f64,
    },
    Preperiod,
    PotentialAndPeriod
    {
        periodicity_tolerance: f64,
        crit_degree: f64,
        fill_rate: f64,
    },
    PreperiodPeriod
    {
        fill_rate: f64,
    },
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

        if scaling_rate > 1e-10 {
            -scaling_rate.log2() * fill_rate
        } else {
            10.
        }
    }

    #[must_use]
    pub fn color_periodic<D>(&self, palette: &Palette, point_info: &PointInfoPeriodic<D>) -> Color32
    where
        D: Polar<Real>,
    {
        match self {
            Self::Solid => palette.in_color,
            Self::Period => {
                let hue_id = point_info.period as f32;
                palette.period_coloring.map(hue_id, PERIOD_LUMA_MODIFIER)
            }
            Self::PeriodMultiplier => {
                let hue_id = point_info.period as f32;
                let luminosity_modifier = point_info.multiplier.norm() as f32;
                palette.period_coloring.map(hue_id, luminosity_modifier)
            }
            Self::Preperiod => {
                let per = IterCount::from(point_info.period);
                let val = IterCount::from(point_info.preperiod);

                palette.map_color32(val * val / per)
            }
            Self::PreperiodPeriod { fill_rate } => {
                let per = IterCount::from(point_info.period);
                let val = IterCount::from(point_info.preperiod);

                let potential = (val * fill_rate / per).tanh();
                palette.period_coloring.map(per as f32, potential as f32)
            }
            Self::InternalPotential {
                periodicity_tolerance,
                crit_degree,
            } => {
                let val =
                    Self::relative_potential(point_info, *periodicity_tolerance, *crit_degree);
                palette.map_color32(val)
            }
            Self::PotentialAndPeriod {
                periodicity_tolerance,
                crit_degree,
                fill_rate,
            } => {
                let n = IterCount::from(point_info.period);
                let k = IterCount::from(point_info.preperiod);

                let mult_norm = point_info.multiplier.norm();

                let potential = Self::internal_potential(
                    point_info.final_error,
                    *periodicity_tolerance,
                    mult_norm,
                    *crit_degree,
                );

                let val = k / n - potential;
                let luma = val.powi(2) * n;

                let coloring_rate = if mult_norm <= 1e-10 || (1. - mult_norm).abs() < 1e-5 {
                    0.1
                } else {
                    Self::multiplier_coloring_rate(mult_norm, *fill_rate)
                };

                let luminosity_modifier = (coloring_rate * luma).tanh();

                palette
                    .period_coloring
                    .map(point_info.period as f32, luminosity_modifier as f32)
            }
            Self::Multiplier => Hsv::new(
                (point_info.multiplier.arg() / TAU) as f32 + 0.5,
                1.,
                point_info.multiplier.norm() as f32,
            )
            .into(),
        }
    }

    fn internal_potential(
        err: IterCount,
        tol: IterCount,
        mult_norm: IterCount,
        crit_degree: f64,
    ) -> IterCount
    {
        // Superattracting case
        // Assumes the first return map has local degree 2.
        // This could be improved to handle higher order critical points,
        // but we would need access to more information to estimate the order
        let potential = if mult_norm <= 1e-10 {
            2. * (err.log(tol)).log(crit_degree as IterCount) as IterCount
        }
        // Parabolic case
        else if (1. - mult_norm).abs() <= 1e-5 {
            err / tol
        } else {
            (err / tol).log(mult_norm) as IterCount
        };

        if !potential.is_finite() {
            return 0.2;
        }
        potential
    }

    fn relative_potential<D>(
        point_info: &PointInfoPeriodic<D>,
        tol: IterCount,
        crit_degree: f64,
    ) -> IterCount
    where
        D: Polar<Real>,
    {
        let n = IterCount::from(point_info.period);
        let k = IterCount::from(point_info.preperiod);

        let mult_norm = point_info.multiplier.norm();

        let potential =
            Self::internal_potential(point_info.final_error, tol, mult_norm, crit_degree);

        let val = k / n - potential;

        val.powi(2) * n
    }

    pub fn color_known_potential<D: Polar<Real>>(
        &self,
        palette: &Palette,
        info: &PointInfoKnownPotential<D>,
    ) -> Color32
    {
        let rescaled_potential = info.potential.powi(2) / info.period as f64;
        match self {
            Self::Solid => palette.in_color,
            Self::Period => palette
                .period_coloring
                .map(info.period as f32, PERIOD_LUMA_MODIFIER),
            Self::PeriodMultiplier => {
                let hue_id = info.period as f32;
                let luminosity_modifier = info.multiplier.norm() as f32;
                palette.period_coloring.map(hue_id, luminosity_modifier)
            }
            Self::Preperiod => palette.map_color32(rescaled_potential.floor()),
            Self::PreperiodPeriod { fill_rate } => {
                let luma = (rescaled_potential * fill_rate).tanh() as f32;
                palette.period_coloring.map(info.period as f32, luma)
            }
            Self::InternalPotential { .. } => palette.map_color32(rescaled_potential),
            Self::PotentialAndPeriod { fill_rate, .. } => {
                let n = IterCount::from(info.period);

                let coloring_rate =
                    Self::multiplier_coloring_rate(info.multiplier.norm(), *fill_rate);

                let luma = (coloring_rate * rescaled_potential).tanh() as f32;

                palette.period_coloring.map(n as f32, luma)
            }
            Self::Multiplier => Hsv::new(
                (info.multiplier.arg() / TAU) as f32 + 0.5,
                1.,
                info.multiplier.norm() as f32,
            )
            .into(),
        }
    }
}

impl Default for IncoloringAlgorithm
{
    fn default() -> Self
    {
        Self::PeriodMultiplier
    }
}
