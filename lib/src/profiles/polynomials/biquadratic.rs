use crate::macros::*;
use crate::math_utils::solve_cubic;
use crate::types::param_stack::Summarize;
use crate::types::variables::{Bicomplex, PlaneID};
profile_imports!();
use derive_more::{Add, Display, From, Sub};

#[derive(Default, Clone, Copy, Debug, Add, From, PartialEq, Display)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[display(fmt = "[ a: {}, b: {} ] ", a, b)]
pub struct ComplexPair
{
    pub a: Cplx,
    pub b: Cplx,
}

impl Summarize for ComplexPair {}

impl From<Cplx> for ComplexPair
{
    fn from(z: Cplx) -> Self
    {
        Self::from((z, ONE))
    }
}
impl From<ComplexPair> for Cplx
{
    fn from(value: ComplexPair) -> Self
    {
        value.a * value.b
    }
}
impl From<ComplexPair> for Bicomplex
{
    fn from(value: ComplexPair) -> Self
    {
        Self::PlaneA(-0.5 * value.a)
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Biquadratic
{
    point_grid: PointGrid,
    max_iter: Period,
    multiplier: Cplx,
}

impl Biquadratic
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -1.6,
        max_x: 1.25,
        min_y: -1.25,
        max_y: 1.25,
    };
    const JULIA_BOUNDS: Bounds = Bounds::centered_square(2.5);
}

impl Default for Biquadratic
{
    fractal_impl!(multiplier, ZERO);
}

impl ParameterPlane for Biquadratic
{
    type Var = Bicomplex;
    type Param = Cplx;
    type Deriv = Cplx;
    type MetaParam = Cplx;
    type Child = JuliaSet<Self>;
    basic_plane_impl!();

    #[inline]
    fn name(&self) -> String
    {
        let param = self.multiplier;
        format!("Biquadratic({param})")
    }

    fn encode_escaping_point(
        &self,
        iters: Period,
        z: Self::Var,
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
        let residual = (v / u).log2() / 2.;
        let potential = f64::from(iters) - (residual as IterCount);
        PointInfo::Escaping { potential }
    }

    #[inline]
    fn param_map(&self, c: Cplx) -> Cplx
    {
        c
    }

    #[inline]
    fn start_point(&self, _point: Cplx, _c: Cplx) -> Self::Var
    {
        Self::Var::default()
    }

    #[inline]
    fn map(&self, zw: Self::Var, c: Cplx) -> Self::Var
    {
        match zw
        {
            Bicomplex::PlaneA(z) => Bicomplex::PlaneB(z * z + c),
            Bicomplex::PlaneB(w) => Bicomplex::PlaneA(w * w + self.multiplier),
        }
    }

    #[inline]
    fn map_and_multiplier(&self, zw: Self::Var, c: Cplx) -> (Self::Var, Cplx)
    {
        match zw
        {
            Bicomplex::PlaneA(z) => (Bicomplex::PlaneB(z * z + c), z + z),
            Bicomplex::PlaneB(w) => (Bicomplex::PlaneA(w * w + self.multiplier), w + w),
        }
    }

    #[inline]
    fn dynamical_derivative(&self, zw: Self::Var, _c: Cplx) -> Cplx
    {
        let u: Cplx = zw.into();
        u + u
    }

    #[inline]
    fn parameter_derivative(&self, zw: Self::Var, _c: Cplx) -> Cplx
    {
        match zw
        {
            Bicomplex::PlaneA(_) => ONE,
            Bicomplex::PlaneB(_) => ZERO,
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BiquadraticMult
{
    point_grid: PointGrid,
    max_iter: Period,
    multiplier: Cplx,
    starting_plane: PlaneID,
}

impl BiquadraticMult
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -2.8,
        max_x: 2.8,
        min_y: -2.55,
        max_y: 2.55,
    };
    const JULIA_BOUNDS: Bounds = Bounds::centered_square(2.5);
}

impl Default for BiquadraticMult
{
    fn default() -> Self
    {
        let bounds = Self::DEFAULT_BOUNDS;
        let point_grid = PointGrid::new_by_res_y(1024, bounds);
        Self {
            point_grid,
            max_iter: 1024,
            multiplier: 0.5.into(),
            starting_plane: PlaneID::ZPlane,
        }
    }
}

impl ParameterPlane for BiquadraticMult
{
    type Var = Bicomplex;
    type Param = ComplexPair;
    type Deriv = Cplx;
    type MetaParam = Cplx;
    type Child = JuliaSet<Self>;
    basic_plane_impl!();

