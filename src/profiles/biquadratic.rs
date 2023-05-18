use crate::macros::*;
profile_imports!();
use derive_more::{Add, Display, From, Sub};

#[derive(Copy, Clone, Debug, Display)]
pub enum Bicomplex
{
    #[display(fmt = "PlaneA({})", _0)]
    PlaneA(ComplexNum),
    #[display(fmt = "PlaneB({})", _0)]
    PlaneB(ComplexNum),
}

impl From<ComplexNum> for Bicomplex
{
    fn from(value: ComplexNum) -> Self
    {
        // Bicomplex::PlaneA(value)
        Bicomplex::PlaneB(value)
    }
}
impl From<Bicomplex> for ComplexNum
{
    fn from(value: Bicomplex) -> Self
    {
        match value
        {
            Bicomplex::PlaneA(z) => z,
            Bicomplex::PlaneB(z) => z,
        }
    }
}
impl Norm<RealNum> for Bicomplex
{
    fn norm(&self) -> RealNum
    {
        match self
        {
            Bicomplex::PlaneA(z) => z.norm(),
            Bicomplex::PlaneB(z) => z.norm(),
        }
    }
    fn norm_sqr(&self) -> RealNum
    {
        match self
        {
            Bicomplex::PlaneA(z) => z.norm_sqr(),
            Bicomplex::PlaneB(z) => z.norm_sqr(),
        }
    }
    fn arg(&self) -> RealNum
    {
        match self
        {
            Bicomplex::PlaneA(z) => z.arg(),
            Bicomplex::PlaneB(z) => z.arg(),
        }
    }
    fn is_nan(&self) -> bool
    {
        match self
        {
            Bicomplex::PlaneA(z) => z.is_nan(),
            Bicomplex::PlaneB(z) => z.is_nan(),
        }
    }
}

impl Default for Bicomplex
{
    fn default() -> Self
    {
        Bicomplex::PlaneA(ZERO)
    }
}

impl Dist<RealNum> for Bicomplex
{
    fn dist(&self, rhs: Self) -> RealNum
    {
        match self
        {
            Bicomplex::PlaneA(z) => match rhs
            {
                Bicomplex::PlaneA(w) => (z - w).norm(),
                Bicomplex::PlaneB(_) => RealNum::INFINITY,
            },
            Bicomplex::PlaneB(z) => match rhs
            {
                Bicomplex::PlaneA(_) => RealNum::INFINITY,
                Bicomplex::PlaneB(w) => (z - w).norm(),
            },
        }
    }
    fn dist_sqr(&self, rhs: Self) -> RealNum
    {
        match self
        {
            Bicomplex::PlaneA(z) => match rhs
            {
                Bicomplex::PlaneA(w) => (z - w).norm_sqr(),
                Bicomplex::PlaneB(_) => RealNum::INFINITY,
            },
            Bicomplex::PlaneB(z) => match rhs
            {
                Bicomplex::PlaneA(_) => RealNum::INFINITY,
                Bicomplex::PlaneB(w) => (z - w).norm_sqr(),
            },
        }
    }
}

#[derive(Default, Clone, Copy, Add, From, Display)]
#[display(fmt = "[ a: {}, b: {} ] ", a, b)]
pub struct Param
{
    pub a: ComplexNum,
    pub b: ComplexNum,
}

impl From<ComplexNum> for Param
{
    fn from(z: ComplexNum) -> Self
    {
        Param::from((z, ONE))
    }
}
impl From<Param> for ComplexNum
{
    fn from(value: Param) -> Self
    {
        -0.5 * value.a
    }
}
impl From<Param> for Bicomplex
{
    fn from(value: Param) -> Self
    {
        Bicomplex::PlaneA(-0.5 * value.a)
    }
}

#[derive(Clone, Debug)]
pub struct Biquadratic
{
    point_grid: PointGrid,
    max_iter: Period,
    param: ComplexNum,
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

    #[must_use]
    pub const fn new(
        res_x: usize,
        res_y: usize,
        max_iter: Period,
        param: ComplexNum,
        bounds: Bounds,
    ) -> Self
    {
        let point_grid = PointGrid::new(res_x, res_y, bounds);

        Self {
            point_grid,
            max_iter,
            param,
        }
    }

