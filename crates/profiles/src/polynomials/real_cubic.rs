use crate::macros::{
    basic_plane_impl, default_bounds, default_name, degree_impl, fractal_impl, profile_imports,
};
profile_imports!();

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RealCubicRealCrit
{
    point_grid: PointGrid,
    compute_mode: ComputeMode,
    max_iter: IterCount,
}

impl RealCubicRealCrit
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -2.5,
        max_x: 2.5,
        min_y: -2.5,
        max_y: 2.5,
    };
}
impl Default for RealCubicRealCrit
{
    fractal_impl!();
}

#[allow(clippy::suboptimal_flops)]
impl DynamicalFamily for RealCubicRealCrit
{
    type Param = RealPair;
    type Var = Cplx;
    type Deriv = Cplx;
    type MetaParam = NoParam;
    basic_plane_impl!();
    default_name!();

    // Critical point = a
    fn map(&self, z: Self::Var, RealPair { a, b }: &Self::Param) -> Self::Var
    {
        b + z * (z.powi(2) - 3. * a.powi(2))
    }

    #[inline]
    fn map_and_multiplier(
        &self,
        z: Self::Var,
        RealPair { a, b }: &Self::Param,
    ) -> (Self::Var, Self::Deriv)
    {
        let z2 = z.powi(2);
        let a2 = a.powi(2);
        (b + z * (z2 - 3. * a2), 3. * (z2 - a2))
    }

    fn start_point(&self, _point: Cplx, c: &Self::Param) -> Self::Var
    {
        c.a.into()
    }

    fn param_map(&self, point: Cplx) -> Self::Param
    {
        RealPair {
            a: point.re,
            b: point.im,
        }
    }
}

impl FamilyDefaults for RealCubicRealCrit
{
    default_bounds!();
}

impl HasJulia for RealCubicRealCrit {}

impl MarkedPoints for RealCubicRealCrit
{
    fn critical_points_child(&self, c: &Self::Param) -> Vec<Self::Var>
    {
        vec![c.a.into(), (-c.a).into()]
    }
}

degree_impl!(RealCubicRealCrit, 3);
degree_impl!(RealCubicImagCrit, 3);

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RealCubicImagCrit
{
    point_grid: PointGrid,
    compute_mode: ComputeMode,
    max_iter: IterCount,
}

impl RealCubicImagCrit
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -2.5,
        max_x: 2.5,
        min_y: -2.5,
        max_y: 2.5,
    };
}
impl Default for RealCubicImagCrit
{
    fractal_impl!();
}

#[allow(clippy::suboptimal_flops)]
impl DynamicalFamily for RealCubicImagCrit
{
    type Param = RealPair;
    type Var = Cplx;
    type Deriv = Cplx;
    type MetaParam = NoParam;
    basic_plane_impl!();
    default_name!();

    // Critical point = ai
    fn map(&self, z: Self::Var, RealPair { a, b }: &Self::Param) -> Self::Var
    {
        b + z * (z.powi(2) + 3. * a.powi(2))
    }

    #[inline]
    fn map_and_multiplier(
        &self,
        z: Self::Var,
        RealPair { a, b }: &Self::Param,
    ) -> (Self::Var, Self::Deriv)
    {
        let z2 = z.powi(2);
        let a2 = a.powi(2);
        (b + z * (z2 + 3. * a2), 3. * (z2 + a2))
    }

    fn start_point(&self, _point: Cplx, c: &Self::Param) -> Self::Var
    {
        Cplx::new(0., c.a)
    }

    fn param_map(&self, point: Cplx) -> Self::Param
    {
        RealPair {
            a: point.re,
            b: point.im,
        }
    }
}

impl FamilyDefaults for RealCubicImagCrit
{
    default_bounds!();
}

impl HasJulia for RealCubicImagCrit {}

impl MarkedPoints for RealCubicImagCrit
{
    fn critical_points_child(&self, c: &Self::Param) -> Vec<Self::Var>
    {
        let crit = Cplx::new(0., c.a);
        vec![crit, -crit]
    }
}
