use crate::globals::DISPLAY_PREC;
use crate::types::{IterCount, Period, Real};
use std::fmt::Display;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PointInfo<V, D>
{
    Escaping
    {
        potential: IterCount,
    },
    Periodic
    {
        data: PointInfoPeriodic<V, D>,
    },
    #[default]
    Bounded,
    Wandering,
    MarkedPoint
    {
        data: PointInfoPeriodic<V, D>,
        class_id: PointClassId,
        num_point_classes: usize,
    },
}
impl<V, P, D> std::fmt::Display for OrbitInfo<V, P, D>
where
    V: std::fmt::Display,
    P: std::fmt::Display,
    D: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        use PointInfo::{Bounded, Escaping, MarkedPoint, Periodic, Wandering};
        let result_summary = match &self.result
        {
            Escaping { potential } => format!("Escaped, potential: {potential:.DISPLAY_PREC$}"),
            Periodic { data } => data.to_string(),
            Bounded => "Bounded (no cycle detected or period too high)".to_owned(),
            Wandering => "Wandering (appears to escape very slowly)".to_owned(),
            MarkedPoint { data, .. } => data.to_string(),
        };
        write!(
            f,
            "Parameter: {:.*}\nStarting point: {:.*}\n{}",
            DISPLAY_PREC, self.param, DISPLAY_PREC, self.start, result_summary
        )
    }
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct OrbitInfo<V, P, D>
{
    pub start: V,
    pub param: P,
    pub result: PointInfo<V, D>,
}
pub struct OrbitAndInfo<V, P, D>
{
    pub orbit: Vec<V>,
    pub info: OrbitInfo<V, P, D>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PointInfoPeriodic<V, D>
{
    pub value: V,
    pub preperiod: Period,
    pub period: Period,
    pub multiplier: D,
    pub final_error: Real,
}
impl<V, D> ToString for PointInfoPeriodic<V, D>
where
    V: Display,
    D: Display,
{
    fn to_string(&self) -> String
    {
        format!(
            "Cycle detected after {preperiod} iterations.\n\
                Period: {period}\n\
                Multiplier: {multiplier:.DISPLAY_PREC$}",
            preperiod = self.preperiod,
            period = self.period,
            multiplier = self.multiplier
        )
    }
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum EscapeState<V, D>
{
    Escaped
    {
        iters: Period,
        final_value: V,
    },
    Periodic
    {
        data: PointInfoPeriodic<V, D>,
    },
    NotYetEscaped,
    Bounded,
}
impl<V, D> Default for EscapeState<V, D>
{
    fn default() -> Self
    {
        Self::NotYetEscaped
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