    #[must_use]
    pub const fn with_res_y(
        res_y: usize,
        max_iter: Period,
        param: ComplexNum,
        bounds: Bounds,
    ) -> Self
    {
        let point_grid = PointGrid::with_res_y(res_y, bounds);

        Self {
            point_grid,
            max_iter,
            param,
        }
    }

    #[must_use]
    pub const fn with_res_x(
        res_x: usize,
        max_iter: Period,
        param: ComplexNum,
        bounds: Bounds,
    ) -> Self
    {
        let point_grid = PointGrid::with_res_x(res_x, bounds);
        Self {
            point_grid,
            max_iter,
            param,
        }
    }

    #[must_use]
    pub const fn new_default(res_y: usize, max_iter: Period, param: ComplexNum) -> Self
    {
        let bounds = Self::DEFAULT_BOUNDS;
        Self::with_res_y(res_y, max_iter, param, bounds)
    }
}

impl ParameterPlane for Biquadratic
{
    type Var = Bicomplex;
    type Param = ComplexNum;
    type Deriv = ComplexNum;
    basic_plane_impl!();

    #[inline]
    fn name(&self) -> String
    {
        let param = self.param;
        format!("Biquadratic({param})")
    }

    fn encode_escaping_point(
        &self,
        iters: Period,
        z: Self::Var,
        _base_param: ComplexNum,
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
    fn param_map(&self, c: ComplexNum) -> ComplexNum
    {
        c
    }

    #[inline]
    fn start_point(&self, _point: ComplexNum, _c: ComplexNum) -> Self::Var
    {
        Self::Var::default()
    }

    #[inline]
    fn map(&self, zw: Self::Var, c: ComplexNum) -> Self::Var
    {
        match zw
        {
            Bicomplex::PlaneA(z) => Bicomplex::PlaneB(z * z + c),
            Bicomplex::PlaneB(w) => Bicomplex::PlaneA(w * w + self.param),
        }
    }

    #[inline]
    fn map_and_multiplier(&self, zw: Self::Var, c: ComplexNum) -> (Self::Var, ComplexNum)
    {
        match zw
        {
            Bicomplex::PlaneA(z) => (Bicomplex::PlaneB(z * z + c), z + z),
            Bicomplex::PlaneB(w) => (Bicomplex::PlaneA(w * w + self.param), w + w),
        }
    }

    #[inline]
    fn dynamical_derivative(&self, zw: Self::Var, c: ComplexNum) -> ComplexNum
    {
        let u: ComplexNum = zw.into();
        u + u
    }

    #[inline]
    fn parameter_derivative(&self, zw: Self::Var, c: ComplexNum) -> ComplexNum
    {
        match zw
        {
            Bicomplex::PlaneA(_) => ONE,
            Bicomplex::PlaneB(_) => ZERO,
        }
    }
}

#[derive(Clone, Debug)]
pub struct BiquadraticMult
{
    point_grid: PointGrid,
    max_iter: Period,
    multiplier: ComplexNum,
}

impl BiquadraticMult
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -2.5,
        max_x: 2.5,
        min_y: -2.25,
        max_y: 2.25,
    };
    const JULIA_BOUNDS: Bounds = Bounds::centered_square(2.5);

    #[must_use]
    pub const fn new(
        res_x: usize,
        res_y: usize,
        max_iter: Period,
        param: ComplexNum,
        bounds: Bounds,
    ) -> Self
    {
        let point_grid = PointGrid::new(res_x, res_y, bounds);

        Self {
            point_grid,
            max_iter,
            multiplier: param,
        }
    }

    #[must_use]
    pub const fn with_res_y(
        res_y: usize,
        max_iter: Period,
        param: ComplexNum,
        bounds: Bounds,
    ) -> Self
    {
        let point_grid = PointGrid::with_res_y(res_y, bounds);

        Self {
            point_grid,
            max_iter,
            multiplier: param,
        }
    }

    #[must_use]
    pub const fn with_res_x(
        res_x: usize,
        max_iter: Period,
        multiplier: ComplexNum,
        bounds: Bounds,
    ) -> Self
    {
        let point_grid = PointGrid::with_res_x(res_x, bounds);
        Self {
            point_grid,
            max_iter,
            multiplier,
        }
    }

    #[must_use]
    pub const fn new_default(res_y: usize, max_iter: Period, multiplier: ComplexNum) -> Self
    {
        let bounds = Self::DEFAULT_BOUNDS;
        Self::with_res_y(res_y, max_iter, multiplier, bounds)
    }
}

