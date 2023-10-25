use crate::macros::{degree_impl, profile_imports};
use dynamo_common::types::variables::Matrix2x2;
profile_imports!();

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BurningShip<const N: Period>
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl<const N: Period> BurningShip<N>
{
    const N_FLOAT: Real = N as Real;
    const N_MINUS_1: Real = (N - 1) as Real;
    const DEFAULT_BOUNDS: Bounds = match N
    {
        2 => Bounds {
            min_x: -2.2,
            max_x: 1.25,
            min_y: -1.9,
            max_y: 0.6,
        },
        _ => Bounds::centered_square(1.5),
    };
}
impl<const N: Period> Default for BurningShip<N>
{
    fractal_impl!();
}

impl<const N: Period> ParameterPlane for BurningShip<N>
{
    type Var = Cplx;
    type Param = Cplx;
    type MetaParam = NoParam;
    // type Deriv = Matrix2x2;
    type Deriv = Cplx;
    type Child = JuliaSet<Self>;
    basic_plane_impl!();
    default_name!();
    default_bounds!();

    #[inline]
    fn map(&self, z: Cplx, c: Cplx) -> Cplx
    {
        let z = Cplx::new(z.re.abs(), z.im.abs());
        z.powf(Self::N_FLOAT) + c
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, c: Self::Param) -> (Self::Var, Self::Deriv)
    {
        let znm1 = z.powf(Self::N_MINUS_1);
        (znm1 * z + c, Self::N_FLOAT * znm1)
    }

    #[inline]
    fn gradient(&self, z: Self::Var, c: Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        let (f, df) = self.map_and_multiplier(z, c);
        (f, df, ONE)
    }

    #[inline]
    fn start_point(&self, _point: Cplx, _c: Self::Param) -> Self::Var
    {
        ZERO
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

    fn default_julia_bounds(&self, _point: Cplx, _c: Self::Param) -> Bounds
    {
        Bounds::centered_square(4.)
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
    default_bounds!();

    #[inline]
    fn map(&self, z: Cplx, c: Cplx) -> Cplx
    {
        let z = Cplx::new(z.re.abs(), z.im.abs()) + self.shift;
        z.powi(2) + c
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, c: Self::Param) -> (Self::Var, Self::Deriv)
    {
        (z.powi(2) + c, 2. * z)
    }

    #[inline]
    fn gradient(&self, z: Self::Var, c: Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        let (f, df) = self.map_and_multiplier(z, c);
        (f, df, ONE)
    }

    #[inline]
    fn param_map(&self, c: Cplx) -> Cplx
    {
        c
    }

    #[inline]
    fn start_point(&self, _point: Cplx, _c: Self::Param) -> Self::Var
    {
        ZERO
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

    fn default_julia_bounds(&self, _point: Cplx, _c: Self::Param) -> Bounds
    {
        Bounds::centered_square(2.5 + self.shift.norm())
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SailboatParam
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl SailboatParam
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -2.5,
        max_x: 2.5,
        min_y: -2.5,
        max_y: 2.5,
    };
}
impl Default for SailboatParam
{
    fractal_impl!();
}

impl ParameterPlane for SailboatParam
{
    type Param = Cplx;
    type MetaParam = NoParam;
    type Var = Cplx;
    type Deriv = Cplx;
    type Child = Sailboat;

    basic_plane_impl!();
    default_bounds!();

    #[inline]
    fn map(&self, z: Self::Var, a: Self::Param) -> Self::Var
    {
        let z = Cplx::new(z.re.abs(), z.im.abs()) + a;
        z.powi(2)
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, a: Self::Param) -> (Self::Var, Self::Deriv)
    {
        let z = Cplx::new(z.re.abs(), z.im.abs());
        (z.powi(2) + a, 2. * z)
    }

    #[inline]
    fn start_point(&self, _point: Cplx, c: Self::Param) -> Self::Var
    {
        -TWO_THIRDS * c
    }

    #[inline]
    fn start_point_d(&self, _point: Cplx, c: Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        (-TWO_THIRDS * c, ZERO, Cplx::new(-TWO_THIRDS, 0.))
    }

    fn critical_points_child(&self, _param: Self::Param) -> Vec<Self::Var>
    {
        vec![ZERO, ONE]
    }

    fn name(&self) -> String
    {
        "Sailboat Param".to_owned()
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

impl<const N: Period> InfinityFirstReturnMap for BurningShip<N>
{
    degree_impl!(2);
}

impl<const N: Period> EscapeEncoding for BurningShip<N> {}
impl<const N: Period> ExternalRays for BurningShip<N> {}

degree_impl!(Sailboat, 2);
degree_impl!(SailboatParam, 2);

impl ToChildParam<Cplx> for SailboatParam
{
    fn to_child_param(&self, c: Cplx) -> Cplx
    {
        c
    }
}
