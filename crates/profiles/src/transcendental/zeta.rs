use crate::macros::{degree_impl_transcendental, profile_imports};
use dynamo_color::{Coloring, IncoloringAlgorithm};
use dynamo_common::math_utils::{riemann_xi, riemann_xi_d, riemann_xi_d2};
use dynamo_core::dynamics::PlaneType;
profile_imports!();

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RiemannXi
{
    point_grid: PointGrid,
    max_iter: Period,
}
impl RiemannXi
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -30.,
        max_x: 30.,
        min_y: -30.,
        max_y: 30.,
    };
}
impl Default for RiemannXi
{
    fractal_impl!();
}
impl DynamicalFamily for RiemannXi
{
    type Param = Cplx;
    type Var = Cplx;
    type Deriv = Cplx;
    type MetaParam = NoParam;
    basic_plane_impl!();

    #[inline]
    fn map(&self, s: Self::Var, c: &Self::Param) -> Self::Var
    {
        riemann_xi(s) + c
    }
    #[inline]
    fn map_and_multiplier(&self, s: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv)
    {
        let [u, du] = riemann_xi_d(s);
        (u + c, du)
    }
    #[inline]
    fn gradient(&self, s: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        let (f, df) = self.map_and_multiplier(s, c);
        (f, df, ONE)
    }
    #[inline]
    fn param_map(&self, t: Cplx) -> Self::Param
    {
        t
    }
    #[inline]
    fn param_map_d(&self, t: Cplx) -> (Self::Param, Self::Deriv)
    {
        (t, ONE)
    }
    #[inline]
    fn start_point(&self, _point: Cplx, c: &Self::Param) -> Self::Var
    {
        *c
    }
    #[inline]
    fn start_point_d(&self, _t: Cplx, c: &Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        (*c, ZERO, ONE)
    }
    fn name(&self) -> String
    {
        "Riemann Xi".to_owned()
    }
}

impl FamilyDefaults for RiemannXi
{
    default_bounds!();

    #[inline]
    fn default_selection(&self) -> Cplx
    {
        ZERO
    }
}

impl HasChild<RiemannXiNewton> for RiemannXi
{
    fn to_child_param(
        param: Self::Param,
    ) -> <<RiemannXiNewton as DynamicalFamily>::MetaParam as ParamList>::Param
    {
        param
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
    const DEFAULT_BOUNDS: Bounds = Bounds::square(30., Cplx::new(0.500001, 0.));
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

impl DynamicalFamily for RiemannXiNewton
{
    type Var = Cplx;
    type Param = Cplx;
    type Deriv = Cplx;
    type MetaParam = ParamStack<NoParam, Cplx>;
    basic_plane_impl!();

    fn plane_type(&self) -> PlaneType
    {
        PlaneType::Dynamical
    }

    fn start_point(&self, s: Cplx, _c: &Self::Param) -> Self::Var
    {
        s
    }

    fn map(&self, s: Self::Var, c: &Self::Param) -> Self::Var
    {
        let [z, dz] = riemann_xi_d(s);
        s - (z + c) / dz
    }
    fn map_and_multiplier(&self, s: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv)
    {
        let [z, dz, d2z] = riemann_xi_d2(s);
        let z = z + c;
        (s - z / dz, z / d2z)
    }
    #[inline]
    fn gradient(&self, s: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        let [z, dz, d2z] = riemann_xi_d2(s);
        let z = z + c;
        (s - z / dz, z / d2z, -dz.inv())
    }
    fn param_map(&self, _point: Cplx) -> Self::Param
    {
        self.param
    }
    fn set_param(&mut self, value: <Self::MetaParam as ParamList>::Param)
    {
        self.param = value;
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

impl FamilyDefaults for RiemannXiNewton
{
    fn default_bounds(&self) -> Bounds
    {
        Bounds::square(30., Cplx::new(0.5, 0.))
    }

    #[inline]
    fn default_selection(&self) -> Cplx
    {
        ZERO
    }

    fn default_coloring(&self) -> dynamo_color::Coloring
    {
        Coloring::default().with_interior_algorithm(self.internal_potential_coloring())
    }
}

impl HasChild<Self> for RiemannXiNewton
{
    fn to_child_param(
        param: Self::Param,
    ) -> <<Self as DynamicalFamily>::MetaParam as ParamList>::Param
    {
        param
    }
}

impl MarkedPoints for RiemannXi {}
impl MarkedPoints for RiemannXiNewton {}

degree_impl_transcendental!(RiemannXi);
degree_impl_transcendental!(RiemannXiNewton);
