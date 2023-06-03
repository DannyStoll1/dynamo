use crate::macros::*;
use crate::math_utils::{slog, solve_cubic, solve_quartic, weierstrass_p};
profile_imports!();

pub mod mandelbrot;
pub use mandelbrot::Mandelbrot;

pub mod quad_rat_per_2;
pub use quad_rat_per_2::QuadRatPer2;
pub mod quad_rat_per_3;
pub use quad_rat_per_3::QuadRatPer3;
pub mod quad_rat_per_4;
pub use quad_rat_per_4::QuadRatPer4;
pub mod quad_rat_per_5;
pub use quad_rat_per_5::QuadRatPer5;
pub mod quad_rat_preper_2_1;
pub use quad_rat_preper_2_1::QuadRatPreper21;

pub mod cubic_per_1_lambda;
pub use cubic_per_1_lambda::{CubicPer1Lambda, CubicPer1LambdaParam, CubicPer1_1};
pub mod cubic_per_2_lambda;
pub use cubic_per_2_lambda::{CubicPer2Lambda, CubicPer2LambdaParam};
pub mod cubic_per_3_0;
pub use cubic_per_3_0::CubicPer3_0;

pub mod biquadratic;
pub use biquadratic::{Biquadratic, BiquadraticMult, BiquadraticMultParam};

pub mod zeta;
pub use zeta::RiemannXi;

pub mod rulkov;
pub use rulkov::Rulkov;

#[derive(Clone, Debug)]
pub struct QuadRatSymmetryLocus
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl QuadRatSymmetryLocus
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -1.1,
        max_x: 1.1,
        min_y: -1.1,
        max_y: 1.1,
    };
}
impl Default for QuadRatSymmetryLocus
{
    fractal_impl!();
}

impl ParameterPlane for QuadRatSymmetryLocus
{
    parameter_plane_impl!();
    default_name!();

    fn encode_escaping_point(
        &self,
        iters: Period,
        z: ComplexNum,
        base_param: ComplexNum,
    ) -> PointInfo<Self::Deriv>
    {
        if z.is_nan()
        {
            return PointInfo::Escaping {
                potential: f64::from(iters) - 2.,
            };
        }

        let expansion_rate = base_param.norm_sqr();
        let u = self.escape_radius().log(expansion_rate);
        let v = z.norm_sqr().log(expansion_rate);
        let residual = u - v;
        let potential = IterCount::from(iters) + (residual as IterCount);
        PointInfo::Escaping { potential }
    }

    #[inline]
    fn map(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        c * (z + 1. / z)
    }

    #[inline]
    fn map_and_multiplier(&self, z: ComplexNum, c: ComplexNum) -> (ComplexNum, ComplexNum)
    {
        let u = z.inv();
        (c * (z + u), c * (1. - u * u))
    }

    fn start_point(&self, _point: ComplexNum, _c: Self::Param) -> Self::Var
    {
        (1.).into()
    }

    #[inline]
    fn dynamical_derivative(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        c * (1. - 1. / (z * z))
    }

    #[inline]
    fn parameter_derivative(&self, z: ComplexNum, _c: ComplexNum) -> ComplexNum
    {
        z + 1. / z
    }

    #[inline]
    fn critical_points_child(&self, _param: ComplexNum) -> ComplexVec
    {
        vec![(-1.).into(), (1.).into()]
    }

    #[inline]
    fn default_julia_bounds(&self, _point: ComplexNum, _param: ComplexNum) -> Bounds
    {
        Bounds::centered_square(4.)
    }
}

#[derive(Clone, Debug)]
pub struct OddCubic
{
    point_grid: PointGrid,
    max_iter: Period,
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

impl ParameterPlane for OddCubic
{
    parameter_plane_impl!();
    default_name!();

    fn encode_escaping_point(
        &self,
        iters: Period,
        z: ComplexNum,
        _base_param: ComplexNum,
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
    fn map(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        2. * (z * z * z / 3. - c * z)
    }

    #[inline]
    fn start_point(&self, _point: ComplexNum, param: ComplexNum) -> ComplexNum
    {
        param.powf(0.5)
    }

    #[inline]
    fn dynamical_derivative(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        2. * (z * z - c)
    }

    #[inline]
    fn parameter_derivative(&self, z: ComplexNum, _c: ComplexNum) -> ComplexNum
    {
        -(z + z)
    }

    #[inline]
    fn critical_points_child(&self, param: ComplexNum) -> ComplexVec
    {
        let sqrt_c = param.sqrt();
        vec![-sqrt_c, sqrt_c]
    }

    #[inline]
    fn default_julia_bounds(&self, _point: ComplexNum, _param: ComplexNum) -> Bounds
    {
        Bounds::centered_square(2.2)
    }
}

// Cubic polynomials with critical 2-cycle 0 <-> c
#[derive(Clone, Debug)]
pub struct CubicPer2CritMarked
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl CubicPer2CritMarked
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -2.6,
        max_x: 2.6,
        min_y: -1.9,
        max_y: 1.9,
    };
}
impl Default for CubicPer2CritMarked
{
    fractal_impl!();
}

