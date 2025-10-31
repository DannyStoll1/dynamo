use dynamo_common::types::Cplx;
use dynamo_common::types::variables::{Matrix2x2, Point};

use crate::macros::{degree_impl, profile_imports};

profile_imports!();

fn df_dz(z: Point, c: &Point) -> Matrix2x2
{
    let v = z.x.mul_add(z.x, 1.);
    let df_dx = Point {
        x: -2. * c.x * z.x / (v * v),
        y: -c.y,
    };
    let df_dy = Point { x: 1., y: 1. };

    Matrix2x2 {
        v0: df_dx,
        v1: df_dy,
    }

    // Point {
    //     x: -2. * c.x * z.x / (v * v),
    //     y: -c.y,
    // }
}

fn df_dc(z: Point, _c: &Point) -> Matrix2x2
{
    Matrix2x2::diag(1. / z.x.mul_add(z.x, 1.), -z.x - 1.)
}

fn f(z: Point, c: &Point) -> Point
{
    Point {
        x: c.x / z.x.mul_add(z.x, 1.) + z.y,
        y: c.y.mul_add(-z.x - 1., z.y),
    }
}

#[derive(Clone, Debug)]
pub struct Rulkov
{
    point_grid:   PointGrid,
    compute_mode: ComputeMode,
    max_iter:     IterCount,
}
impl Rulkov
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -5.,
        max_x: 5.,
        min_y: -5.,
        max_y: 5.,
    };
}
impl Default for Rulkov
{
    fractal_impl!();
}

impl DynamicalFamily for Rulkov
{
    type Var = Point;
    type Param = Point;
    type Deriv = Matrix2x2;
    type MetaParam = NoParam;

    basic_plane_impl!();
    default_name!();

    fn param_map(&self, point: Cplx) -> Self::Param
    {
        Point {
            x: point.re,
            y: point.im,
        }
    }

    #[inline]
    fn map(&self, z: Self::Var, c: &Self::Param) -> Self::Var
    {
        f(z, c)
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv)
    {
        (f(z, c), df_dz(z, c))
    }

    #[inline]
    fn gradient(&self, z: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        (f(z, c), df_dz(z, c), df_dc(z, c))
    }

    #[inline]
    fn start_point(&self, _point: Cplx, param: &Self::Param) -> Self::Var
    {
        let mut z = Point { x: 0.5, y: 1.5 };
        for _ in 0..10000 {
            z = f(z, param);
        }
        z
    }
}

impl FamilyDefaults for Rulkov
{
    default_bounds!();
}

impl HasJulia for Rulkov {}

impl MarkedPoints for Rulkov {}
degree_impl!(Rulkov, 2);
