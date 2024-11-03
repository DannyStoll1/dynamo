use dynamo_color::{Coloring, IncoloringAlgorithm};

use crate::macros::{
    default_bounds, default_bounds_impl, default_name, degree_impl_transcendental, fractal_impl,
    has_child_impl, profile_imports,
};
profile_imports!();

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Gudermannian
{
    point_grid: PointGrid,
    compute_mode: ComputeMode,
    max_iter: IterCount,
}

impl Gudermannian
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -7.,
        max_x: 7.,
        min_y: -7.,
        max_y: 7.,
    };
}
impl Default for Gudermannian
{
    fractal_impl!();
}

impl DynamicalFamily for Gudermannian
{
    parameter_plane_impl!();
    default_name!();

    #[inline]
    fn map(&self, z: Cplx, lambda: &Cplx) -> Cplx
    {
        z.sinh().atan() + lambda
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, lambda: &Self::Param) -> (Self::Var, Self::Deriv)
    {
        (z.sinh().atan() + lambda, z.cosh().inv())
    }

    #[inline]
    fn gradient(&self, z: Self::Var, lambda: &Self::Param)
        -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        let gd = z.sinh().atan();
        (gd + lambda, z.cosh().inv(), ONE)
    }

    #[inline]
    fn start_point(&self, _point: Cplx, lambda: &Self::Param) -> Self::Var
    {
        const QTR: f64 = TAU / 4.0;
        QTR + lambda
    }

    #[inline]
    fn extra_stop_condition(
        &self,
        z: Self::Var,
        _c: &Self::Param,
        iter: IterCount,
    ) -> Option<EscapeResult<Self::Var, Self::Deriv>>
    {
        if z.im.abs() > 350. {
            Some(EscapeResult::Escaped {
                iters: iter,
                final_value: z,
            })
        } else if z.re.abs() > 1e15 {
            Some(EscapeResult::Unknown)
        } else {
            None
        }
    }
}

default_bounds_impl!(Gudermannian);
has_child_impl!(Gudermannian, 5.5);

impl MarkedPoints for Gudermannian
{
    #[allow(clippy::cast_sign_loss)]
    fn critical_points_child(&self, _c: &Self::Param) -> Vec<Self::Var>
    {
        if self.point_grid().min_y > 0.0 || self.point_grid().max_y < 0.0 {
            return vec![];
        }

        let n_min = (self.point_grid().min_y / PI + 0.5).ceil() as i32;
        let n_max = (self.point_grid().max_y / PI + 0.5).ceil() as i32;

        let mut pts = Vec::with_capacity((n_max - n_min) as usize);

        for n in n_min..n_max {
            pts.push(Cplx::new(0.0, (Real::from(n) - 0.5) * PI));
        }
        pts
    }
}

degree_impl_transcendental!(Gudermannian);