    #[inline]
    fn name(&self) -> String
    {
        let param = self.multiplier;
        format!("Biquadratic({param})")
    }

    fn encode_escaping_point(
        &self,
        iters: Period,
        z: Self::Var,
        _base_param: Self::Param,
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
        let residual = (v / u).log2();
        let potential = f64::from(iters) - (residual as IterCount);
        PointInfo::Escaping { potential }
    }

    #[inline]
    fn param_map(&self, point: Cplx) -> Self::Param
    {
        Self::Param {
            a: point,
            b: self.multiplier / point,
        }
    }

    #[inline]
    fn start_point(&self, _point: Cplx, c: Self::Param) -> Self::Var
    {
        match self.starting_plane
        {
            PlaneID::ZPlane => Bicomplex::PlaneA(-0.5 * c.a),
            PlaneID::WPlane => Bicomplex::PlaneB(-0.5 * c.b),
        }
    }

    #[inline]
    fn map(&self, zw: Self::Var, c: Self::Param) -> Self::Var
    {
        match zw
        {
            Bicomplex::PlaneA(z) => Bicomplex::PlaneB(z * (z + c.a)),
            Bicomplex::PlaneB(w) => Bicomplex::PlaneA(w * (w + c.b)),
        }
    }

    #[inline]
    fn map_and_multiplier(&self, zw: Self::Var, c: Self::Param) -> (Self::Var, Cplx)
    {
        match zw
        {
            Bicomplex::PlaneA(z) => (Bicomplex::PlaneB(z * (z + c.a)), z + z + c.a),
            Bicomplex::PlaneB(w) => (Bicomplex::PlaneA(w * (w + c.b)), w + w + c.b),
        }
    }

    #[inline]
    fn dynamical_derivative(&self, zw: Self::Var, c: Self::Param) -> Cplx
    {
        match zw
        {
            Bicomplex::PlaneA(z) => z + z + c.a,
            Bicomplex::PlaneB(w) => w + w + c.b,
        }
    }

    #[inline]
    fn parameter_derivative(&self, zw: Self::Var, c: Self::Param) -> Cplx
    {
        match zw
        {
            Bicomplex::PlaneA(_) => ONE,
            Bicomplex::PlaneB(_) => -c.b / c.a,
        }
    }

    fn critical_points(&self) -> Vec<Self::Var>
    {
        let l = self.multiplier;
        let d0 = ((l - 8.) * l + 32.).sqrt();
        let q0 = (2. * (l - 4. + d0)).powf(ONE_THIRD);
        let q1 = (2. * (l - 4. - d0)).powf(ONE_THIRD);
        [
            q0,
            q1,
            q0 * OMEGA,
            q1 * OMEGA,
            q0 * OMEGA_BAR,
            q1 * OMEGA_BAR,
        ]
        .iter()
        .map(|x| Bicomplex::PlaneA(*x))
        .collect()
    }

    #[inline]
    fn critical_points_child(&self, c: Self::Param) -> Vec<Self::Var>
    {
        match self.starting_plane
        {
            PlaneID::ZPlane =>
            {
                let disc = (c.a * c.a - c.b - c.b).sqrt();
                vec![
                    Bicomplex::PlaneA(-0.5 * c.a),
                    Bicomplex::PlaneA(-0.5 * (c.a + disc)),
                    Bicomplex::PlaneA(-0.5 * (c.a - disc)),
                ]
            }
            PlaneID::WPlane =>
            {
                let disc = (c.b * c.b - c.a - c.a).sqrt();
                vec![
                    Bicomplex::PlaneB(-0.5 * c.b),
                    Bicomplex::PlaneB(-0.5 * (c.b + disc)),
                    Bicomplex::PlaneB(-0.5 * (c.b - disc)),
                ]
            }
        }
    }

