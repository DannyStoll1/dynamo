use crate::macros::*;
use crate::types::{
    variables::{Matrix2x2, Point},
    ComplexNum,
};

profile_imports!();

fn df_dz(z: Point, c: Point) -> Matrix2x2
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

fn df_dc(z: Point, _c: Point) -> Matrix2x2
{
    Matrix2x2::diag(1. / z.x.mul_add(z.x, 1.), -z.x - 1.)
}

fn f(z: Point, c: Point) -> Point
{
    Point {
        x: c.x / z.x.mul_add(z.x, 1.) + z.y,
        y: c.y.mul_add(-z.x - 1., z.y),
    }
}

#[derive(Clone, Debug)]
pub struct Rulkov
{
    point_grid: PointGrid,
    max_iter: Period,
}
impl Rulkov
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -5.,
        max_x: 5.,
        min_y: -5.,
        max_y: 5.,
    };
    fractal_impl!();
}

impl ParameterPlane for Rulkov
{
    type Var = Point;
    type Param = Point;
    type Deriv = Matrix2x2;

    point_grid_getters!();
    default_name!();

    fn param_map(&self, point: ComplexNum) -> Self::Param
    {
        Point {
            x: point.re,
            y: point.im,
        }
    }

    #[inline]
    fn map(&self, z: Self::Var, c: Self::Param) -> Self::Var
    {
        f(z, c)
    }

    #[inline]
    fn dynamical_derivative(&self, z: Self::Var, c: Self::Param) -> Self::Deriv
    {
        df_dz(z, c)
    }

    #[inline]
    fn parameter_derivative(&self, z: Self::Var, c: Self::Param) -> Self::Deriv
    {
        df_dc(z, c)
    }

    #[inline]
    fn start_point(&self, _point: ComplexNum, param: Self::Param) -> Self::Var
    {
        let mut z = Point { x: 0.5, y: 1.5 };
        for _ in 0..10000
        {
            z = f(z, param);
        }
        z
    }

    fn set_max_iter(&mut self, new_max_iter: Period)
    {
        self.max_iter = new_max_iter;
    }

    fn max_iter(&self) -> Period
    {
        self.max_iter
    }

    fn max_iter_mut(&mut self) -> &mut Period
    {
        &mut self.max_iter
    }
}
