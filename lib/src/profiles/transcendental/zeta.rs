use crate::{
    macros::*,
    math_utils::{riemann_xi, riemann_xi_d, riemann_xi_d2},
};
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
impl ParameterPlane for RiemannXi
{
    type Param = Cplx;
    type Var = Cplx;
    type Deriv = Cplx;
    type MetaParam = NoParam;
    type Child = RiemannXiNewton;
    basic_plane_impl!();

    fn map(&self, s: Self::Var, c: Self::Param) -> Self::Var
    {
        riemann_xi(s) + c
    }
    fn dynamical_derivative(&self, s: Self::Var, _c: Self::Param) -> Self::Deriv
    {
        riemann_xi_d(s)[1]
    }
    fn parameter_derivative(&self, _z: Self::Var, _c: Self::Param) -> Self::Deriv
    {
        ONE
    }
    fn param_map(&self, point: Cplx) -> Self::Param
    {
        point
    }
    fn default_selection(&self) -> Cplx
    {
        ZERO
    }
    fn default_julia_bounds(&self, _point: Cplx, _param: Self::Param) -> Bounds
    {
        Bounds::square(30., Cplx::new(0.5, 0.))
    }
    fn name(&self) -> String
    {
        "Riemann Xi".to_owned()
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RiemannXiNewton
{
    point_grid: PointGrid,
    max_iter: Period,
    param: Cplx,
}
impl RiemannXiNewton
{
    const DEFAULT_BOUNDS: Bounds = Bounds::square(30., Cplx::new(0.5, 0.));
}
impl Default for RiemannXiNewton
{
    fractal_impl!(param, ZERO);
}
impl From<RiemannXi> for RiemannXiNewton
{
    fn from(plane: RiemannXi) -> Self
    {
        Self {
            point_grid: plane.point_grid.clone(),
            max_iter: plane.max_iter,
            param: plane.default_selection(),
        }
    }
}

impl ParameterPlane for RiemannXiNewton
{
    type Var = Cplx;
    type Param = Cplx;
    type Deriv = Cplx;
    type MetaParam = ParamStack<NoParam, Cplx>;
    type Child = Self;
    basic_plane_impl!();

    fn is_dynamical(&self) -> bool
    {
        true
    }

    fn start_point(&self, s: Cplx, c: Self::Param) -> Self::Var
    {
        s
    }

    fn map(&self, s: Self::Var, c: Self::Param) -> Self::Var
    {
        let [z, dz] = riemann_xi_d(s);
        s - (z + c) / dz
    }
    fn map_and_multiplier(&self, s: Self::Var, c: Self::Param) -> (Self::Var, Self::Deriv)
    {
        let [z, dz, d2z] = riemann_xi_d2(s);
        let z = z + c;
        (s - z / dz, z / d2z)
    }
    fn dynamical_derivative(&self, s: Self::Var, _c: Self::Param) -> Self::Deriv
    {
        let [z, _, d2z] = riemann_xi_d2(s);
        z / d2z
    }
    fn parameter_derivative(&self, _z: Self::Var, _c: Self::Param) -> Self::Deriv
    {
        ONE
    }
    fn param_map(&self, _point: Cplx) -> Self::Param
    {
        self.param
    }
    fn default_selection(&self) -> Cplx
    {
        ZERO
    }
    fn default_julia_bounds(&self, _point: Cplx, _param: Self::Param) -> Bounds
    {
        Bounds::centered_square(30.)
    }
    fn set_param(&mut self, value: <Self::MetaParam as ParamList>::Param)
    {
        self.param = value
    }
    fn get_param(&self) -> <Self::MetaParam as ParamList>::Param
    {
        self.param
    }
    fn name(&self) -> String
    {
        "Riemann Xi Newton".to_owned()
    }
}