    fn cycles_child(&self, ComplexPair { a, b }: Self::Param, period: Period) -> Vec<Self::Var>
    {
        match period
        {
            2 => match self.starting_plane
            {
                PlaneID::ZPlane =>
                {
                    let [r0, r1, r2] = solve_cubic(a * b - 1., a * a + b, a + a);
                    vec![
                        Bicomplex::PlaneA(ZERO),
                        Bicomplex::PlaneA(r0),
                        Bicomplex::PlaneA(r1),
                        Bicomplex::PlaneA(r2),
                    ]
                }
                PlaneID::WPlane =>
                {
                    let [r0, r1, r2] = solve_cubic(b * a - 1., b * b + a, b + b);
                    vec![
                        Bicomplex::PlaneB(ZERO),
                        Bicomplex::PlaneB(r0),
                        Bicomplex::PlaneB(r1),
                        Bicomplex::PlaneB(r2),
                    ]
                }
            },
            _ => vec![],
        }
    }

    fn cycle_active_plane(&mut self)
    {
        self.starting_plane = self.starting_plane.swap();
    }

    fn dynam_map(&self, point: Cplx) -> Self::Var
    {
        match self.starting_plane
        {
            PlaneID::ZPlane => Bicomplex::PlaneA(point),
            PlaneID::WPlane => Bicomplex::PlaneB(point),
        }
    }

    fn periodicity_tolerance(&self) -> Real
    {
        1e-14
    }

    fn default_selection(&self) -> Cplx
    {
        Cplx::new(1.0626588, 0.)
    }

    fn default_julia_bounds(&self, _point: Cplx, c: Self::Param) -> Bounds
    {
        Bounds::square(2.5, -0.5 * c.a)
    }

    fn set_meta_param(&mut self, meta_param: Self::MetaParam)
    {
        self.multiplier = meta_param;
    }

    fn set_param(&mut self, multiplier: Cplx)
    {
        self.multiplier = multiplier;
    }

    fn get_meta_params(&self) -> Self::MetaParam
    {
        self.multiplier
    }

    fn get_param(&self) -> <Self::MetaParam as ParamList>::Param
    {
        self.multiplier
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BiquadraticMultParam
{
    point_grid: PointGrid,
    max_iter: Period,
    starting_plane: PlaneID,
}

impl BiquadraticMultParam
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -2.2,
        max_x: 4.2,
        min_y: -2.5,
        max_y: 2.5,
    };
}
impl Default for BiquadraticMultParam
{
    fn default() -> Self
    {
        let bounds = Self::DEFAULT_BOUNDS;
        let point_grid = PointGrid::new_by_res_y(1024, bounds);
        Self {
            point_grid,
            max_iter: 1024,
            starting_plane: PlaneID::ZPlane,
        }
    }
}

impl ParameterPlane for BiquadraticMultParam
{
    type Param = ComplexPair;
    type Var = Bicomplex;
    type Deriv = Cplx;
    type MetaParam = NoParam;
    type Child = BiquadraticMult;
    basic_plane_impl!();

    fn encode_escaping_point(
        &self,
        iters: Period,
        z: Self::Var,
        _base_param: Self::Param,
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
        let residual = (v / u).log2();
        let potential = f64::from(iters) - (residual as IterCount);
        PointInfo::Escaping { potential }
    }

    #[inline]
    fn param_map(&self, c: Cplx) -> Self::Param
    {
        Self::Param {
            a: 1e-4.into(),
            b: c * 1e4,
        }
    }
    #[inline]
    fn map(&self, zw: Self::Var, c: Self::Param) -> Self::Var
    {
        match zw
        {
            Bicomplex::PlaneA(z) => Bicomplex::PlaneB(z * (z + c.a)),
            Bicomplex::PlaneB(w) => Bicomplex::PlaneA(w * (w + c.b)),
        }
    }

    #[inline]
    fn map_and_multiplier(&self, zw: Self::Var, c: Self::Param) -> (Self::Var, Cplx)
    {
        match zw
        {
            Bicomplex::PlaneA(z) => (Bicomplex::PlaneB(z * (z + c.a)), z + z + c.a),
            Bicomplex::PlaneB(w) => (Bicomplex::PlaneA(w * (w + c.b)), w + w + c.b),
        }
    }

    #[inline]
    fn dynamical_derivative(&self, zw: Self::Var, c: Self::Param) -> Cplx
    {
        match zw
        {
            Bicomplex::PlaneA(z) => z + z + c.a,
            Bicomplex::PlaneB(w) => w + w + c.b,
        }
    }

