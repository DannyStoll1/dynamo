use dynamo_common::prelude::*;

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
        if matches!(self, Self::DoNothing)
        {
            *self = Self::Rerun;
        }
    }
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
    Following(RationalAngle),
}
