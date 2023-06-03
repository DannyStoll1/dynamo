use crate::{macros::*, math_utils::solve_cubic, types::param_stack::Summarize};
use derive_more::{Add, Display, From};
profile_imports!();

#[derive(Default, Clone, Copy, Debug, Add, From, PartialEq, Display)]
#[display(fmt = "[ a: {}, b: {} ] ", a, b)]
pub struct ComplexPair
{
    pub a: ComplexNum,
    pub b: ComplexNum,
}

impl Summarize for ComplexPair {}

impl From<ComplexNum> for ComplexPair
{
    fn from(z: ComplexNum) -> Self
    {
        todo!()
    }
}

impl From<ComplexPair> for ComplexNum
{
    fn from(value: ComplexPair) -> Self
    {
        todo!()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CubicPer3_0
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl Default for CubicPer3_0
{
    fractal_impl!(-7.5, 2.5, -5., 5.);
}

impl ParameterPlane for CubicPer3_0
{
    type Var = ComplexNum;
    type Param = ComplexPair;
    type MetaParam = NoParam;
    type Deriv = ComplexNum;
    type Child = JuliaSet<Self>;
    basic_plane_impl!();
    default_name!();
    basic_escape_encoding!(3., 1.);

    fn map_and_multiplier(
        &self,
        z: Self::Var,
        ComplexPair { a, b }: Self::Param,
    ) -> (Self::Var, Self::Deriv)
    {
        let z2 = z * z;
        (1. + z2 * (b + a * z), z * (3. * a * z + b + b))
    }

    fn map(&self, z: Self::Var, ComplexPair { a, b }: Self::Param) -> Self::Var
    {
        horner!(z, 1., 0., b, a)
    }

    fn dynamical_derivative(&self, z: Self::Var, ComplexPair { a, b }: Self::Param) -> Self::Deriv
    {
        z * (3. * a + b + b)
    }

    fn parameter_derivative(&self, z: Self::Var, _c: Self::Param) -> Self::Deriv
    {
        z * z * z
    }
    fn param_map(&self, t: ComplexNum) -> Self::Param
    {
        let u = t + 1.;
        let u2_inv = (u * u).inv();
        let a = 1. - t - 3. / u + u2_inv;
        let b = t * t * t * u2_inv + u / t;
        ComplexPair { a, b }
    }
    fn start_point(&self, _point: ComplexNum, ComplexPair { a, b }: Self::Param) -> Self::Var
    {
        -(b + b) / (3. * a)
    }
    fn critical_points_child(&self, ComplexPair { a, b }: Self::Param) -> Vec<Self::Var>
    {
        vec![ZERO, -(b + b) / (3. * a)]
    }
    fn critical_points(&self) -> Vec<Self::Var>
    {
        let [r0, r1, r2] = solve_cubic(ONE, 2.0.into(), ONE);
        vec![ZERO, (-1.).into(), r0, r1, r2]
    }
    fn default_selection(&self) -> ComplexNum
    {
        // ComplexNum::new(-3.34447065821736, 0.) // center of a capture component; c1 -2> c0=0 -2> 1 -> a+b+1 -> 0
        ComplexNum::new(-0.521257806222939, 0.) // center of a period 1 component; c1 -2> c1
    }
    fn default_julia_bounds(&self, point: ComplexNum, c: Self::Param) -> Bounds
    {
        let crit = self.start_point(point, c);
        // let center = (crit + 2. + c.a + c.b) * ONE_THIRD;
        // Centroid of free critical point and marked critical orbit
        let center = (crit + 2. + c.a + c.b) * 0.25;
        let radius = crit.norm() + (c.a + c.b + 1.).norm() + 0.5;
        Bounds::square(radius, center)
    }
}
