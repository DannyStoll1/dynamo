use ndarray::Array2;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::point_grid::PointGrid;
use crate::point_info::PointInfo;

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IterPlane<D>
{
    #[cfg_attr(feature = "serde", serde(skip))]
    pub iter_counts: Array2<PointInfo<D>>,
    pub point_grid:  PointGrid,
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

    /// Integer resolution ratio between this plane and a coarser one, if the
    /// coarse plane embeds exactly into this one along both axes.
    ///
    /// Returns `None` when the resolutions are not an exact common multiple, in
    /// which case coarse samples do not coincide with this plane's samples and
    /// copy-forward would be invalid.
    #[must_use]
    pub fn embedding_scale(&self, coarse: &Self) -> Option<usize>
    {
        let (fine_x, fine_y) = self.point_grid.shape();
        let (coarse_x, coarse_y) = coarse.point_grid.shape();
        if coarse_x == 0 || coarse_y == 0 {
            return None;
        }
        let scale = fine_x / coarse_x;
        (scale >= 1 && fine_x == coarse_x * scale && fine_y == coarse_y * scale).then_some(scale)
    }

    /// Copy every cell of a coarser plane into the coinciding cell of this one.
    ///
    /// The coarse plane must embed exactly into this plane (see
    /// [`embedding_scale`](Self::embedding_scale)); otherwise this is a no-op and
    /// returns `None`. On success it returns the resolution ratio, which pairs
    /// with [`lifted_pixel`](Self::lifted_pixel) to skip recomputing the carried
    /// cells.
    pub fn lift_from_coarser(&mut self, coarse: &Self) -> Option<usize>
    {
        let scale = self.embedding_scale(coarse)?;
        for ((cx, cy), info) in coarse.iter_counts.indexed_iter() {
            self.iter_counts[(cx * scale, cy * scale)] = info.clone();
        }
        Some(scale)
    }

    /// Whether the pixel `(x, y)` was carried in from a coarser plane at the
    /// given resolution ratio, and so should not be recomputed.
    #[must_use]
    pub const fn lifted_pixel(x: usize, y: usize, scale: usize) -> bool
    {
        x.is_multiple_of(scale) && y.is_multiple_of(scale)
    }
}

#[cfg(test)]
mod tests
{
    use ndarray::Array2;

    use super::*;
    use crate::point_grid::Bounds;

    fn grid(res_x: usize, res_y: usize) -> PointGrid
    {
        PointGrid::new(
            res_x,
            res_y,
            Bounds {
                min_x: 0.,
                max_x: 1.,
                min_y: 0.,
                max_y: 1.,
            },
        )
    }

    fn plane_from(res_x: usize, res_y: usize, fill: impl Fn(usize, usize) -> u32)
    -> IterPlane<u32>
    {
        let iter_counts = Array2::from_shape_fn((res_x, res_y), |(x, y)| PointInfo::Escaping {
            potential: f64::from(fill(x, y)),
            phase:     None,
        });
        IterPlane {
            iter_counts,
            point_grid: grid(res_x, res_y),
        }
    }

    #[test]
    fn embedding_scale_detects_exact_multiple()
    {
        let fine = IterPlane::<u32>::create(grid(256, 256));
        let coarse = IterPlane::<u32>::create(grid(64, 64));
        assert_eq!(fine.embedding_scale(&coarse), Some(4));
    }

    #[test]
    fn embedding_scale_rejects_inexact()
    {
        let fine = IterPlane::<u32>::create(grid(200, 200));
        let coarse = IterPlane::<u32>::create(grid(64, 64));
        assert_eq!(fine.embedding_scale(&coarse), None);
    }

    #[test]
    fn lift_copies_coarse_cells_into_aligned_slots()
    {
        let coarse = plane_from(2, 2, |x, y| (10 * x + y) as u32);
        let mut fine = IterPlane::<u32>::create(grid(4, 4));
        let scale = fine.lift_from_coarser(&coarse).unwrap();
        assert_eq!(scale, 2);

        // Carried cells hold the coarse value; others are still default Bounded.
        for ((x, y), info) in fine.iter_counts.indexed_iter() {
            if IterPlane::<u32>::lifted_pixel(x, y, scale) {
                let expected = &coarse.iter_counts[(x / scale, y / scale)];
                assert_eq!(info, expected);
            } else {
                assert_eq!(*info, PointInfo::Bounded);
            }
        }
    }

    #[test]
    fn lift_is_noop_on_inexact_embedding()
    {
        let coarse = plane_from(3, 3, |_, _| 7);
        let mut fine = IterPlane::<u32>::create(grid(4, 4));
        assert_eq!(fine.lift_from_coarser(&coarse), None);
        assert!(
            fine.iter_counts
                .iter()
                .all(|info| *info == PointInfo::Bounded)
        );
    }
}
