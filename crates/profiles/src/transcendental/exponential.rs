use std::f64::consts::PI;

use crate::macros::{degree_impl_transcendental, profile_imports};
use dynamo_common::math_utils::slog;
profile_imports!();

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Exponential
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl Exponential
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -7.,
        max_x: 7.,
        min_y: -7.,
        max_y: 7.,
    };
}
impl Default for Exponential
{
    fractal_impl!();
}

impl DynamicalFamily for Exponential
{
    parameter_plane_impl!();
    default_name!();
    default_bounds!();

    #[inline]
    fn map(&self, z: Cplx, lambda: &Cplx) -> Cplx
    {
        z.exp() * lambda
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, lambda: &Self::Param) -> (Self::Var, Self::Deriv)
    {
        let u = z.exp() * lambda;
        (u, u)
    }

    #[inline]
    fn extra_stop_condition(
        &self,
        z: Self::Var,
        _c: &Self::Param,
        iter: Period,
    ) -> Option<EscapeResult<Self::Var, Self::Deriv>>
    {
        if z.re > 250. {
            Some(EscapeResult::Escaped {
                iters: iter,
                final_value: z,
            })
        } else if z.re < -50. {
            None
        } else if z.im.abs() > 1e15 {
            Some(EscapeResult::Unknown)
        } else {
            None
        }
    }

    #[inline]
    fn gradient(&self, z: Cplx, lambda: &Cplx) -> (Cplx, Cplx, Cplx)
    {
        let u = z.exp();
        let v = lambda * u;
        (v, v, u)
    }

    #[inline]
    fn param_map(&self, lambda: Cplx) -> Cplx
    {
        lambda
    }

    #[inline]
    fn start_point(&self, _point: Cplx, _c: &Self::Param) -> Self::Var
    {
        ZERO
    }

    fn default_julia_bounds(&self, _point: Cplx, lambda: &Self::Param) -> Bounds
    {
        Bounds::square(5., *lambda)
    }
}

impl MarkedPoints for Exponential
{
    #[inline]
    fn critical_points_child(&self, _param: &Self::Param) -> Vec<Self::Var>
    {
        vec![ZERO]
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CosineAdd
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl CosineAdd
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -7.,
        max_x: 7.,
        min_y: -7.,
        max_y: 7.,
    };
}

impl Default for CosineAdd
{
    fractal_impl!();
}

impl DynamicalFamily for CosineAdd
{
    parameter_plane_impl!();
    default_name!();
    default_bounds!();

    #[inline]
    fn map(&self, z: Cplx, c: &Cplx) -> Cplx
    {
        z.cos() + c
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv)
    {
        (z.cos() + c, -z.sin())
    }

    #[inline]
    fn gradient(&self, z: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        (z.cos() + c, -z.sin(), ONE)
    }

    #[inline]
    fn param_map(&self, t: Cplx) -> Cplx
    {
        t
    }

    #[inline]
    fn param_map_d(&self, t: Cplx) -> (Self::Param, Self::Deriv)
    {
        (t, ONE)
    }

    #[inline]
    fn start_point(&self, _point: Cplx, _c: &Self::Param) -> Self::Var
    {
        ZERO
    }

    fn default_julia_bounds(&self, _point: Cplx, _c: &Self::Param) -> Bounds
    {
        Bounds::centered_square(5.5)
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Cosine
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl Cosine
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -7.,
        max_x: 7.,
        min_y: -7.,
        max_y: 7.,
    };
}
impl Default for Cosine
{
    fractal_impl!();
}

impl DynamicalFamily for Cosine
{
    parameter_plane_impl!();
    default_name!();
    default_bounds!();

    #[inline]
    fn map(&self, z: Cplx, lambda: &Cplx) -> Cplx
    {
        z.cos() * lambda
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, lambda: &Self::Param) -> (Self::Var, Self::Deriv)
    {
        (z.cos() * lambda, -z.sin() * lambda)
    }

    #[inline]
    fn gradient(&self, z: Self::Var, lambda: &Self::Param)
        -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        let cos = z.cos();
        (cos * lambda, -z.sin() * lambda, cos)
    }

    #[inline]
    fn param_map(&self, lambda: Cplx) -> Cplx
    {
        lambda
    }

    #[inline]
    fn start_point(&self, _point: Cplx, _c: &Self::Param) -> Self::Var
    {
        ZERO
    }

    fn default_julia_bounds(&self, _point: Cplx, _param: &Self::Param) -> Bounds
    {
        Bounds::centered_square(5.5)
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SineWander
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl SineWander
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -7.,
        max_x: 7.,
        min_y: -7.,
        max_y: 7.,
    };
}

impl Default for SineWander
{
    fractal_impl!();
}

impl DynamicalFamily for SineWander
{
    parameter_plane_impl!();
    default_name!();
    default_bounds!();

    #[inline]
    fn map(&self, z: Cplx, c: &Cplx) -> Cplx
    {
        z.sin() + z + TAU * c
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv)
    {
        (z.sin() + z + TAU * c, z.cos() + 1.)
    }

    #[inline]
    fn gradient(&self, z: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        (z.sin() + z + TAU * c, z.cos() + 1., TAU.into())
    }

    #[inline]
    fn start_point(&self, _point: Cplx, _c: &Self::Param) -> Self::Var
    {
        PI.into()
    }

    fn default_selection(&self) -> Cplx
    {
        ONE
    }

    fn default_julia_bounds(&self, _point: Cplx, _param: &Self::Param) -> Bounds
    {
        Bounds::centered_square(5.5)
    }
}

impl MarkedPoints for Cosine
{
    fn critical_points_child(&self, _c: &Self::Param) -> Vec<Self::Var>
    {
        if self.point_grid().min_y > 0.0 || self.point_grid().max_y < 0.0 {
            return vec![];
        }

        let n_min = (self.point_grid().min_x / PI).ceil() as i32;
        let n_max = (self.point_grid().max_x / PI).ceil() as i32;

        let mut pts = Vec::with_capacity((n_max - n_min) as usize);

        for n in n_min..n_max {
            pts.push((n as Real * PI).into());
        }
        pts
    }
}

impl MarkedPoints for CosineAdd
{
    fn critical_points_child(&self, _c: &Self::Param) -> Vec<Self::Var>
    {
        if self.point_grid().min_y > 0.0 || self.point_grid().max_y < 0.0 {
            return vec![];
        }

        let n_min = (self.point_grid().min_x / PI).ceil() as i32;
        let n_max = (self.point_grid().max_x / PI).ceil() as i32;

        let mut pts = Vec::with_capacity((n_max - n_min) as usize);

        for n in n_min..n_max {
            pts.push((n as Real * PI).into());
        }
        pts
    }
}

impl MarkedPoints for SineWander
{
    fn critical_points_child(&self, _c: &Self::Param) -> Vec<Self::Var>
    {
        if self.point_grid().min_y > 0.0 || self.point_grid().max_y < 0.0 {
            return vec![];
        }

        let n_min = (self.point_grid().min_x / TAU).ceil() as i32;
        let n_max = (self.point_grid().max_x / TAU).ceil() as i32;

        let mut pts = Vec::with_capacity((n_max - n_min + 1) as usize);

        for n in n_min..=n_max {
            pts.push((n as Real).mul_add(TAU, PI).into());
        }
        pts
    }
}

degree_impl_transcendental!(Exponential);
degree_impl_transcendental!(Cosine);
degree_impl_transcendental!(CosineAdd);
degree_impl_transcendental!(SineWander);
