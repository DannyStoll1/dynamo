use crate::{
    macros::{horner, horner_monic, profile_imports},
    math_utils::{solve_cubic, solve_quartic, weierstrass_p},
};
profile_imports!();

// Cubic polynomials with 2-cycle 0 <-> 1
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CubicMarked2Cycle
{
    point_grid: PointGrid,
    max_iter: Period,
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

impl ParameterPlane for CubicMarked2Cycle
{
    parameter_plane_impl!();
    default_name!();

    fn encode_escaping_point(
        &self,
        iters: Period,
        z: Cplx,
        _base_param: Cplx,
    ) -> PointInfo<Self::Deriv>
    {
        if z.is_nan()
        {
            return PointInfo::Escaping {
                potential: f64::from(iters) - 1.,
            };
        }

        let u = self.escape_radius().log2();
        let v = z.norm_sqr().log2();
        let residual = (v / u).log(3.);
        let potential = f64::from(iters) - (residual as IterCount);
        PointInfo::Escaping { potential }
    }

    #[inline]
    fn map(&self, z: Cplx, c: Cplx) -> Cplx
    {
        let z2 = z * z;
        (z + c) * z2 - (2. + c) * z + 1.
    }

    fn map_and_multiplier(&self, z: Cplx, c: Cplx) -> (Cplx, Cplx)
    {
        let x0 = c + 2.;
        let z2 = z * z;
        let x1 = z + c;
        (-x0 * z + x1 * z2 + 1., -x0 + z2 + x1 * (z + z))
    }

    #[inline]
    fn start_point(&self, _point: Cplx, c: Cplx) -> Cplx
    {
        let x0 = c * ONE_THIRD;
        let disc = (c * (c + 3.) + 6.).sqrt() * ONE_THIRD;
        disc * (c.re + 1.5).signum() - x0
    }

    #[inline]
    fn critical_points_child(&self, c: Cplx) -> ComplexVec
    {
        let x0 = c * ONE_THIRD;
        let disc = (c * (c + 3.) + 6.).sqrt() * ONE_THIRD;
        vec![(disc - x0), (-disc - x0)]
    }

    #[inline]
    fn dynamical_derivative(&self, z: Cplx, c: Cplx) -> Cplx
    {
        let z2 = z * z;
        let x1 = z + c;
        -c + z2 + (x1 + x1) * z - 2.
    }

    #[inline]
    fn parameter_derivative(&self, z: Cplx, _c: Cplx) -> Cplx
    {
        z * (z - 1.)
    }

    #[inline]
    fn default_julia_bounds(&self, _point: Cplx, param: Cplx) -> Bounds
    {
        Bounds::square(2.2, -param / 3.)
    }

    #[inline]
    fn cycles_child(&self, c: Self::Param, period: Period) -> Vec<Self::Var>
    {
        match period
        {
            1 => solve_cubic(ONE, -c - 3., c).to_vec(),
            2 =>
            {
                let a0 = 3. + c * (3. + c);
                let a1 = -c * (c + 2.);
                let a2 = c * c - 2.;
                let a3 = c + c + 1.;
                let [r2, r3, r4, r5] = solve_quartic(a0, a1, a2, a3);
                vec![ZERO, ONE, r2, r3, r4, r5]
            }
            _ => vec![],
        }
    }
}

const MIS_1_1_G2: Cplx = Cplx::new(1. / 192., 0.);
const MIS_1_1_G3: Cplx = Cplx::new(-161. / 13824., 0.);
const FRAC_5_12: f64 = 5. / 12.;
const FRAC_7_48: f64 = 7. / 48.;

impl HasDynamicalCovers for CubicMarked2Cycle
{
    fn marked_cycle_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> Cplx;
        let bounds: Bounds;

        match period
        {
            1 =>
            {
                param_map = |t| (t * t * (t - 3.) + 1.) / (t * (t - 1.));
                bounds = Bounds {
                    min_x: -2.5,
                    max_x: 2.5,
                    min_y: -2.5,
                    max_y: 2.5,
                };
            }
            _ =>
            {
                param_map = |c| c;
                bounds = self.point_grid.bounds.clone();
            }
        };
        let grid = self.point_grid.new_with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }

    fn dynatomic_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> Cplx;
        let bounds: Bounds;

        match period
        {
            1 =>
            {
                param_map = |t| (t * t * (t - 3.) + 1.) / (t * (t - 1.));
                bounds = Bounds {
                    min_x: -2.5,
                    max_x: 2.5,
                    min_y: -2.5,
                    max_y: 2.5,
                };
            }
            2 =>
            {
                param_map = |t| {
                    let l = t + 0.5;
                    let numer = horner_monic!(l, OMEGA - 1., OMEGA + 1.);
                    -numer / (OMEGA + l)
                };
                bounds = Bounds {
                    min_x: -2.5,
                    max_x: 2.5,
                    min_y: -2.5,
                    max_y: 2.5,
                };
            }
            _ =>
            {
                param_map = |c| c;
                bounds = self.point_grid.bounds.clone();
            }
        };
        let grid = self.point_grid.new_with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }
    fn misiurewicz_curve(self, preperiod: Period, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> Cplx;
        let bounds: Bounds;

        match (preperiod, period)
        {
            (1, 1) =>
            {
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

                    y / x
                };
                bounds = Bounds {
                    min_x: -2.5,
                    max_x: 2.5,
                    min_y: -2.5,
                    max_y: 2.5,
                };
            }
            (1, 2) =>
            {
                param_map = |t| {
                    let l = t + 0.5;
                    let numer = horner_monic!(l, OMEGA + 1., 1. - 3. * OMEGA, -3., OMEGA);
                    let denom = l * (1. - l) * (OMEGA + l);
                    numer / denom
                };
                bounds = Bounds {
                    min_x: -3.0,
                    max_x: 3.0,
                    min_y: -3.0,
                    max_y: 3.0,
                };
            }
            (_, _) =>
            {
                param_map = |t| t;
                bounds = self.point_grid.bounds.clone();
            }
        };
        let grid = self.point_grid.new_with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }
}
