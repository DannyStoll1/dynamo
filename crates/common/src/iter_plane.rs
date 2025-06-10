use crate::point_grid::PointGrid;

use crate::point_info::PointInfo;
use ndarray::Array2;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IterPlane<D>
{
    #[cfg_attr(feature = "serde", serde(skip))]
    pub iter_counts: Array2<PointInfo<D>>,
    pub point_grid: PointGrid,
}

impl<D> IterPlane<D>
where
    D: Clone,
{
    #[must_use]
    pub fn create(point_grid: PointGrid) -> Self
    {
        let iter_counts = Array2::from_elem(point_grid.shape(), PointInfo::Bounded);
        Self {
            iter_counts,
            point_grid,
        }
    }

    #[must_use]
    pub fn fill(&mut self, value: PointInfo<D>)
    {
        self.iter_counts.fill(value);
    }
}
