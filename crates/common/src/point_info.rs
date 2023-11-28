use crate::globals::DISPLAY_PREC;
use crate::types::{IterCountSmooth, Period, Real};
use std::fmt::Display;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PointInfo<D>
{
    Escaping
    {
        potential: IterCountSmooth,
        phase: Option<Period>,
    },
    Periodic(PointInfoPeriodic<D>),
    PeriodicKnownPotential(PointInfoKnownPotential<D>),
    #[default]
    Bounded,
    Wandering,
    MarkedPoint
    {
        data: PointInfoPeriodic<D>,
        class_id: PointClassId,
        num_point_classes: usize,
    },
    DistanceEstimate
    {
        distance: Real,
        phase: Period,
    },
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PointInfoPeriodic<D>
{
    pub preperiod: Period,
    pub period: Period,
    pub multiplier: D,
    pub final_error: Real,
}
impl<D> std::fmt::Display for PointInfoPeriodic<D>
where
    D: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(
            f,
            "Cycle detected after {preperiod} iterations.\n\
                Period: {period}\n\
                Multiplier: {multiplier:.DISPLAY_PREC$}",
            preperiod = self.preperiod,
            period = self.period,
            multiplier = self.multiplier
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PointInfoKnownPotential<D>
{
    pub period: Period,
    pub multiplier: D,
    pub potential: IterCountSmooth,
}
impl<D> std::fmt::Display for PointInfoKnownPotential<D>
where
    D: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(
            f,
            "Cycle detected.\n
                Period: {period}\n\
                Multiplier: {multiplier:.DISPLAY_PREC$}\n\
                Potential: {potential:.DISPLAY_PREC$}",
            period = self.period,
            multiplier = self.multiplier,
            potential = self.potential,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PointClassId(pub u8);

impl From<usize> for PointClassId
{
    fn from(n: usize) -> Self
    {
        Self(n as u8)
    }
}

impl From<PointClassId> for f32
{
    fn from(id: PointClassId) -> Self
    {
        Self::from(id.0)
    }
}
