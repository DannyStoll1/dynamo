use std::collections::VecDeque;

use dynamo_common::math_utils::{binomial, nth_roots, roots_of_unity};

use crate::macros::{degree_impl, profile_imports};
profile_imports!();

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MinsikHanPhi<const D: i32>
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl<const D: i32> MinsikHanPhi<D>
{
    const D_FLOAT: Real = D as Real;
    const D_MINUS_1: Real = Self::D_FLOAT - 1.;
    const DEFAULT_BOUNDS: Bounds = Bounds::centered_square(12.);

    #[must_use]
    pub const fn new(point_grid: PointGrid, max_iter: Period) -> Self
    {
        Self {
            point_grid,
            max_iter,
        }
    }
}

impl<const D: i32> Default for MinsikHanPhi<D>
{
    fractal_impl!();
}

impl<const D: i32> DynamicalFamily for MinsikHanPhi<D>
{
    parameter_plane_impl!();

    fn map(&self, z: Self::Var, a: &Self::Param) -> Self::Var
    {
        let u = z.powi(D) + Self::D_MINUS_1;
        a * z / u
    }

    fn map_and_multiplier(&self, z: Self::Var, a: &Self::Param) -> (Self::Var, Self::Deriv)
    {
        let u = z.powi(D) + Self::D_MINUS_1;
        (
            a * z / u,
            -a * (Self::D_MINUS_1) * (u - Self::D_FLOAT) / u.powi(2),
        )
    }

    fn gradient(&self, z: Self::Var, a: &Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        let u = z.powi(D) + Self::D_MINUS_1;
        let v = z / u;
        (
            a * v,
            -a * (Self::D_MINUS_1) * (u - Self::D_FLOAT) / u.powi(2),
            v,
        )
    }

    fn start_point(&self, _point: Cplx, _a: &Self::Param) -> Self::Var
    {
        ONE
    }

    fn name(&self) -> String
    {
        format!("Minsik Han Family, degree {D}")
    }
}

impl<const D: i32> FamilyDefaults for MinsikHanPhi<D>
{
    default_bounds!();

    fn default_selection(&self) -> Cplx
    {
        Cplx::new(Self::D_FLOAT, 0.0)
    }
}

impl<const D: i32> HasChild for MinsikHanPhi<D>
{
    type Child = JuliaSet<Self>;
}

impl<const D: i32> MarkedPoints for MinsikHanPhi<D>
{
    fn critical_points_child(&self, _param: &Self::Param) -> Vec<Self::Var>
    {
        (0..D)
            .map(|k| (TAUI * f64::from(k) / Self::D_FLOAT).exp())
            .collect()
    }

    fn cycles_child(&self, c: &Self::Param, period: Period) -> Vec<Self::Var>
    {
        match period {
            1 => {
                let u = (c + 1. - Self::D_FLOAT).powf(1. / Self::D_FLOAT);
                roots_of_unity(D).map(|z| z * u).collect()
            }
            2 => {
                if D < 2 {
                    return vec![];
                }

                let u = c / Self::D_MINUS_1;

                let coeffs: VecDeque<Cplx> = (0..D)
                    .map(|i| {
                        if i == 0 {
                            return -u - 1.;
                        }
                        let mut val = ONE;
                        (1..=(D - i)).for_each(|j| {
                            val *= u;
                            val += Real::from(binomial(i + j - 1, j));
                        });
                        val -= u * Real::from(binomial(D - 1, i)) + Real::from(binomial(D, i));
                        val
                    })
                    .collect();

                solve_polynomial(coeffs)
                    .iter()
                    .map(|z| (z * Self::D_MINUS_1))
                    .flat_map(|z| nth_roots(z, D))
                    .collect()
            }
            _ => vec![],
        }
    }
}

impl<const D: i32> InfinityFirstReturnMap for MinsikHanPhi<D>
{
    degree_impl!(0);
}

impl<const D: i32> EscapeEncoding for MinsikHanPhi<D> {}
impl<const D: i32> ExternalRays for MinsikHanPhi<D> {}
