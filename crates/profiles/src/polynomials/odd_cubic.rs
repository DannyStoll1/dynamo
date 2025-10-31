use dynamo_common::math_utils::weierstrass_p;
use dynamo_common::{horner, horner_monic};

use crate::macros::{degree_impl, profile_imports};
profile_imports!();

#[derive(Clone, Debug)]
pub struct OddCubic
{
    point_grid:   PointGrid,
    compute_mode: ComputeMode,
    max_iter:     IterCount,
}

impl OddCubic
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -1.6,
        max_x: 1.6,
        min_y: -1.3,
        max_y: 1.3,
    };
}
impl Default for OddCubic
{
    fractal_impl!();
}

#[allow(clippy::suspicious_operation_groupings)]
impl DynamicalFamily for OddCubic
{
    parameter_plane_impl!();
    default_name!();

    #[inline]
    fn map(&self, z: Cplx, c: &Cplx) -> Cplx
    {
        2. * z * (z.powi(2) / 3. - c)
    }

    #[inline]
    fn start_point(&self, _point: Cplx, c: &Cplx) -> Cplx
    {
        c.powf(0.5)
    }

    #[inline]
    fn start_point_d(&self, _point: Cplx, c: &Self::Param)
    -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        let z0 = c.powf(0.5);
        (z0, ZERO, 0.5 / z0)
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv)
    {
        let z2 = z.powi(2);
        (2. * z * (z2 / 3. - c), 2. * (z2 - c))
    }

    #[inline]
    fn gradient(&self, z: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        let z2 = z.powi(2);
        let u = 2. * z;
        (u * (z2 / 3. - c), 2. * (z2 - c), -u)
    }
}

impl FamilyDefaults for OddCubic
{
    default_bounds!();
}

impl HasJulia for OddCubic
{
    #[inline]
    fn default_bounds_child(&self, _point: Cplx, _param: &Cplx) -> Bounds
    {
        Bounds::centered_square(2.2)
    }
}

impl MarkedPoints for OddCubic
{
    #[inline]
    fn critical_points_child(&self, param: &Cplx) -> ComplexVec
    {
        let sqrt_c = param.sqrt();
        vec![-sqrt_c, sqrt_c]
    }

    #[inline]
    fn cycles_child(&self, c: &Self::Param, period: Period) -> Vec<Self::Var>
    {
        match period {
            1 => {
                let r1 = (3. * c + 1.5).sqrt();
                vec![ZERO, r1, -r1]
            }
            2 => {
                let r0 = (3. * c - 1.5).sqrt();
                let disc = (c * c - 1.).sqrt();
                let r2 = (1.5 * (c + disc)).sqrt();
                let r4 = (1.5 * (c - disc)).sqrt();
                vec![r0, -r0, r2, -r2, r4, -r4]
            }
            3 => {
                let u = -(c + c);
                let coeffs = [
                    horner_monic!(u, 1., 1.),
                    horner_monic!(u, 1., 2., 2., 2., 1.),
                    horner!(u, 1., 3., 5., 4., 5., 3., 3.),
                    horner!(u, 1., 4., 6., 10., 12., 15., 3., 3.),
                    horner_monic!(u, 1., 4., 10., 19., 31., 16., 19., 1.),
                    horner!(u, 1., 5., 15., 34., 35., 51., 7., 8.),
                    horner!(u, 1., 6., 21., 40., 75., 21., 28.),
                    horner!(u, 1., 7., 25., 65., 35., 56.),
                    horner!(u, 1., 8., 33., 35., 70.),
                    horner!(u, 1., 9., 21., 56.),
                    horner!(u, 1., 7., 28.),
                    horner!(u, 1., 8.),
                    ONE,
                ];
                let squared_sols = solve_polynomial(coeffs);

                squared_sols
                    .iter()
                    .flat_map(|w| {
                        let z = (1.5 * w).sqrt();
                        [z, -z]
                    })
                    .collect()
            }
            _ => vec![],
        }
    }
}

impl InfinityFirstReturnMap for OddCubic
{
    degree_impl!(3);

