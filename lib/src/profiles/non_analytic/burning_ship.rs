use crate::{macros::*, types::variables::Matrix2x2};
profile_imports!();

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BurningShip
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl BurningShip
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -2.2,
        max_x: 1.25,
        min_y: -1.9,
        max_y: 0.6,
    };
    const JULIA_BOUNDS: Bounds = Bounds::centered_square(4.);
}
impl Default for BurningShip
{
    fractal_impl!();
}

impl ParameterPlane for BurningShip
{
    type Var = Cplx;
    type Param = Cplx;
    type MetaParam = NoParam;
    // type Deriv = Matrix2x2;
    type Deriv = Cplx;
    type Child = JuliaSet<Self>;
    basic_plane_impl!();
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
        let residual = (v / u).log2();
        let potential = f64::from(iters) - (residual as IterCount);
        PointInfo::Escaping { potential }
    }

    #[inline]
    fn map(&self, z: Cplx, c: Cplx) -> Cplx
    {
        let z = Cplx::new(z.re.abs(), z.im.abs());
        z * z + c
    }

    #[inline]
    fn dynamical_derivative(&self, z: Cplx, _c: Cplx) -> Self::Deriv
    {
        z + z
    }

    #[inline]
    fn parameter_derivative(&self, _z: Cplx, _c: Cplx) -> Self::Deriv
    {
        ONE
        // Matrix2x2::identity()
    }

    #[inline]
    fn critical_points_child(&self, _param: Self::Param) -> Vec<Self::Var>
    {
        vec![ZERO]
    }

    #[inline]
    fn cycles_child(&self, c: Self::Param, period: Period) -> Vec<Self::Var>
    {
        match period
        {
            1 =>
            {
                let disc = (1. - 4. * c).sqrt();
                vec![0.5 * (1. - disc), 0.5 * (1. + disc)]
            }
            _ => vec![],
        }
    }

    #[inline]
    fn param_map(&self, c: Cplx) -> Cplx
    {
        c
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Sailboat
{
    point_grid: PointGrid,
    max_iter: Period,
    shift: Cplx,
}

impl Sailboat
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -6.,
        max_x: 6.,
        min_y: -6.,
        max_y: 6.,
    };
    const JULIA_BOUNDS: Bounds = Bounds::centered_square(5.);
}
impl Default for Sailboat
{
    fractal_impl!(shift, ZERO);
}

impl ParameterPlane for Sailboat
{
    type Var = Cplx;
    type Param = Cplx;
    type MetaParam = Cplx;
    type Deriv = Cplx;
    type Child = JuliaSet<Self>;
    basic_plane_impl!();

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
        let residual = (v / u).log2();
        let potential = f64::from(iters) - (residual as IterCount);
        PointInfo::Escaping { potential }
    }

    #[inline]
    fn map(&self, z: Cplx, c: Cplx) -> Cplx
    {
        let z = Cplx::new(z.re.abs(), z.im.abs()) + self.shift;
        z * z + c
    }

    #[inline]
    fn dynamical_derivative(&self, z: Cplx, _c: Cplx) -> Cplx
    {
        let mut w = z + z;
        w.re *= z.re.signum();
        w.im *= z.im.signum();
        w
    }

    #[inline]
    fn parameter_derivative(&self, _z: Cplx, _c: Cplx) -> Cplx
    {
        ONE //TODO
    }

    #[inline]
    fn gradient(&self, _z: Self::Var, _c: Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        (ONE, ONE, ONE) //TODO
    }

    #[inline]
    fn param_map(&self, c: Cplx) -> Cplx
    {
        c
    }

    fn set_param(&mut self, new_param: <Self::MetaParam as ParamList>::Param)
    {
        self.shift = new_param;
    }

    #[inline]
    fn name(&self) -> String
    {
        let shift = self.shift;
        format!("Sailboat({shift})")
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SailboatParam
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl Default for SailboatParam
{
    fractal_impl!(-2.5, 2.5, -2.5, 2.5);
}

impl ParameterPlane for SailboatParam
{
    type Param = Cplx;
    type MetaParam = NoParam;
    type Var = Cplx;
    type Deriv = Cplx;
    type Child = Sailboat;

    basic_plane_impl!();

    #[inline]
    fn map(&self, z: Self::Var, a: Self::Param) -> Self::Var
    {
        let z = Cplx::new(z.re.abs(), z.im.abs()) + a;
        z * z
    }

    #[inline]
    fn dynamical_derivative(&self, z: Self::Var, _a: Self::Param) -> Self::Deriv
    {
        let mut w = z + z;
        w.re *= z.re.signum();
        w.im *= z.im.signum();
        w
    }

    #[inline]
    fn parameter_derivative(&self, _z: Self::Var, _c: Self::Param) -> Self::Deriv
    {
        ONE
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

    fn default_julia_bounds(&self, _point: Cplx, _param: Self::Param) -> Bounds
    {
        Bounds::centered_square(3.5)
    }
}

impl From<SailboatParam> for Sailboat
{
    fn from(parent: SailboatParam) -> Self
    {
        let point = parent.default_selection();
        let param = parent.param_map(point);
        let point_grid = parent
            .point_grid()
            .new_with_same_height(parent.default_julia_bounds(point, param));
        Self {
            point_grid,
            max_iter: parent.max_iter(),
            shift: param,
        }
    }
}
