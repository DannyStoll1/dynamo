use std::sync::atomic::{AtomicU8, Ordering};

use crossbeam::channel::Receiver;
use dynamo_common::prelude::PointInfo;

#[repr(transparent)]
#[derive(Default, Debug)]
struct ComputeState(AtomicU8);
impl ComputeState
{
    const IDLE: u8 = 0;
    const RUNNING: u8 = 1;
    const ABORTING: u8 = 2;
    const FINISHED: u8 = 3;

    const fn from_u8(val: u8) -> Self
    {
        Self(AtomicU8::new(val))
    }

    pub const fn idle() -> Self
    {
        Self::from_u8(Self::IDLE)
    }

    pub fn abort(&self, success: Ordering, failure: Ordering) -> Result<Self, Self>
    {
        self.0
            .compare_exchange(Self::RUNNING, Self::ABORTING, success, failure)
            .map(Self::from_u8)
            .map_err(Self::from_u8)
    }

    pub fn wait_for(&self, state: u8)
    {
        while self.0.load(Ordering::Acquire) != state {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
}

pub type Job<D> = Option<Receiver<((usize, usize), PointInfo<D>)>>;

// #[derive(Debug)]
// pub struct Job<T>
// {
//     state: ComputeState,
//     rx: Receiver<T>,
// }
// impl<T> Job<T>
// {
//     pub const fn from_rx(rx: Receiver<T>) -> Self
//     {
//         Self {
//             state: ComputeState::idle(),
//             rx,
//         }
//     }
// }
