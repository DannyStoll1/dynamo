use crate::macros::*;
profile_imports!();

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RiemannXi {
    point_grid: PointGrid,
    max_iter: Period,
}
