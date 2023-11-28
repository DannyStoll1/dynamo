use dynamo_color::{Coloring, IncoloringAlgorithm};

use crate::macros::*;
profile_imports!();

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CosineAdd
{
    point_grid: PointGrid,
    compute_mode: ComputeMode,
    max_iter: IterCount,
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
    fn start_point(&self, _point: Cplx, _c: &Self::Param) -> Self::Var
    {
        ZERO
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

impl FamilyDefaults for CosineAdd
{
    default_bounds!();
}

impl HasJulia for CosineAdd
{
    fn default_bounds_child(&self, _point: Cplx, _c: &Self::Param) -> Bounds
    {
        Bounds::centered_square(5.5)
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Cosine
{
    point_grid: PointGrid,
    compute_mode: ComputeMode,
    max_iter: IterCount,
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
    fn start_point(&self, _point: Cplx, _c: &Self::Param) -> Self::Var
    {
        ZERO
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

default_bounds_impl!(Cosine);
has_child_impl!(Cosine, 5.5);

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SineWander
{
    point_grid: PointGrid,
    compute_mode: ComputeMode,
    max_iter: IterCount,
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

impl FamilyDefaults for SineWander
{
    default_bounds!();

    fn default_selection(&self) -> Cplx
    {
        ONE
    }
}

has_child_impl!(SineWander, 5.5);

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

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CoshNewton
{
    point_grid: PointGrid,
    compute_mode: ComputeMode,
    max_iter: IterCount,
}

impl CoshNewton
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -7.,
        max_x: 7.,
        min_y: -7.,
        max_y: 7.,
    };
}

impl Default for CoshNewton
{
    fractal_impl!();
}

impl DynamicalFamily for CoshNewton
{
    type Var = Cplx;
    type Param = NoParam;
    type MetaParam = NoParam;
    type Deriv = Cplx;
    basic_plane_impl!();
    default_name!();

    #[inline]
    fn map(&self, z: Cplx, _: &NoParam) -> Cplx
    {
        let diff = z.tanh().inv();
        if diff.is_nan() {
            return z;
        }
        z - diff
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, _c: &Self::Param) -> (Self::Var, Self::Deriv)
    {
        let diff = z.tanh().inv();
        if diff.is_nan() {
            return (z, ZERO);
        }
        (z - diff, 1.0 + z.sinh().powi(-2))
    }

    #[inline]
    fn gradient(&self, z: Self::Var, _c: &Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        (z - z.tanh().inv(), z.sinh().powi(-2), ZERO)
    }

    #[inline]
    fn periodicity_tolerance(&self) -> Real
    {
        9e-4
    }

    #[inline]
    fn param_map(&self, _point: Cplx) -> Self::Param
    {
        NoParam
    }

    #[inline]
    fn param_map_d(&self, _point: Cplx) -> (Self::Param, Self::Deriv)
    {
        (NoParam, ZERO)
    }

    #[inline]
    fn start_point(&self, t: Cplx, _c: &Self::Param) -> Self::Var
    {
        t
    }

    #[inline]
    fn plane_type(&self) -> PlaneType
    {
        PlaneType::Dynamical
    }

    #[inline]
    fn internal_potential_coloring(&self) -> IncoloringAlgorithm
    {
        IncoloringAlgorithm::InternalPotential {
            periodicity_tolerance: self.periodicity_tolerance(),
            crit_degree: 3.3,
        }
    }
}

impl HasChild<Self> for CoshNewton
{
    fn to_child_param(param: Self::Param) -> <Self::MetaParam as ParamList>::Param
    {
        param
    }
}

impl FamilyDefaults for CoshNewton
{
    default_bounds!();
    fn default_coloring(&self) -> Coloring
    {
        Coloring::default().with_interior_algorithm(self.internal_potential_coloring())
    }
}

impl MarkedPoints for CoshNewton {}

degree_impl_transcendental!(Cosine);
degree_impl_transcendental!(CosineAdd);
degree_impl_transcendental!(SineWander);
degree_impl_transcendental!(CoshNewton);
