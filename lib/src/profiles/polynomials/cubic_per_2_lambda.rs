use crate::{
    macros::*,
    math_utils::{solve_cubic, solve_quartic, weierstrass_p},
    types::param_stack::Summarize,
};
use derive_more::{Add, Display, From};
profile_imports!();

#[derive(Default, Clone, Copy, Debug, Add, From, PartialEq, Display)]
#[display(fmt = "[ a: {}, b: {} ] ", a, b)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ComplexPair
{
    a: Cplx,
    b: Cplx,
}
impl Summarize for ComplexPair {}

// Unused
impl From<Cplx> for ComplexPair
{
    fn from(t: Cplx) -> Self
    {
        Self { a: t, b: ZERO }
    }
}
impl From<ComplexPair> for Cplx
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
    multiplier: Cplx,
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
    parameter_plane_impl!(Cplx, ComplexPair, Cplx, Cplx);

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
    fn start_point(&self, _m: Cplx, ComplexPair { a, b }: Self::Param) -> Self::Var
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

    fn cycles_child(&self, Self::Param { a, b }: Self::Param, period: Period) -> Vec<Self::Var>
    {
        match period
        {
            1 =>
            {
                let u = b / a;
                solve_cubic(u, 2. / a - 1., -u).to_vec()
            }
            _ => vec![],
        }
    }

    fn get_meta_params(&self) -> Self::MetaParam
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

    fn param_map(&self, m: Cplx) -> Self::Param
    {
        let s = (1. - self.get_meta_params()) / 4.;
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

impl Default for CubicPer2LambdaParam
{
    fractal_impl!(-2.5, 2.5, -2.5, 2.5);
}

impl ParameterPlane for CubicPer2LambdaParam
{
    type Param = Cplx;
    type MetaParam = NoParam;
    type Var = Cplx;
    type Deriv = Cplx;
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
    fn start_point(&self, _point: Cplx, c: Self::Param) -> Self::Var
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

    fn default_selection(&self) -> Cplx
    {
        ZERO
    }

    fn default_julia_bounds(&self, point: Cplx, _param: Self::Param) -> Bounds
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

// Cubic polynomials with critical 2-cycle 0 <-> c
#[derive(Clone, Debug)]
pub struct CubicPer2CritMarked
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl CubicPer2CritMarked
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -2.6,
        max_x: 2.6,
        min_y: -1.9,
        max_y: 1.9,
    };
}
impl Default for CubicPer2CritMarked
{
    fractal_impl!();
}

impl ParameterPlane for CubicPer2CritMarked
{
    parameter_plane_impl!();
    default_name!();

    fn encode_escaping_point(
        &self,
        iters: Period,
        z: Cplx,
        _base_param: Cplx,
    ) -> PointInfo<Self::Deriv>
    {
        if z.is_nan()
        {
            return PointInfo::Escaping {
                potential: f64::from(iters) - 1.,
            };
        }

        let u = self.escape_radius().log2();
        let v = z.norm_sqr().log2();
        let residual = (v / u).log(3.);
        let potential = f64::from(iters) - (residual as IterCount);
        PointInfo::Escaping { potential }
    }

    #[inline]
    fn map(&self, z: Cplx, c: Cplx) -> Cplx
    {
        z * z * (z - c - 1. / c) + c
    }

    #[inline]
    fn start_point(&self, _point: Cplx, param: Cplx) -> Cplx
    {
        2. / 3. * (param + 1. / param)
    }

    #[inline]
    fn dynamical_derivative(&self, z: Cplx, c: Cplx) -> Cplx
    {
        // let u = z * (3. * z - c - c - 2. / c) * (z / c).re.signum();
        // u / u.norm()
        z * (3. * z - c - c - 2. / c)
    }

    #[inline]
    fn parameter_derivative(&self, z: Cplx, c: Cplx) -> Cplx
    {
        let z2 = z * z;
        let c2 = c * c;
        1. + z2 / c2 + -z2
    }

    #[inline]
    fn critical_points_child(&self, c: Self::Param) -> Vec<Self::Var>
    {
        let u = c + c.inv();
        vec![(0.).into(), TWO_THIRDS * u]
    }

    fn cycles_child(&self, c: Self::Param, period: Period) -> Vec<Self::Var>
    {
        match period
        {
            1 => solve_cubic(c, -ONE, -c - c.inv()).to_vec(),
            2 =>
            {
                let cinv = c.inv();
                let cinv2 = cinv * cinv;
                let [r2, r3, r4, r5] = solve_quartic(cinv2, c - cinv, cinv2 + 1., -c - cinv - cinv);
                vec![ZERO, c, r2, r3, r4, r5]
            }
            _ => vec![],
        }
    }

    #[inline]
    fn default_julia_bounds(&self, _point: Cplx, param: Cplx) -> Bounds
    {
        Bounds::square(2.2, param / 2.)
    }
}

impl HasDynamicalCovers for CubicPer2CritMarked
{
    fn marked_cycle_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> Cplx;
        let bounds: Bounds;

        match period
        {
            1 =>
            {
                param_map = |t| {
                    let g2 = 0.5.into();
                    let g3 = Cplx::new(-0.0625, 0.);
                    let (mut x, y) = weierstrass_p(g2, g3, t, 0.01);

                    x += x;
                    x * (x - 1.) / (y + y - x + 0.5)
                };
                bounds = Bounds {
                    min_x: -3.5,
                    max_x: 3.5,
                    min_y: -3.5,
                    max_y: 3.5,
                };
            }
            2 =>
            {
                param_map = |t| t + t.inv();
                bounds = Bounds {
                    min_x: -2.2,
                    max_x: 2.2,
                    min_y: -2.8,
                    max_y: 2.8,
                };
            }
            _ =>
            {
                param_map = |c| c;
                bounds = self.point_grid.bounds.clone();
            }
        };
        let grid = self.point_grid.new_with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }

    fn dynatomic_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> Cplx;
        let bounds: Bounds;

        match period
        {
            1 =>
            {
                param_map = |t| {
                    let g2 = 0.5.into();
                    let g3 = Cplx::new(-0.0625, 0.);
                    let (mut x, y) = weierstrass_p(g2, g3, t, 0.01);

                    x += x;
                    x * (x - 1.) / (y + y - x + 0.5)
                };
                bounds = Bounds {
                    min_x: -3.5,
                    max_x: 3.5,
                    min_y: -3.5,
                    max_y: 3.5,
                };
            }
            _ =>
            {
                param_map = |c| c;
                bounds = self.point_grid.bounds.clone();
            }
        };
        let grid = self.point_grid.new_with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }
}
