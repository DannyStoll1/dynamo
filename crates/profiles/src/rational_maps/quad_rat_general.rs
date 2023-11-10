use crate::macros::horner;
use crate::macros::{horner_monic, profile_imports};
use dynamo_common::types::CplxPair;
profile_imports!();

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct QuadRatGeneral
{
    pub point_grid: PointGrid,
    pub max_iter: Period,
}

impl Default for QuadRatGeneral
{
    fn default() -> Self
    {
        let bounds = Bounds::centered_square(3.);
        let point_grid = PointGrid::new_by_res_y(1024, bounds);
        Self {
            point_grid,
            max_iter: 1024,
        }
    }
}

impl DynamicalFamily for QuadRatGeneral
{
    type Var = Cplx;
    type Param = CplxPair;
    type Deriv = Cplx;
    type MetaParam = NoParam;
    basic_plane_impl!();
    default_name!();

    fn map(&self, z: Self::Var, CplxPair { a, b }: &Self::Param) -> Self::Var
    {
        let z2 = z.powi(2);
        (z2 + a) / (z2 + b)
    }

    fn param_map(&self, t: Cplx) -> Self::Param
    {
        CplxPair { a: t, b: -ONE }
    }

    fn map_and_multiplier(
        &self,
        z: Self::Var,
        CplxPair { a, b }: &Self::Param,
    ) -> (Self::Var, Self::Deriv)
    {
        let z2 = z.powi(2);
        let denom = (z2 + b).inv();
        ((z2 + a) * denom, 2. * z * (b - a) * denom.powi(2))
    }

    fn start_point(&self, _point: Cplx, _c: &Self::Param) -> Self::Var
    {
        ZERO
    }
}

// default_bounds_impl!(QuadRatGeneral, Bounds::centered_square(3.));

impl MarkedPoints for QuadRatGeneral
{
    fn critical_points_child(&self, _c: &Self::Param) -> Vec<Self::Var>
    {
        vec![ZERO]
    }

