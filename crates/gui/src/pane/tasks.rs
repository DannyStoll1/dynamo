use dynamo_common::prelude::*;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::dialog::RayParams;

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
    pub orbit: OrbitTask,
    pub follow: FollowState,
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
            orbit: OrbitTask::Disabled,
            follow: FollowState::Idle,
        }
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
pub enum FollowState
{
    #[default]
    Idle,
    SelectRay
    {
        angle: RationalAngle, follow: bool
    },
    SelectPeriodic
    {
        orbit_schema: OrbitSchema,
        follow: bool,
    },
}

impl FollowState
{
    #[must_use]
    pub fn pop(&mut self) -> Self
    {
        match self {
            Self::SelectRay {
                angle,
                follow: false,
            } => {
                let angle = *angle;
                *self = Self::Idle;
                Self::SelectRay {
                    angle,
                    follow: false,
                }
            }
            Self::SelectPeriodic {
                orbit_schema,
                follow: false,
            } => {
                let orbit_schema = *orbit_schema;
                *self = Self::Idle;
                Self::SelectPeriodic {
                    orbit_schema,
                    follow: false,
                }
            }
            _ => *self,
        }
    }
}

impl std::fmt::Display for FollowState
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self {
            Self::SelectPeriodic {
                orbit_schema,
                follow: true,
            } => write!(
                f,
                "Following point of {orbit_schema}\n\
                [ESC] to stop."
            ),
            Self::SelectRay {
                angle,
                follow: true,
            } => write!(
                f,
                "Following landing point of ray at angle {angle}\n\
                [ESC] to stop."
            ),
            _ => write!(f, ""),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum OrbitTask
{
    Enabled,
    DrawOnce,
    SkipOnce,
    #[default]
    Disabled,
}

impl OrbitTask
{
    pub fn disable(&mut self)
    {
        *self = Self::Disabled;
    }
    pub fn enable(&mut self)
    {
        *self = Self::Enabled;
    }
    pub fn draw_once(&mut self)
    {
        *self = Self::DrawOnce;
    }
    pub fn skip(&mut self)
    {
        if matches!(self, Self::Enabled) {
            *self = Self::SkipOnce;
        }
    }
    pub fn pop(&mut self) -> bool
    {
        match self {
            Self::Disabled => false,
            Self::Enabled => true,
            Self::SkipOnce => {
                self.enable();
                false
            }
            Self::DrawOnce => {
                self.disable();
                true
            }
        }
    }
    #[must_use]
    pub const fn is_enabled(&self) -> bool
    {
        !matches!(self, Self::Disabled)
    }
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
    pub fn run_on<P: Pane + ?Sized>(&self, pane: &mut P, ray_params: &RayParams)
    {
        let angle_info = &ray_params.angle_info;
        if ray_params.include_orbit {
            for t in angle_info.orbit(pane.degree()) {
                pane.marking_mut().toggle_ray(t);
            }
        } else {
            pane.marking_mut().toggle_ray(angle_info.angle);
        }
        pane.schedule_redraw();
        match self {
            Self::Select => pane.set_follow_state(FollowState::SelectRay {
                angle: angle_info.angle,
                follow: false,
            }),
            Self::Follow => pane.set_follow_state(FollowState::SelectRay {
                angle: angle_info.angle,
                follow: true,
            }),
            Self::DoNothing => {}
        }
    }
}