impl ParameterPlane for BiquadraticMult
{
    type Var = Bicomplex;
    type Param = Param;
    type Deriv = ComplexNum;
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
    fn param_map(&self, point: ComplexNum) -> Self::Param
    {
        Self::Param {
            a: point,
            b: self.multiplier / point,
        }
    }

    #[inline]
    fn start_point(&self, _point: ComplexNum, c: Self::Param) -> Self::Var
    {
        Bicomplex::PlaneA(-0.5 * c.a)
        // Bicomplex::PlaneB(-0.5*c.b)
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
    fn map_and_multiplier(&self, zw: Self::Var, c: Self::Param) -> (Self::Var, ComplexNum)
    {
        match zw
        {
            Bicomplex::PlaneA(z) => (Bicomplex::PlaneB(z * (z + c.a)), z + z + c.a),
            Bicomplex::PlaneB(w) => (Bicomplex::PlaneA(w * (w + c.b)), w + w + c.b),
        }
    }

    #[inline]
    fn dynamical_derivative(&self, zw: Self::Var, c: Self::Param) -> ComplexNum
    {
        match zw
        {
            Bicomplex::PlaneA(z) => z + z + c.a,
            Bicomplex::PlaneB(w) => w + w + c.b,
        }
    }

    #[inline]
    fn parameter_derivative(&self, zw: Self::Var, c: Self::Param) -> ComplexNum
    {
        match zw
        {
            Bicomplex::PlaneA(_) => ONE,
            Bicomplex::PlaneB(_) => -c.b / c.a,
        }
    }

    #[inline]
    fn critical_points(&self, c: Self::Param) -> Vec<Self::Var>
    {
        // vec![Bicomplex::PlaneA(-0.5 * c.a), Bicomplex::PlaneB(-0.5 * c.b)]
        //
        // let disc = (c.a * c.a - c.b - c.b).sqrt();
        // vec![
        //     Bicomplex::PlaneA(-0.5 * c.a),
        //     Bicomplex::PlaneA(-0.5 * (c.a + disc)),
        //     Bicomplex::PlaneA(-0.5 * (c.a - disc)),
        // ]
        let disc = (c.b * c.b - c.a - c.a).sqrt();
        vec![
            Bicomplex::PlaneB(-0.5 * c.b),
            Bicomplex::PlaneB(-0.5 * (c.b + disc)),
            Bicomplex::PlaneB(-0.5 * (c.b - disc)),
        ]
    }

    fn periodicity_tolerance(&self) -> RealNum
    {
        1e-14
    }

    fn default_selection(&self) -> ComplexNum
    {
        ComplexNum::new(1.0626588, 0.)
    }

    fn default_julia_bounds(&self, _point: ComplexNum, c: Self::Param) -> Bounds {
        Bounds::square(2.5, -0.5*c.a)
    }

    fn set_param(&mut self, value: Self::Param)
    {
        self.multiplier = value.a * value.b;
    }
}

#[derive(Clone, Debug)]
pub struct BiquadraticMultParam
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl BiquadraticMultParam
{
    fractal_impl!(-2.2, 4.2, -2.5, 2.5);
}

impl ParameterPlane for BiquadraticMultParam
{
    type Param = Param;
    type Var = Bicomplex;
    type Deriv = ComplexNum;
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
    fn param_map(&self, c: ComplexNum) -> Self::Param
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
    fn map_and_multiplier(&self, zw: Self::Var, c: Self::Param) -> (Self::Var, ComplexNum)
    {
        match zw
        {
            Bicomplex::PlaneA(z) => (Bicomplex::PlaneB(z * (z + c.a)), z + z + c.a),
            Bicomplex::PlaneB(w) => (Bicomplex::PlaneA(w * (w + c.b)), w + w + c.b),
        }
    }

    #[inline]
    fn dynamical_derivative(&self, zw: Self::Var, c: Self::Param) -> ComplexNum
    {
        match zw
        {
            Bicomplex::PlaneA(z) => z + z + c.a,
            Bicomplex::PlaneB(w) => w + w + c.b,
        }
    }

