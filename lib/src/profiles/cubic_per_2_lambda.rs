use crate::macros::*;
use derive_more::{Add, Display, From};
profile_imports!();

#[derive(Default, Clone, Copy, Debug, Add, From, Display)]
#[display(fmt = "[ a: {}, b: {} ] ", a, b)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ComplexPair
{
    a: ComplexNum,
    b: ComplexNum,
}
// Unused
impl From<ComplexNum> for ComplexPair
{
    fn from(t: ComplexNum) -> Self
    {
        Self { a: t, b: ZERO }
    }
}
impl From<ComplexPair> for ComplexNum
{
    fn from(c: ComplexPair) -> Self
    {
        let disc = (3. * c.a * (c.a + 1.) + c.b * c.b).sqrt();
        (c.b + disc) / (3. * c.a)
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CubicPer2Lambda
{
    point_grid: PointGrid,
    max_iter: Period,
    multiplier: ComplexNum,
}

impl CubicPer2Lambda
{
    const DEFAULT_BOUNDS: Bounds = Bounds::centered_square(2.5);
}

impl Default for CubicPer2Lambda
{
    fractal_impl!(multiplier, ZERO);
}

impl ParameterPlane for CubicPer2Lambda
{
    parameter_plane_impl!(ComplexNum, ComplexPair, ComplexNum, ComplexNum);

    #[inline]
    fn map(&self, z: Self::Var, c: Self::Param) -> Self::Var
    {
        horner!(z, c.b, -(1. + c.a), -c.b, c.a)
    }

    #[inline]
    fn map_and_multiplier(
        &self,
        z: Self::Var,
        ComplexPair { a, b }: Self::Param,
    ) -> (Self::Var, Self::Deriv)
    {
        let x1 = -a - 1.;
        (horner!(z, b, x1, -b, a), horner!(z, x1, -(b + b), 3. * a))
    }

    #[inline]
    fn dynamical_derivative(&self, z: Self::Var, ComplexPair { a, b }: Self::Param) -> Self::Deriv
    {
        horner!(z, -a - 1., -(b + b), 3. * a)
    }

    #[inline]
    fn parameter_derivative(&self, z: Self::Var, _c: Self::Param) -> Self::Deriv
    {
        z * (1. + z * z)
    }

    #[inline]
    fn start_point(&self, _m: ComplexNum, ComplexPair { a, b }: Self::Param) -> Self::Var
    {
        let disc = (3. * a * (a + 1.) + b * b).sqrt();
        (b + disc) / (3. * a)
    }

    fn critical_points_child(&self, c: Self::Param) -> Vec<Self::Var>
    {
        let disc = (3. * c.a * (c.a + 1.) + c.b * c.b).sqrt();
        let denom = 3. * c.a;
        vec![(c.b + disc) / denom, (c.b - disc) / denom]
    }

    fn get_param(&self) -> Self::MetaParam
    {
        self.multiplier
    }

    fn set_meta_param(&mut self, value: Self::MetaParam)
    {
        self.multiplier = value
    }

    fn set_param(&mut self, value: Self::MetaParam)
    {
        self.multiplier = value
    }

    fn param_map(&self, m: ComplexNum) -> Self::Param
    {
        let s = (1. - self.get_param()) / 4.;
        let m2 = m * m;
        let denom = m + m + 1.;
        ComplexPair {
            a: (s - m2) / denom,
            b: (m2 + m + s) / denom,
        }
    }

    fn name(&self) -> String
    {
        format!("Cubic Per(2, {})", self.multiplier)
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CubicPer2LambdaParam
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl CubicPer2LambdaParam
{
    fractal_impl!(-2.5, 2.5, -2.5, 2.5);
}

impl ParameterPlane for CubicPer2LambdaParam
{
    type Param = ComplexNum;
    type MetaParam = NoParam;
    type Var = ComplexNum;
    type Deriv = ComplexNum;
    type Child = CubicPer2Lambda;

    basic_plane_impl!();
    basic_escape_encoding!(3., 1.);

    #[inline]
    fn map(&self, z: Self::Var, a: Self::Param) -> Self::Var
    {
        let z2 = z * z;
        z * (z2 + a)
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, a: Self::Param) -> (Self::Var, Self::Deriv)
    {
        let z2 = z * z;
        let u = z2 + a;
        (z * u, u + z * (a + z + z))
    }

    #[inline]
    fn dynamical_derivative(&self, z: Self::Var, a: Self::Param) -> Self::Deriv
    {
        let z2 = z * z;
        let u = z2 + a;
        u + z * (a + z + z)
    }

    #[inline]
    fn parameter_derivative(&self, z: Self::Var, _c: Self::Param) -> Self::Deriv
    {
        z
    }

    #[inline]
    fn start_point(&self, _point: ComplexNum, c: Self::Param) -> Self::Var
    {
        -ONE_THIRD * (c + c)
    }

    fn critical_points_child(&self, _param: Self::Param) -> Vec<Self::Var>
    {
        vec![ZERO, ONE]
    }

    fn name(&self) -> String
    {
        "Cubic Per(2, lambda) lambda-plane".to_owned()
    }

    fn default_selection(&self) -> ComplexNum
    {
        ZERO
    }

    fn default_julia_bounds(&self, point: ComplexNum, _param: Self::Param) -> Bounds
    {
        let r = 3.5 / (point.norm() + 0.01);
        Bounds::centered_square(r)
    }
}

impl From<CubicPer2LambdaParam> for CubicPer2Lambda
{
    fn from(parent: CubicPer2LambdaParam) -> Self
    {
        let point = parent.default_selection();
        let param = parent.param_map(point);
        let point_grid = parent
            .point_grid()
            .new_with_same_height(parent.default_julia_bounds(point, param));
        Self {
            point_grid,
            max_iter: parent.max_iter(),
            multiplier: param,
        }
    }
}