    #[inline]
    fn parameter_derivative(&self, zw: Self::Var, c: Self::Param) -> Cplx
    {
        match zw
        {
            Bicomplex::PlaneA(_) => ONE,
            Bicomplex::PlaneB(_) => -c.b / c.a,
        }
    }

    #[inline]
    fn start_point(&self, _point: Cplx, c: Self::Param) -> Self::Var
    {
        match self.starting_plane
        {
            PlaneID::ZPlane => Bicomplex::PlaneA(-0.5 * c.a),
            PlaneID::WPlane => Bicomplex::PlaneB(-0.5 * c.b),
        }
    }

    fn cycle_active_plane(&mut self)
    {
        self.starting_plane = self.starting_plane.swap();
    }

    fn default_julia_bounds(&self, _point: Cplx, _param: Self::Param) -> Bounds
    {
        Bounds::centered_square(3.5)
    }

    fn default_selection(&self) -> Cplx
    {
        // (1.0 - 5.0_f64.sqrt()).into()
        Cplx::new(0., 0.99)
    }

    fn name(&self) -> String
    {
        "Biquadratic Param".to_owned()
    }
}

impl From<BiquadraticMultParam> for BiquadraticMult
{
    fn from(parent: BiquadraticMultParam) -> Self
    {
        let point = parent.default_selection();
        let param = parent.param_map(point);
        let point_grid = parent
            .point_grid()
            .new_with_same_height(parent.default_julia_bounds(point, param));
        Self {
            point_grid,
            max_iter: parent.max_iter(),
            multiplier: param.a * param.b,
            starting_plane: parent.starting_plane,
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BiquadraticMultSecondIterate
{
    point_grid: PointGrid,
    max_iter: Period,
    multiplier: Cplx,
}

impl BiquadraticMultSecondIterate
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -2.6,
        max_x: 3.25,
        min_y: -2.25,
        max_y: 2.25,
    };
    const JULIA_BOUNDS: Bounds = Bounds::centered_square(2.5);
}
impl Default for BiquadraticMultSecondIterate
{
    fractal_impl!(multiplier, ZERO);
}

impl ParameterPlane for BiquadraticMultSecondIterate
{
    type Var = Cplx;
    type Param = Cplx;
    type Deriv = Cplx;
    type MetaParam = Cplx;
    type Child = JuliaSet<Self>;
    basic_plane_impl!();

    #[inline]
    fn name(&self) -> String
    {
        let param = self.multiplier;
        format!("BiquadraticMult({param})")
    }

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
        let residual = (v / u).log2() / 2.;
        let potential = f64::from(iters) - (residual as IterCount);
        PointInfo::Escaping { potential }
    }

    #[inline]
    fn param_map(&self, c: Cplx) -> Cplx
    {
        c
    }

    #[inline]
    fn start_point(&self, _point: Cplx, c: Cplx) -> Cplx
    {
        -0.5 * c
    }

    #[inline]
    fn map(&self, z: Cplx, c: Cplx) -> Cplx
    {
        let w = z * (z + c);
        w * (w + self.multiplier / c)
    }

    #[inline]
    fn map_and_multiplier(&self, z: Cplx, c: Cplx) -> (Cplx, Cplx)
    {
        let a = self.multiplier / c;
        let w = z * (z + c);
        (w * (w + a), (c + z + z) * (a + w + w))
    }

    #[inline]
    fn dynamical_derivative(&self, z: Cplx, c: Cplx) -> Cplx
    {
        let a = self.multiplier / c;
        let w = z * (z + c);
        (c + z + z) * (a + w + w)
    }

    #[inline]
    fn parameter_derivative(&self, z: Cplx, c: Cplx) -> Cplx
    {
        2. * (z * z + c)
    }

    #[inline]
    fn gradient(&self, z: Self::Var, c: Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        let a = self.multiplier / c;
        let x0 = c + z;
        let w = z * x0;
        let x2 = w + a;
        let x2z = x2 * z;
        (
            w * x2,
            x0 * x2 + w * (c + z + z) + x2z,
            w * (z - a * a) + x2z,
        )
    }

    fn set_meta_param(&mut self, value: Self::Param)
    {
        self.multiplier = value;
    }

    fn get_meta_params(&self) -> Self::Param
    {
        self.multiplier
    }
}
