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
    const DEFAULT_BOUNDS: Bounds = match N {
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

impl<const N: Period> DynamicalFamily for BurningShip<N>
{
    type Var = Cplx;
    type Param = Cplx;
    type MetaParam = NoParam;
    // type Deriv = Matrix2x2;
    type Deriv = Cplx;
    basic_plane_impl!();
    default_name!();

    #[inline]
    fn map(&self, z: Cplx, c: &Cplx) -> Cplx
    {
        let z = Cplx::new(z.re.abs(), z.im.abs());
        z.powf(Self::N_FLOAT) + c
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv)
    {
        let z = Cplx::new(z.re.abs(), z.im.abs());
        let znm1 = z.powf(Self::N_MINUS_1);
        (znm1 * z + c, Self::N_FLOAT * znm1)
    }

    #[inline]
    fn gradient(&self, z: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        let (f, df) = self.map_and_multiplier(z, c);
        (f, df, ONE)
    }

    #[inline]
    fn start_point(&self, _point: Cplx, _c: &Self::Param) -> Self::Var
    {
        ZERO
    }

    #[inline]
    fn param_map(&self, c: Cplx) -> Cplx
    {
        c
    }
}

impl<const D: Period> FamilyDefaults for BurningShip<D>
{
    default_bounds!();
}

impl<const D: Period> HasJulia for BurningShip<D>
{
    fn default_bounds_child(&self, _point: Cplx, _c: &Self::Param) -> Bounds
    {
        Bounds::centered_square(4.)
    }
}

impl<const N: Period> MarkedPoints for BurningShip<N>
{
    #[inline]
    fn critical_points_child(&self, _param: &Self::Param) -> Vec<Self::Var>
    {
        vec![ZERO]
    }

    #[inline]
    fn cycles_child(&self, c: &Self::Param, period: Period) -> Vec<Self::Var>
    {
        match period {
            1 => {
                let disc = (1. - 4. * c).sqrt();
                vec![0.5 * (1. - disc), 0.5 * (1. + disc)]
            }
            _ => vec![],
        }
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
    const DEFAULT_BOUNDS: Bounds = Bounds::centered_square(3.5);
}
impl Default for Sailboat
{
    fractal_impl!(shift, ZERO);
}

impl DynamicalFamily for Sailboat
{
    type Var = Cplx;
    type Param = Cplx;
    type MetaParam = Cplx;
    type Deriv = Cplx;
    basic_plane_impl!();

    #[inline]
    fn map(&self, z: Cplx, c: &Cplx) -> Cplx
    {
        let z = Cplx::new(z.re.abs(), z.im.abs()) + self.shift;
        z.powi(2) + c
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv)
    {
        let z = Cplx::new(z.re.abs(), z.im.abs()) + self.shift;
        (z.powi(2) + c, 2. * z)
    }

    #[inline]
    fn gradient(&self, z: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
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
    fn start_point(&self, _point: Cplx, c: &Self::Param) -> Self::Var
    {
        *c
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

impl FamilyDefaults for Sailboat
{
    default_bounds!();
}

impl HasJulia for Sailboat
{
    fn default_bounds_child(&self, _point: Cplx, _c: &Self::Param) -> Bounds
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

impl DynamicalFamily for SailboatParam
{
    parameter_plane_impl!();

    #[inline]
    fn map(&self, z: Self::Var, a: &Self::Param) -> Self::Var
    {
        let z = Cplx::new(z.re.abs(), z.im.abs());
        z.powi(2) + a
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, a: &Self::Param) -> (Self::Var, Self::Deriv)
    {
        let z = Cplx::new(z.re.abs(), z.im.abs());
        (z.powi(2) + a, 2. * z)
    }

    #[inline]
    fn start_point(&self, _point: Cplx, c: &Self::Param) -> Self::Var
    {
        *c
    }

    #[inline]
    fn start_point_d(&self, _point: Cplx, c: &Self::Param)
        -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        (*c, ZERO, ONE)
    }

    #[inline]
    fn name(&self) -> String
    {
        "Sailboat Param".to_owned()
    }
}

impl FamilyDefaults for SailboatParam
{
    default_bounds!();

    fn default_selection(&self) -> Cplx
    {
        ZERO
    }
}

impl HasChild<Sailboat> for SailboatParam
{
    fn to_child_param(
        param: Self::Param,
    ) -> <<Sailboat as DynamicalFamily>::MetaParam as ParamList>::Param
    {
        param
    }
}

impl MarkedPoints for SailboatParam
{
    fn critical_points_child(&self, _param: &Self::Param) -> Vec<Self::Var>
    {
        vec![ZERO, ONE]
    }
}

impl From<SailboatParam> for Sailboat
{
    fn from(parent: SailboatParam) -> Self
    {
        let point = parent.default_selection();
        let param = parent.param_map(point);
        let point_grid = parent.point_grid().clone();
        Self {
            point_grid,
            max_iter: parent.max_iter(),
            shift: param,
        }
        .with_default_bounds()
    }
}

impl<const N: Period> InfinityFirstReturnMap for BurningShip<N>
{
    degree_impl!(2);
}

impl<const N: Period> EscapeEncoding for BurningShip<N> {}
impl<const N: Period> ExternalRays for BurningShip<N> {}

impl MarkedPoints for Sailboat
{
    #[inline]
    fn critical_points_child(&self, _c: &Self::Param) -> Vec<Self::Var>
    {
        vec![ZERO]
    }
}

degree_impl!(Sailboat, 2);
degree_impl!(SailboatParam, 2);
