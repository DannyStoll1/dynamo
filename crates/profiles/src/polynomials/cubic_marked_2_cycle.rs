use crate::macros::{degree_impl, horner, horner_monic, profile_imports};
use dynamo_common::math_utils::weierstrass_p;
profile_imports!();

// Cubic polynomials with 2-cycle 0 <-> 1
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CubicMarked2Cycle
{
    point_grid: PointGrid,
    compute_mode: ComputeMode,
    max_iter: IterCount,
}

impl CubicMarked2Cycle
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -3.4,
        max_x: 0.4,
        min_y: -2.9,
        max_y: 2.9,
    };
}
impl Default for CubicMarked2Cycle
{
    fractal_impl!();
}

impl DynamicalFamily for CubicMarked2Cycle
{
    parameter_plane_impl!();
    default_name!();

    #[inline]
    fn map(&self, z: Cplx, c: &Cplx) -> Cplx
    {
        let z2 = z.powi(2);
        (z + c) * z2 - (2. + c) * z + 1.
    }

    fn map_and_multiplier(&self, z: Cplx, c: &Cplx) -> (Cplx, Cplx)
    {
        let x0 = c + 2.;
        let z2 = z.powi(2);
        let x1 = z + c;
        (-x0 * z + x1 * z2 + 1., z2 - x0 + 2. * x1 * z)
    }

    fn gradient(&self, z: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        let x0 = c + 2.;
        let x1 = z + c;
        let z2 = z.powi(2);
        (-x0 * z + x1 * z2 + 1., z2 - x0 + 2. * x1 * z, z2 - z)
    }

    #[inline]
    fn start_point(&self, _point: Cplx, c: &Cplx) -> Cplx
    {
        let x0 = c * ONE_THIRD;
        let disc = (c * (c + 3.) + 6.).sqrt() * ONE_THIRD;
        disc * (c.re + 1.5).signum() - x0
    }

    #[inline]
    fn start_point_d(&self, _point: Cplx, c: &Self::Param)
        -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        let x0 = c * ONE_THIRD;
        let disc = (c * (c + 3.) + 6.).sqrt() * ONE_THIRD;
        let s = (c.re + 1.5).signum();
        (
            disc * s - x0,
            ZERO,
            (2. * c + 3.) * s / (18. * disc) - ONE_THIRD,
        )
    }
}

impl FamilyDefaults for CubicMarked2Cycle
{
    default_bounds!();
}

impl HasJulia for CubicMarked2Cycle
{
    #[inline]
    fn default_bounds_child(&self, _point: Cplx, param: &Cplx) -> Bounds
    {
        Bounds::square(2.2, -param / 3.)
    }
}

impl MarkedPoints for CubicMarked2Cycle
{
    #[inline]
    fn critical_points_child(&self, c: &Cplx) -> ComplexVec
    {
        let x0 = c * ONE_THIRD;
        let disc = (c * (c + 3.) + 6.).sqrt() * ONE_THIRD;
        vec![(disc - x0), (-disc - x0)]
    }

