use crate::{macros::*, math_utils::{riemann_xi, riemann_xi_d}};
profile_imports!();

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RiemannXi
{
    point_grid: PointGrid,
    max_iter: Period,
}
impl Default for RiemannXi
{
    fractal_impl!(-30., 30., -30., 30.);
}
impl ParameterPlane for RiemannXi {
    parameter_plane_impl!();

    fn map(&self, s: Self::Var, c: Self::Param) -> Self::Var {
        riemann_xi(s) + c
    }
    fn dynamical_derivative(&self, s: Self::Var, _c: Self::Param) -> Self::Deriv {
        riemann_xi_d(s).1
    }
    fn parameter_derivative(&self, _z: Self::Var, _c: Self::Param) -> Self::Deriv {
        ONE
    }
    fn param_map(&self, point: Cplx) -> Self::Param {
        point
    }
    fn default_selection(&self) -> Cplx {
        ZERO
    }
    fn default_julia_bounds(&self, _point: Cplx, _param: Self::Param) -> Bounds {
        Bounds::centered_square(30.)
    }
    fn name(&self) -> String {
        "Riemann Xi".to_owned()
    }
}