impl ParameterPlane for CubicPer2CritMarked
{
    parameter_plane_impl!();
    default_name!();

    fn encode_escaping_point(
        &self,
        iters: Period,
        z: ComplexNum,
        _base_param: ComplexNum,
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
    fn map(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        z * z * (z - c - 1. / c) + c
    }

    #[inline]
    fn start_point(&self, _point: ComplexNum, param: ComplexNum) -> ComplexNum
    {
        2. / 3. * (param + 1. / param)
    }

    #[inline]
    fn dynamical_derivative(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        // let u = z * (3. * z - c - c - 2. / c) * (z / c).re.signum();
        // u / u.norm()
        z * (3. * z - c - c - 2. / c)
    }

    #[inline]
    fn parameter_derivative(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        let z2 = z * z;
        let c2 = c * c;
        1. + z2 / c2 + -z2
    }

    #[inline]
    fn critical_points_child(&self, c: Self::Param) -> Vec<Self::Var>
    {
        let u = c + c.inv();
        vec![(0.).into(), TWO_THIRDS * u]
    }

    fn cycles_child(&self, c: Self::Param, period: Period) -> Vec<Self::Var>
    {
        match period
        {
            1 => solve_cubic(c, -ONE, -c - c.inv()).to_vec(),
            2 =>
            {
                let cinv = c.inv();
                let cinv2 = cinv * cinv;
                let [r2, r3, r4, r5] = solve_quartic(cinv2, c - cinv, cinv2 + 1., -c - cinv - cinv);
                vec![ZERO, c, r2, r3, r4, r5]
            }
            _ => vec![],
        }
    }

    #[inline]
    fn default_julia_bounds(&self, _point: ComplexNum, param: ComplexNum) -> Bounds
    {
        Bounds::square(2.2, param / 2.)
    }
}

// Cubic polynomials with 2-cycle 0 <-> 1
#[derive(Clone, Debug)]
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
        z: ComplexNum,
        _base_param: ComplexNum,
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
    fn map(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        let z2 = z * z;
        (z + c) * z2 - (2. + c) * z + 1.
    }

    fn map_and_multiplier(&self, z: ComplexNum, c: ComplexNum) -> (ComplexNum, ComplexNum)
    {
        let x0 = c + 2.;
        let z2 = z * z;
        let x1 = z + c;
        (-x0 * z + x1 * z2 + 1., -x0 + z2 + x1 * (z + z))
    }

    #[inline]
    fn start_point(&self, _point: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        let x0 = c * ONE_THIRD;
        let disc = (c * (c + 3.) + 6.).sqrt() * ONE_THIRD;
        -disc - x0
    }

    #[inline]
    fn critical_points_child(&self, c: ComplexNum) -> ComplexVec
    {
        let x0 = c * ONE_THIRD;
        let disc = (c * (c + 3.) + 6.).sqrt() * ONE_THIRD;
        vec![(disc - x0), (-disc - x0)]
    }

    #[inline]
    fn dynamical_derivative(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        let z2 = z * z;
        let x1 = z + c;
        -c + z2 + (x1 + x1) * z - 2.
    }

    #[inline]
    fn parameter_derivative(&self, z: ComplexNum, _c: ComplexNum) -> ComplexNum
    {
        z * (z - 1.)
    }

    #[inline]
    fn default_julia_bounds(&self, _point: ComplexNum, param: ComplexNum) -> Bounds
    {
        Bounds::square(2.2, -param / 3.)
    }
}

#[derive(Clone, Debug)]
pub struct BurningShip
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl BurningShip
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -2.2,
        max_x: 1.25,
        min_y: -1.9,
        max_y: 0.6,
    };
    const JULIA_BOUNDS: Bounds = Bounds::centered_square(4.);
}
impl Default for BurningShip
{
    fractal_impl!();
}

impl ParameterPlane for BurningShip
{
    parameter_plane_impl!();
    default_name!();

    fn encode_escaping_point(
        &self,
        iters: Period,
        z: ComplexNum,
        _base_param: ComplexNum,
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
        let residual = (v / u).log2();
        let potential = f64::from(iters) - (residual as IterCount);
        PointInfo::Escaping { potential }
    }