    #[inline]
    fn angle_map_large_param(&self, angle: RationalAngle) -> RationalAngle
    {
        angle * Rational::new(3, 2) + RationalAngle::ONE_HALF
    }
}

impl EscapeEncoding for OddCubic {}
impl ExternalRays for OddCubic {}

#[allow(clippy::suspicious_operation_groupings)]
impl HasDynamicalCovers for OddCubic
{
    fn marked_cycle_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> (Cplx, Cplx);
        let bounds: Bounds;

        match period {
            1 => {
                param_map = |t| (ONE_THIRD * t * t - 0.5, TWO_THIRDS * t);
                bounds = Bounds {
                    min_x: -2.5,
                    max_x: 2.5,
                    min_y: -2.5,
                    max_y: 2.5,
                };
            }
            2 => {
                param_map = |t| (ONE_THIRD * t * t + 1., TWO_THIRDS * t);
                bounds = Bounds {
                    min_x: -2.5,
                    max_x: 2.5,
                    min_y: -2.5,
                    max_y: 2.5,
                };
            }
            _ => {
                param_map = |t| (t, ONE);
                bounds = self.point_grid.bounds.clone();
            }
        }
        CoveringMap::new(self, param_map).with_orig_bounds(bounds)
    }

    fn dynatomic_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> (Cplx, Cplx);
        let bounds: Bounds;

        match period {
            1 => {
                param_map = |t| (ONE_THIRD * t * t - 0.5, TWO_THIRDS * t);
                bounds = Bounds {
                    min_x: -2.5,
                    max_x: 2.5,
                    min_y: -2.5,
                    max_y: 2.5,
                };
            }
            2 => {
                param_map = |t| {
                    let t2 = t * t;
                    (0.75 * t2 + ONE_THIRD / t2, 1.5 * t - TWO_THIRDS / (t * t2))
                };
                bounds = Bounds {
                    min_x: -1.5,
                    max_x: 1.5,
                    min_y: -1.5,
                    max_y: 1.5,
                };
            }
            _ => {
                param_map = |t| (t, ONE);
                bounds = self.point_grid.bounds.clone();
            }
        }
        CoveringMap::new(self, param_map).with_orig_bounds(bounds)
    }
    fn misiurewicz_curve(self, preperiod: Period, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> (Cplx, Cplx);
        let bounds: Bounds;

        match (preperiod, period) {
            (1, 1) => {
                param_map = |t| {
                    let t2 = t * t;
                    (
                        0.75 * t2 + 0.5 + ONE_THIRD / t2,
                        1.5 * t - TWO_THIRDS / (t * t2),
                    )
                };
                bounds = Bounds {
                    min_x: -2.5,
                    max_x: 2.5,
                    min_y: -2.5,
                    max_y: 2.5,
                };
            }
            (1, 2) => {
                param_map = |t| {
                    let g2 = ONE_NINTH.into();
                    let g3 = ZERO;
                    let (mut x, mut y) = weierstrass_p(g2, g3, t, 0.01);

                    x *= 3.;
                    y *= 6.;

                    x = x.inv();
                    y *= x;

                    let y2 = y * y;
                    let x2 = x * x;
                    let x4 = x2 * x2;

                    let u0 = 3. / y;
                    let u2 = (x2 * x + 3. * y2).inv();
                    let u3 = x4 * u0 * u2 - x * u0;
                    let u4 = 3. / y2;
                    let u5 = x4 * x * u2 * u4 - x2 * u4;

                    let u3_2 = u3 * u3;
                    let u5_2 = u5 * u5;
                    let v = u3_2 * u3_2 / (u5 * u5_2) + 3. * u3_2 / u5_2;

                    (v.inv(), ONE)
                };
                bounds = Bounds {
                    min_x: -2.5,
                    max_x: 2.5,
                    min_y: -2.5,
                    max_y: 2.5,
                };
            }
            (_, _) => {
                param_map = |t| (t, ONE);
                bounds = self.point_grid.bounds.clone();
            }
        }
        CoveringMap::new(self, param_map).with_orig_bounds(bounds)
    }
}
