use crate::dynamics::DynamicalFamily;
use dynamo_common::prelude::*;
use num_traits::One;

pub mod distance_estimation;
pub mod floyd;
pub mod potential;
pub mod simple;

pub use distance_estimation::DistanceEstimation;
pub use floyd::CycleDetected;
pub use potential::Potential;
pub use simple::Simple;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum EscapeResult<V, D>
{
    Escaped
    {
        iters: Period,
        final_value: V,
    },
    Periodic
    {
        info: PointInfoPeriodic<D>,
        final_value: V,
    },
    Bounded(V),
    #[default]
    Unknown,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Info<P, V, D>
{
    pub param: P,
    pub start: V,
    pub result: PointInfo<D>,
}

#[derive(Clone, Debug)]
pub struct OrbitAndInfo<P, V, D>
{
    pub orbit: Vec<V>,
    pub info: Info<P, V, D>,
}

#[derive(Clone, Debug, Default)]
pub struct OrbitSummaryConf
{
    pub show_selection: bool,
    pub show_parameter: bool,
    pub show_start_point: bool,
    pub float_prec: usize,
}
impl OrbitSummaryConf
{
    #[must_use]
    pub const fn selection_conf(&self) -> DescriptionConf
    {
        DescriptionConf::new()
            .enabled()
            .with_visibility(self.show_parameter)
            .with_precision(self.float_prec)
    }
    #[must_use]
    pub const fn parameter_conf(&self) -> DescriptionConf
    {
        DescriptionConf::new()
            .enabled()
            .with_visibility(self.show_parameter)
            .with_precision(self.float_prec)
    }
    #[must_use]
    pub const fn start_point_conf(&self) -> DescriptionConf
    {
        DescriptionConf::new()
            .enabled()
            .with_visibility(self.show_start_point)
            .with_precision(self.float_prec)
    }
}

impl<P, V, D> Info<P, V, D>
where
    P: Describe,
    V: Describe,
    D: std::fmt::Display,
{
    pub fn summary(&self, conf: &OrbitSummaryConf) -> String
    {
        use PointInfo::*;

        let param_desc = self
            .param
            .describe(&conf.parameter_conf())
            .map(|d| format!("Parameter: {d}\n"))
            .unwrap_or_default();
        let start_desc = self
            .start
            .describe(&conf.start_point_conf())
            .map(|d| format!("Start: {d}\n"))
            .unwrap_or_default();

        let result_summary = match &self.result {
            Escaping {
                potential,
                phase: None,
            } => format!("Escaped, potential: {potential:.DISPLAY_PREC$}"),
            Escaping {
                potential,
                phase: Some(p),
            } => format!("Escaped with phase {p}, potential: {potential:.DISPLAY_PREC$}"),
            DistanceEstimate { distance, phase } => {
                format!("Escaped with phase {phase}, est. distance: {distance:.DISPLAY_PREC$}")
            }
            Periodic(data) | MarkedPoint { data, .. } => data.to_string(),
            PeriodicKnownPotential(data) => data.to_string(),
            Bounded => "Bounded (no cycle detected or period too high)".to_owned(),
            Wandering => "Wandering (appears to escape very slowly)".to_owned(),
            Unknown => {
                "Unknown result, likely due to insufficient floting-point precision".to_owned()
            }
        };
        format!(
            "{start_desc}\
            {param_desc}\
            {result_summary}",
        )
    }
}

pub trait Orbit: Send
{
    type Outcome;

    /// Re-initialize aan orbit.
    fn reset(&mut self, selection: Cplx);

    fn run_until_complete(&mut self) -> Self::Outcome;
}