    fn cycles_child(&self, CplxPair { a, b }: &Self::Param, period: Period) -> Vec<Self::Var>
    {
        match period {
            1 => solve_cubic(-a, *b, -ONE).to_vec(),
            2 => {
                let denom = (b + 1.).inv();
                solve_quadratic((a + b.powi(2)) * denom, (b - a) * denom).to_vec()
            }
            3 => {
                let b2 = b.powi(2);
                let b3 = b2 * b;
                let b4 = b2.powi(2);
                let b5 = b3 * b2;
                let u = (b + 1.).powi(2);
                let coeffs = [
                    horner_monic!(a, b3.powi(2), b4, 2. * (b2 + b3), 1.),
                    -horner_monic!(a, -b5, b4 - 2. * b3, 2. * b2 - b),
                    horner!(
                        a,
                        3. * b5 + b4,
                        4. * b3 + 3. * b2,
                        4. * b2 + 3. * b + 3.,
                        3.
                    ),
                    horner_monic!(a, 2. * b4 + b3, 2. * (b - b3) + 3. * b2, -2. - 5. * b),
                    horner!(a, 3. * (b4 + b3) + b2, 3. * b2 + 4. * b + 3., 3. * b + 4.),
                    horner!(a, b * u, -u),
                    horner_monic!(a, b * u + 1., 2.),
                ];
                solve_polynomial(coeffs)
            }
            4 => {
                let b2 = b.powi(2);
                let b3 = b * b2;
                let b4 = b2.powi(2);
                let b5 = b2 * b3;
                let b6 = b3.powi(2);
                let b7 = b3 * b4;
                let b8 = b4.powi(2);
                let b9 = b4 * b5;
                let b11 = b5 * b6;
                let b12 = b6.powi(2);
                let u = 1. + 2. * b;
                let coeffs = [
                    b12 + a
                        * a
                        * horner_monic!(
                            a,
                            b8 * (2. + 6. * b),
                            b6 * (3. + 2. * b),
                            b4 * (3. + 4. * b + 11. * b2),
                            b2 * (3. + 4. * b + 7. * b2),
                            1. + b2 * (5. + 6. * b),
                            3. + 2. * b
                        ),
                    a * horner!(
                        a,
                        2. * b9,
                        b7 * (1. - 4. * b),
                        b5 * (2. + 3. * b + 2. * b2),
                        b3 * (1. - 2. * b - 9. * b2),
                        b2 * (5. * b2 - 1.),
                        -b * u,
                        u
                    ),
                    horner!(
                        a,
                        6. * b11,
                        b8 * (1. + 6. * b),
                        b6 * horner!(b, 5., 15., 30.),
                        b4 * horner!(b, 6., 22., 33.),
                        b2 * horner!(b, 12., 29., 36., 41.),
                        horner!(b, 6., 6., 27., 46.),
                        horner!(b, 18., 21., 11.),
                        7.
                    ),
                    horner_monic!(
                        a,
                        b9,
                        7. * b8,
                        b5 * horner!(b, 4., 16., -17.),
                        b3 * horner!(b, 4., 4., -18., 9.),
                        b2 * horner!(b, -2., -7., -12.),
                        b * horner!(b, -8., -13., 14.),
                        6. + 11. * b
                    ),
                    horner!(
                        a,
                        b9 * horner!(b, 3., 15.),
                        b6 * horner!(b, 4., 8., 24.),
                        b4 * horner!(b, 3., 24., 63., 63.),
                        b2 * horner!(b, 18., 52., 80., 90.),
                        horner!(b, 15., 24., 78., 119., 64.),
                        horner!(b, 48., 66., 66.),
                        horner!(b, 26., 7.)
                    ),
                    horner!(
                        a,
                        4. * b8,
                        b5 * horner!(b, 2., 8., 8.),
                        b3 * horner!(b, 6., 14., 22., -28.),
                        b2 * 2. - b4 * 54. + b5 * 16.,
                        b * horner!(b, -22., -44., 10.),
                        horner!(b, 14., 22., 14.),
                        6.
                    ),
                    horner_monic!(
                        a,
                        b6 + b8 * (12. + 20. * b),
                        b5 * horner!(b, 16., 34., 36.),
                        b2 * horner!(b, 12., 34., 55., 106., 72.),
                        horner!(b, 20., 36., 106., 160., 106.),
                        horner!(b, 72., 106., 127., 54.),
                        horner!(b, 54., 40.)
                    ),
                    horner!(
                        a,
                        b6 * horner!(b, 2., 6.),
                        b3 * horner!(b, 4., 8., 14., 2.),
                        8. * b2 + 22. * (b3 - b5),
                        b * horner!(b, -28., -54., -44., 14.),
                        horner!(b, 16., 10., 22.),
                        14. + 6. * b
                    ),
                    horner!(
                        a,
                        b5 * horner!(b, 4., 3., 18., 15.),
                        b2 * horner!(b, 3., 8., 24., 52., 24.),
                        horner!(b, 15., 24., 63., 80., 78., 48.),
                        horner!(b, 63., 90., 119., 66.),
                        horner!(b, 64., 66., 26.),
                        7.
                    ),
                    horner_monic!(
                        a,
                        b3 + 4. * (b5 + b6),
                        b2 * horner!(b, 7., 16., 4., -2.),
                        b * horner!(b, -17., -18., -7., -8.),
                        horner!(b, 9., -12., -13., 6.),
                        14. + 11. * b
                    ),
                    horner!(
                        a,
                        b3 * horner!(b, 1., 5., 6., 12., 6.),
                        horner!(b, 6., 6., 15., 22., 29., 6.),
                        horner!(b, 30., 33., 36., 27., 18.),
                        horner!(b, 41., 46., 21.),
                        horner!(b, 11., 7.)
                    ),
                    horner!(
                        a,
                        b2 * horner!(b, 2., 1., 2., 1.),
                        -b * horner_monic!(b, 4., -3., 2.),
                        2. - 9. * b - b3,
                        5. - 2. * b + b2,
                        2.
                    ),
                    horner_monic!(
                        a,
                        1. + b2 * horner_monic!(b, 2., 3., 3., 3.),
                        horner!(b, 6., 2., 4., 4.),
                        horner!(b, 11., 7., 5., 3.),
                        2. * b + 6.
                    ),
                ];
                solve_polynomial(coeffs)
            }
            _ => vec![],
        }
    }
}