    #[inline]
    fn map(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        let z = ComplexNum::new(z.re.abs(), z.im.abs());
        z * z + c
    }

    #[inline]
    fn dynamical_derivative(&self, _z: ComplexNum, _c: ComplexNum) -> ComplexNum
    {
        ONE //TODO
    }

    #[inline]
    fn parameter_derivative(&self, _z: ComplexNum, _c: ComplexNum) -> ComplexNum
    {
        ONE //TODO
    }

    #[inline]
    fn gradient(&self, _z: ComplexNum, _c: ComplexNum) -> (ComplexNum, ComplexNum, ComplexNum)
    {
        (ONE, ONE, ONE) //TODO
    }

    #[inline]
    fn param_map(&self, c: ComplexNum) -> ComplexNum
    {
        c
    }
}

#[derive(Clone, Debug)]
pub struct Sailboat
{
    point_grid: PointGrid,
    max_iter: Period,
    shift: ComplexNum,
}

impl Sailboat
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -6.,
        max_x: 6.,
        min_y: -6.,
        max_y: 6.,
    };
    const JULIA_BOUNDS: Bounds = Bounds::centered_square(5.);
}
impl Default for Sailboat
{
    fractal_impl!(shift, ZERO);
}

impl ParameterPlane for Sailboat
{
    parameter_plane_impl!();

    fn encode_escaping_point(
        &self,
        iters: Period,
        z: ComplexNum,
        _base_param: ComplexNum,
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
        let residual = (v / u).log2();
        let potential = f64::from(iters) - (residual as IterCount);
        PointInfo::Escaping { potential }
    }

    #[inline]
    fn map(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        let z = ComplexNum::new(z.re.abs(), z.im.abs()) + self.shift;
        z * z + c
    }

    #[inline]
    fn dynamical_derivative(&self, _z: ComplexNum, _c: ComplexNum) -> ComplexNum
    {
        ONE //TODO
    }

    #[inline]
    fn parameter_derivative(&self, _z: ComplexNum, _c: ComplexNum) -> ComplexNum
    {
        ONE //TODO
    }

    #[inline]
    fn gradient(&self, _z: Self::Var, _c: Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        (ONE, ONE, ONE) //TODO
    }

    #[inline]
    fn param_map(&self, c: ComplexNum) -> ComplexNum
    {
        c
    }

    #[inline]
    fn name(&self) -> String
    {
        let shift = self.shift;
        format!("Sailboat({shift})")
    }
}

#[derive(Clone, Debug)]
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
    const JULIA_BOUNDS: Bounds = Bounds {
        min_x: -5.,
        max_x: 5.,
        min_y: -5.,
        max_y: 5.,
    };
}
impl Default for Exponential
{
    fractal_impl!();
}

impl ParameterPlane for Exponential
{
    parameter_plane_impl!();
    default_name!();

    fn encode_escaping_point(
        &self,
        iters: Period,
        z: ComplexNum,
        _base_param: ComplexNum,
    ) -> PointInfo<Self::Deriv>
    {
        if z.is_nan()
        {
            return PointInfo::Escaping {
                potential: f64::from(iters) - 1.,
            };
        }

        if z.re < 0.
        {
            return PointInfo::Periodic {
                period: 1,
                preperiod: iters,
                multiplier: (0.).into(),
                final_error: 1e-8,
            };
        }
        if z.is_infinite()
        {
            return PointInfo::Escaping {
                potential: f64::from(iters) + 1.,
            };
        }
        let u = slog(self.escape_radius());
        let v = slog(z.norm_sqr());
        let residual = v - u;
        let potential = f64::from(iters) - (residual as IterCount);
        PointInfo::Escaping { potential }
    }

    #[inline]
    fn map(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        z.exp() + c
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, c: Self::Param) -> (Self::Var, Self::Deriv)
    {
        let u = z.exp();
        (u, u + c)
    }

    #[inline]
    fn dynamical_derivative(&self, z: ComplexNum, _c: ComplexNum) -> ComplexNum
    {
        z.exp()
    }

    #[inline]
    fn parameter_derivative(&self, _z: ComplexNum, _c: ComplexNum) -> ComplexNum
    {
        ONE
    }

    #[inline]
    fn gradient(&self, z: ComplexNum, c: ComplexNum) -> (ComplexNum, ComplexNum, ComplexNum)
    {
        let u = z.exp();
        (u + c, u, ONE)
    }

    #[inline]
    fn param_map(&self, c: ComplexNum) -> ComplexNum
    {
        c
    }
}
