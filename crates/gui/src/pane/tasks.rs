use dynamo_common::prelude::*;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::Pane;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum RepeatableTask
{
    #[default]
    DoNothing,
    Rerun,
    InitRun,
}
impl RepeatableTask
{
    pub fn schedule_init_run(&mut self)
    {
        *self = Self::InitRun;
    }
    pub fn schedule_rerun(&mut self)
    {
        if matches!(self, Self::DoNothing) {
            *self = Self::Rerun;
        }
    }
    #[must_use]
    pub fn pop(&mut self) -> Self
    {
        std::mem::take(self)
    }
    pub fn clear(&mut self)
    {
        *self = Self::DoNothing;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PaneTasks
{
    pub compute: RepeatableTask,
    pub draw: RepeatableTask,
}

impl PaneTasks
{
    #[must_use]
    pub const fn init_tasks() -> Self
    {
        let task = RepeatableTask::InitRun;
        Self {
            compute: task,
            draw: task,
        }
    }
    #[must_use]
    pub fn pop(&mut self) -> Self
    {
        let compute = self.compute.pop();
        let draw = self.draw.pop();
        Self { compute, draw }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ResizeTask
{
    #[default]
    DoNothing,
    ShowDialog,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ChildTask
{
    #[default]
    Idle,
    UpdateParam,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum RayState
{
    #[default]
    Idle,
    SelectOnce(RationalAngle),
    Following(RationalAngle),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SelectOrFollow
{
    Select,
    Follow,
    #[default]
    DoNothing,
}
impl SelectOrFollow
{
    pub fn run_on<P: Pane + ?Sized>(&self, pane: &mut P, angle: RationalAngle)
    {
        pane.marking_mut().toggle_ray(angle);
        pane.schedule_redraw();
        match self {
            Self::Select => {
                pane.select_ray_landing_point(angle);
            }
            Self::Follow => {
                pane.follow_ray_landing_point(angle);
            }
            Self::DoNothing => {}
        }
    }
}
