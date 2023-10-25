use crate::point_grid::PointGrid;

use crate::point_info::PointInfo;
use ndarray::Array2;

#[derive(Clone)]
pub struct IterPlane<D>
{
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
}