    #[inline]
    fn parameter_derivative(&self, zw: Self::Var, c: Self::Param) -> ComplexNum
    {
        match zw
        {
            Bicomplex::PlaneA(_) => ONE,
            Bicomplex::PlaneB(_) => -c.b / c.a,
        }
    }

    #[inline]
    fn start_point(&self, _point: ComplexNum, c: Self::Param) -> Self::Var
    {
        Bicomplex::PlaneA(-0.5 * c.a)
        // Bicomplex::PlaneB(-0.5 * c.a)
    }

    fn default_julia_bounds(&self, _point: ComplexNum, _param: Self::Param) -> Bounds
    {
        Bounds::centered_square(3.5)
    }

    fn default_selection(&self) -> ComplexNum
    {
        // (1.0 - 5.0_f64.sqrt()).into()
        ComplexNum::new(0., 0.99)
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
            .with_same_height(parent.default_julia_bounds(point, param));
        Self {
            point_grid,
            max_iter: parent.max_iter(),
            multiplier: param.a * param.b,
        }
    }
}

#[derive(Clone, Debug)]
pub struct BiquadraticMult_second_iterate
{
    point_grid: PointGrid,
    max_iter: Period,
    multiplier: ComplexNum,
}

impl BiquadraticMult_second_iterate
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -2.6,
        max_x: 3.25,
        min_y: -2.25,
        max_y: 2.25,
    };
    const JULIA_BOUNDS: Bounds = Bounds::centered_square(2.5);

    #[must_use]
    pub const fn new(
        res_x: usize,
        res_y: usize,
        max_iter: Period,
        param: ComplexNum,
        bounds: Bounds,
    ) -> Self
    {
        let point_grid = PointGrid::new(res_x, res_y, bounds);

        Self {
            point_grid,
            max_iter,
            multiplier: param,
        }
    }

    #[must_use]
    pub const fn with_res_y(
        res_y: usize,
        max_iter: Period,
        param: ComplexNum,
        bounds: Bounds,
    ) -> Self
    {
        let point_grid = PointGrid::with_res_y(res_y, bounds);

        Self {
            point_grid,
            max_iter,
            multiplier: param,
        }
    }

    #[must_use]
    pub const fn with_res_x(
        res_x: usize,
        max_iter: Period,
        param: ComplexNum,
        bounds: Bounds,
    ) -> Self
    {
        let point_grid = PointGrid::with_res_x(res_x, bounds);
        Self {
            point_grid,
            max_iter,
            multiplier: param,
        }
    }

    #[must_use]
    pub const fn new_default(res_y: usize, max_iter: Period, param: ComplexNum) -> Self
    {
        let bounds = Self::DEFAULT_BOUNDS;
        Self::with_res_y(res_y, max_iter, param, bounds)
    }
}

impl ParameterPlane for BiquadraticMult_second_iterate
{
    parameter_plane_impl!();

    #[inline]
    fn name(&self) -> String
    {
        let param = self.multiplier;
        format!("BiquadraticMult({param})")
    }

    fn encode_escaping_point(
        &self,
        iters: Period,
        z: ComplexNum,
        _base_param: ComplexNum,
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
    fn param_map(&self, c: ComplexNum) -> ComplexNum
    {
        c
    }

    #[inline]
    fn start_point(&self, _point: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        -0.5 * c
    }

    #[inline]
    fn map(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        let w = z * (z + c);
        w * (w + self.multiplier / c)
    }

    #[inline]
    fn map_and_multiplier(&self, z: ComplexNum, c: ComplexNum) -> (ComplexNum, ComplexNum)
    {
        let a = self.multiplier / c;
        let w = z * (z + c);
        (w * (w + a), (c + z + z) * (a + w + w))
    }

    #[inline]
    fn dynamical_derivative(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        let a = self.multiplier / c;
        let w = z * (z + c);
        (c + z + z) * (a + w + w)
    }

    #[inline]
    fn parameter_derivative(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
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

    fn set_param(&mut self, value: Self::Param)
    {
        self.multiplier = value;
    }

    fn get_param(&self) -> Self::Param
    {
        self.multiplier
    }
}