    fn cycles_child(&self, c: &Self::Param, period: Period) -> Vec<Self::Var>
    {
        match period {
            1 => solve_cubic(ONE, -c - 3., *c).to_vec(),
            2 => {
                let a0 = 3. + c * (3. + c);
                let a1 = -c * (c + 2.);
                let a2 = c * c - 2.;
                let a3 = c + c + 1.;
                let [r2, r3, r4, r5] = solve_quartic(a0, a1, a2, a3);
                vec![ZERO, ONE, r2, r3, r4, r5]
            }
            3 => {
                let coeffs = [
                    ONE,
                    horner!(c, 6., 9., 5., 1.),
                    horner!(c, -6., -10., 3., 11., 6., 1.),
                    horner!(c, -13., -73., -161., -168., -88., -22., -2.),
                    horner!(c, 123., 581., 1139., 1186., 693., 222., 35., 2.),
                    horner!(c, -597., -2551., -4957., -5428., -3528., -1348., -285., -29., -1.),
                    horner!(c, 808., 4620., 10901., 13980., 10550., 4732., 1212., 159., 8.),
                    horner!(c, 867., -337., -8526., -18155., -18241., -10118., -3129., -497., -31.),
                    horner!(c, -2922., -10380., -10428., 3693., 15574., 13169., 5268., 1025., 77.),
                    horner!(c, 953., 11891., 27819., 23868., 2830., -8462., -5746., -1476., -136.),
                    horner!(c, 3504., 3929., -15220., -33218., -22551., -2706., 3415., 1488., 179.),
                    horner!(c, -3222., -16406., -16862., 8724., 22515., 11079., 354., -975., -179.),
                    horner!(c, -1682., 6821., 26371., 20790., -3575., -9671., -2761., 267., 136.),
                    horner!(c, 3342., 9002., -4557., -21605., -11905., 2062., 2541., 203., -77.),
                    horner!(c, -180., -8762., -13864., 1695., 10510., 3184., -951., -293., 31.),
                    horner!(c, -1852., -1304., 9117., 9551., -1230., -3011., -189., 174., -8.),
                    horner!(c, 642., 4512., 2169., -5253., -3135., 763., 378., -56., 1.),
                    horner!(c, 594., -894., -4035., -939., 1785., 350., -168., 8.),
                    horner!(c, -353., -1206., 603., 1673., 0., -280., 28.),
                    horner!(c, -105., 499., 777., -294., -280., 56.),
                    horner!(c, 99., 161., -266., -168., 70.),
                    horner!(c, 8., -102., -56., 56.),
                    horner!(c, -15., -8., 28.),
                    8. * c,
                    ONE,
                ];
                solve_polynomial(coeffs)
            }
            _ => vec![],
        }
    }
}

degree_impl!(CubicMarked2Cycle, 3);

const MIS_1_1_G2: Cplx = Cplx::new(1. / 192., 0.);
const MIS_1_1_G3: Cplx = Cplx::new(-161. / 13824., 0.);
const FRAC_5_12: f64 = 5. / 12.;

impl HasDynamicalCovers for CubicMarked2Cycle
{
    fn marked_cycle_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> (Cplx, Cplx);
        let bounds: Bounds;

        match period {
            1 => {
                param_map = |t| {
                    let v = t * (t - 1.);
                    let u = (v + 1.) / v;
                    ((t * t * (t - 3.) + 1.) / v, u * u)
                };
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
        };
        CoveringMap::new(self, param_map).with_orig_bounds(bounds)
    }

    fn dynatomic_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> (Cplx, Cplx);
        let bounds: Bounds;

        match period {
            1 => {
                param_map = |t| {
                    let v = t * (t - 1.);
                    let u = (v + 1.) / v;
                    ((t * t * (t - 3.) + 1.) / v, u * u)
                };
                bounds = Bounds {
                    min_x: -2.5,
                    max_x: 2.5,
                    min_y: -2.5,
                    max_y: 2.5,
                };
            }
            2 => {
                param_map = |t| {
                    let l = t + 0.5;
                    let v = horner_monic!(l, OMEGA - 1., OMEGA + 1.);
                    let dv = horner!(l, OMEGA - 1., 2.);
                    let u = -(OMEGA + l).inv();
                    let du = -u * u;
                    (v * u, v * du + u * dv)
                };
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
        };
        CoveringMap::new(self, param_map).with_orig_bounds(bounds)
    }
    fn misiurewicz_curve(self, preperiod: Period, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> (Cplx, Cplx);
        let bounds: Bounds;

        match (preperiod, period) {
            (1, 1) => {
                param_map = |t| {
                    let (mut x, mut y) = weierstrass_p(MIS_1_1_G2, MIS_1_1_G3, t + 0.123, 0.01);

                    x *= 4.;
                    y *= 4.;

                    x = FRAC_5_12 - x;
                    y += (x - 1.) / 2.;

                    y /= x;

                    let z = x - 1.;
                    y = z / (y - 1.);
                    x /= z;

                    y = horner!(x, -1., -1., 1., y);

                    (y / x, ONE)
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
                    let l = t + 0.5;
                    let numer = horner_monic!(l, OMEGA + 1., 1. - 3. * OMEGA, -3., OMEGA);
                    let denom = l * (1. - l) * (OMEGA + l);
                    (numer / denom, ONE) //TODO
                };
                bounds = Bounds {
                    min_x: -3.0,
                    max_x: 3.0,
                    min_y: -3.0,
                    max_y: 3.0,
                };
            }
            (_, _) => {
                param_map = |t| (t, ONE);
                bounds = self.point_grid.bounds.clone();
            }
        };
        CoveringMap::new(self, param_map).with_orig_bounds(bounds)
    }
}
