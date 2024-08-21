use crate::macros::{degree_impl, profile_imports};
use dynamo_common::types::variables::Matrix2x2;
profile_imports!();

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BurningShip<const N: Period>
{
    point_grid: PointGrid,
    compute_mode: ComputeMode,
    max_iter: IterCount,
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

impl<const D: Period> HasChild<Sailboat<D>> for BurningShip<D>
{
    fn to_child_param(
        param: Self::Param,
    ) -> <<Sailboat<D> as DynamicalFamily>::MetaParam as ParamList>::Param
    {
        param
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
pub struct Sailboat<const N: Period>
{
    point_grid: PointGrid,
    compute_mode: ComputeMode,
    max_iter: IterCount,
    shift: Cplx,
}

impl<const N: Period> Sailboat<N>
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -4.,
        max_x: 2.,
        min_y: -3.,
        max_y: 3.,
    };
    const ASPECT: Real = Self::DEFAULT_BOUNDS.aspect_ratio();
}

impl<const N: Period> Default for Sailboat<N>
{
    fractal_impl!(shift, ZERO);
}

impl<const N: Period> DynamicalFamily for Sailboat<N>
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

    fn get_param(&self) -> <Self::MetaParam as ParamList>::Param
    {
        self.shift
    }

    #[inline]
    fn name(&self) -> String
    {
        let shift = self.shift;
        format!("Sailboat({shift})")
    }
}

impl<const N: Period> FamilyDefaults for Sailboat<N>
{
    fn default_bounds(&self) -> Bounds
    {
        let center = Self::DEFAULT_BOUNDS.center();
        let rad_y_0 = self.shift.im.abs() + Self::DEFAULT_BOUNDS.range_y() / 2.0;
        let rad_x_1 = self.shift.re.abs() + Self::DEFAULT_BOUNDS.range_x() / 2.0;

        let rad_y_1 = rad_x_1 * Self::ASPECT;

        if rad_y_0 <= rad_y_1 {
            Bounds::rect(rad_x_1, rad_y_1, center)
        } else {
            let rad_x = rad_y_0 / Self::ASPECT;
            Bounds::rect(rad_x, rad_y_0, center)
        }
    }
}

impl<const N: Period> HasJulia for Sailboat<N>
{
    fn default_bounds_child(&self, _point: Cplx, _c: &Self::Param) -> Bounds
    {
        Bounds::centered_square(2.5 + self.shift.norm())
    }
}

impl<const N: Period> From<BurningShip<N>> for Sailboat<N>
{
    fn from(fractal: BurningShip<N>) -> Self
    {
        Self::default()
            .with_res_y(fractal.point_grid.res_y)
            .with_param(fractal.default_selection())
    }
}

impl<const N: Period> InfinityFirstReturnMap for BurningShip<N>
{
    degree_impl!(i64::from(N));
}

impl<const N: Period> EscapeEncoding for BurningShip<N> {}
impl<const N: Period> ExternalRays for BurningShip<N> {}

impl<const N: Period> MarkedPoints for Sailboat<N>
{
    #[inline]
    fn critical_points_child(&self, _c: &Self::Param) -> Vec<Self::Var>
    {
        vec![ZERO]
    }
}

degree_impl!(Sailboat, i64::from(N); N: Period);
