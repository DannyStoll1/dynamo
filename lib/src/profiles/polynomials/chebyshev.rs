use crate::{macros::*, math_utils::solve_quadratic};
profile_imports!();
use std::iter::once;

#[derive(Clone, Debug)]
pub struct ChebyshevCoeffTable
{
    pub coeffs: Vec<Vec<i32>>,
}
impl Default for ChebyshevCoeffTable
{
    fn default() -> Self
    {
        let coeffs = vec![vec![1], vec![1]];
        Self { coeffs }
    }
}

impl ChebyshevCoeffTable
{
    #[must_use]
    pub fn new(max_degree: usize) -> Self
    {
        Self::default().with_max_degree(max_degree)
    }

    pub fn extend(&mut self)
    {
        let n = self.coeffs.len();
        let coeff0 = self.coeffs[n - 2].iter().chain(once(&0));

        let new_coeff: Vec<i32> = if n % 2 == 0
        {
            let coeff1 = once(&0).chain(self.coeffs[n - 1].iter());
            coeff0.zip(coeff1).map(|(&a, &b)| 2 * b - a).collect()
        }
        else
        {
            let coeff1 = self.coeffs[n - 1].iter();
            coeff0.zip(coeff1).map(|(&a, &b)| 2 * b - a).collect()
        };

        self.coeffs.push(new_coeff);
    }

    pub fn extend_to(&mut self, degree: usize)
    {
        while self.coeffs.len() <= degree
        {
            self.extend();
        }
    }

    pub fn with_max_degree(mut self, degree: usize) -> Self
    {
        self.extend_to(degree);
        self
    }

    pub fn coefficients(&mut self, degree: usize) -> &Vec<i32>
    {
        self.extend_to(degree);
        &self.coeffs[degree]
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Chebyshev<const D: Period>
{
    point_grid: PointGrid,
    max_iter: Period,
    coeffs: Vec<Real>,
    coeffs_d: Vec<Real>,
}

impl<const D: Period> Default for Chebyshev<D>
{
    fn default() -> Self
    {
        let bounds = Bounds::centered_square(3.0 / (D as f64));
        let point_grid = PointGrid::new_by_res_y(1024, bounds);

        let sign = 1 - 2 * (D as i32 % 2);

        let coeffs: Vec<Real> = ChebyshevCoeffTable::new(2 * D as usize)
            .coefficients(2 * D as usize)
            .iter()
            .map(|&x| (sign * x) as Real)
            .collect();
        let coeffs_d: Vec<Real> = coeffs
            .iter()
            .enumerate()
            .skip(1)
            .map(|(k, a)| (k as Real) * a)
            .collect();
        Self {
            point_grid,
            max_iter: 1024,
            coeffs,
            coeffs_d,
        }
    }
}

impl<const D: Period> ParameterPlane for Chebyshev<D>
{
    parameter_plane_impl!();
    basic_escape_encoding!((2 * D) as Real, 1);
    default_name!();

    fn map(&self, z: Self::Var, c: Self::Param) -> Self::Var
    {
        let z2 = z * z * 0.25;

        let mut z_iter = self.coeffs.iter().rev();

        let an = *z_iter.next().unwrap_or(&0.0);

        let mut zval = Cplx::from(an);

        for &a in z_iter
        {
            zval = zval * z2 + a;
        }

        c * zval
    }

    fn map_and_multiplier(&self, z: Self::Var, c: Self::Param) -> (Self::Var, Self::Deriv)
    {
        let w = z * 0.5;
        let w2 = w * w;

        let mut z_iter = self.coeffs.iter().rev();
        let mut d_iter = self.coeffs_d.iter().rev();

        let an = *z_iter.next().unwrap_or(&0.0);
        let bn = *d_iter.next().unwrap_or(&0.0);

        let mut zval = Cplx::from(an);
        let mut dval = Cplx::from(bn);

        for &a in z_iter
        {
            zval = zval * w2 + a;
        }
        for &b in d_iter
        {
            dval = dval * w2 + b;
        }

        (c * zval, c * dval * w)
    }

    fn dynamical_derivative(&self, z: Self::Var, c: Self::Param) -> Self::Deriv
    {
        let z2 = z * z * 0.25;
        let mut iter = self.coeffs_d.iter().rev();

        let an = *iter.next().unwrap_or(&0.0);
        let mut result = Cplx::from(an);

        for &a in iter
        {
            result = result * z2 + a;
        }
        c * result * z
    }

    fn parameter_derivative(&self, _z: Self::Var, _c: Self::Param) -> Self::Deriv
    {
        ONE
    }

    fn critical_points_child(&self, _c: Self::Param) -> Vec<Self::Var>
    {
        vec![ZERO]
    }

    fn start_point(&self, _point: Cplx, _c: Self::Param) -> Self::Var
    {
        ZERO
    }
}
