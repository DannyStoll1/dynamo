use crate::macros::*;
profile_imports!();

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RealCubicRealCrit
{
    point_grid: PointGrid,
    max_iter: Period,
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

impl ParameterPlane for RealCubicRealCrit
{
    type Param = RealPair;
    type Var = Cplx;
    type Deriv = Cplx;
    type MetaParam = NoParam;
    type Child = JuliaSet<Self>;
    basic_plane_impl!();
    default_name!();
    default_bounds!();
    basic_escape_encoding!(3.);

    // Critical point = a
    fn map(&self, z: Self::Var, RealPair { a, b }: Self::Param) -> Self::Var
    {
        b + z * (z * z - 3. * a * a)
    }

    fn dynamical_derivative(&self, z: Self::Var, RealPair { a, .. }: Self::Param) -> Self::Deriv
    {
        3. * (z * z - a * a)
    }

    #[inline]
    fn map_and_multiplier(
        &self,
        z: Self::Var,
        RealPair { a, b }: Self::Param,
    ) -> (Self::Var, Self::Deriv)
    {
        let z2 = z * z;
        let a2 = a * a;
        (b + z * (z2 - 3. * a2), 3. * (z2 - a2))
    }

    fn parameter_derivative(&self, z: Self::Var, c: Self::Param) -> Self::Deriv
    {
        -6. * c.a * z
    }

    fn critical_points_child(&self, c: Self::Param) -> Vec<Self::Var>
    {
        vec![c.a.into(), (-c.a).into()]
    }

    fn start_point(&self, _point: Cplx, c: Self::Param) -> Self::Var
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

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RealCubicImagCrit
{
    point_grid: PointGrid,
    max_iter: Period,
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

impl ParameterPlane for RealCubicImagCrit
{
    type Param = RealPair;
    type Var = Cplx;
    type Deriv = Cplx;
    type MetaParam = NoParam;
    type Child = JuliaSet<Self>;
    basic_plane_impl!();
    default_name!();
    default_bounds!();
    basic_escape_encoding!(3.);

    // Critical point = ai
    fn map(&self, z: Self::Var, RealPair { a, b }: Self::Param) -> Self::Var
    {
        b + z * (z * z + 3. * a * a)
    }

    fn dynamical_derivative(&self, z: Self::Var, RealPair { a, .. }: Self::Param) -> Self::Deriv
    {
        3. * (z * z + a * a)
    }

    #[inline]
    fn map_and_multiplier(
        &self,
        z: Self::Var,
        RealPair { a, b }: Self::Param,
    ) -> (Self::Var, Self::Deriv)
    {
        let z2 = z * z;
        let a2 = a * a;
        (b + z * (z2 + 3. * a2), 3. * (z2 + a2))
    }

    fn parameter_derivative(&self, z: Self::Var, c: Self::Param) -> Self::Deriv
    {
        6. * c.a * z
    }

    fn critical_points_child(&self, c: Self::Param) -> Vec<Self::Var>
    {
        let crit = Cplx::new(0., c.a);
        vec![crit, -crit]
    }

    fn start_point(&self, _point: Cplx, c: Self::Param) -> Self::Var
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