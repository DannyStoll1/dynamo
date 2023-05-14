use crate::dynamics::{
    covering_maps::{CoveringMap, HasDynamicalCovers},
    ParameterPlane,
};
use crate::math_utils::{slog, weierstrass_p};
use crate::point_grid::{Bounds, PointGrid};
use crate::types::*;

use crate::macros::{default_name, fractal_impl, parameter_plane_impl};

pub mod mandelbrot;
pub use mandelbrot::Mandelbrot;

pub mod quad_rat_per_2;
pub use quad_rat_per_2::QuadRatPer2;

pub mod rulkov;
pub use rulkov::Rulkov;

use std::any::type_name;

// #[derive(Clone, Debug)]
// pub struct Mandelbrot
// {
//     point_grid: PointGrid,
//     max_iter: Period,
// }
//
// impl Mandelbrot
// {
//     const DEFAULT_BOUNDS: Bounds = Bounds {
//         min_x: -2.2,
//         max_x: 0.65,
//         min_y: -1.4,
//         max_y: 1.4,
//     };
//     fractal_impl!();
// }
//
// impl ParameterPlane for Mandelbrot
// {
//     parameter_plane_impl!();
//     default_name!();
//
//     fn encode_escaping_point(
//         &self,
//         iters: Period,
//         z: ComplexNum,
//         _base_param: ComplexNum,
//     ) -> PointInfo
//     {
//         if z.is_nan()
//         {
//             return PointInfo::Escaping {
//                 potential: f64::from(iters) - 1.,
//             };
//         }
//
//         let u = self.escape_radius().log2();
//         let v = z.norm_sqr().log2();
//         let residual = (v / u).log2();
//         let potential = f64::from(iters) - (residual as IterCount);
//         PointInfo::Escaping { potential }
//     }
//
//     #[inline]
//     fn map(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
//     {
//         z * z + c
//     }
//
//     #[inline]
//     fn dynamical_derivative(&self, z: ComplexNum, _c: ComplexNum) -> ComplexNum
//     {
//         z + z
//     }
//
//     #[inline]
//     fn parameter_derivative(&self, _z: ComplexNum, _c: ComplexNum) -> ComplexNum
//     {
//         ONE_COMPLEX
//     }
//
//     #[inline]
//     fn gradient(&self, z: ComplexNum, _c: ComplexNum) -> (ComplexNum, ComplexNum)
//     {
//         (z + z, ONE_COMPLEX)
//     }
//
//     fn early_bailout(&self, _start: ComplexNum, param: ComplexNum) -> EscapeState
//     {
//         // Main cardioid
//         let four_c = 4. * param;
//         let y2 = four_c.im * four_c.im;
//         let temp = four_c.re - 1.;
//         let mu_norm2 = temp * temp + y2;
//         let a = mu_norm2 * (mu_norm2 * 0.25 + temp);
//
//         if a < y2
//         {
//             let multiplier = 1. - (1. - four_c).sqrt();
//             let decay_rate = multiplier.norm();
//             let fixed_point = 0.5 * multiplier;
//             let init_dist = (param - fixed_point).norm_sqr();
//             let potential = init_dist.log(decay_rate);
//             let preperiod = potential as Period;
//             return EscapeState::Periodic {
//                 period: 1,
//                 preperiod,
//                 multiplier,
//                 final_error: (1e-6).into(),
//             };
//         }
//
//         // Basilica bulb
//         let mu2 = four_c + 4.;
//         if mu2.norm_sqr() < 1.
//         {
//             let decay_rate = mu2.norm();
//             let fixed_point = -0.5 - 0.5 * (-four_c - 3.).sqrt();
//             let init_dist = (param - fixed_point).norm_sqr();
//             let potential = 2. * init_dist.log(decay_rate);
//             let preperiod = potential as Period;
//             return EscapeState::Periodic {
//                 period: 2,
//                 preperiod,
//                 multiplier: mu2,
//                 final_error: (1e-6).into(),
//             };
//         }
//
//         EscapeState::NotYetEscaped
//     }
//
//     #[inline]
//     fn default_julia_bounds(&self, _param: ComplexNum) -> Bounds
//     {
//         Bounds::centered_square(2.2)
//     }
// }

#[derive(Clone, Debug)]
pub struct QuadRatPer3
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl QuadRatPer3
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -2.5,
        max_x: 3.2,
        min_y: -2.5,
        max_y: 2.5,
    };
    fractal_impl!();
}

impl ParameterPlane for QuadRatPer3
{
    parameter_plane_impl!();
    default_name!();

    fn start_point(&self, _point: ComplexNum, _c: ComplexNum) -> ComplexNum
    {
        0.0.into()
    }

    fn encode_escaping_point(
        &self,
        iters: Period,
        z: ComplexNum,
        base_param: ComplexNum,
    ) -> PointInfo<Self::Var, Self::Deriv>
    {
        if z.is_nan()
        {
            return PointInfo::Escaping {
                potential: f64::from(iters) - 3.,
            };
        }

        let u = self.escape_radius().log2();
        let v = z.norm_sqr().log2();
        let q = ((base_param - 1.) / (4. * base_param)).norm().log2();
        let residual = ((u + q) / (v + q)).log2();
        let potential = (residual as IterCount).mul_add(3., f64::from(iters));
        PointInfo::Escaping { potential }
    }

    #[inline]
    fn map_and_multiplier(&self, z: ComplexNum, c: ComplexNum) -> (ComplexNum, ComplexNum)
    {
        let z2 = z * z;
        let c2 = c * c;
        let u = (z2 - c2).inv();
        let v = c + 1.;
        ((z2 + c2 * c - v) * u, TWO * (1. - c) * v * v * z * u * u)
    }

    #[inline]
    fn map(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        (z * z + c * c * c - c - 1.) / (z * z - c * c)
    }

    #[inline]
    fn dynamical_derivative(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        let u = 1. / (c * c - z * z);
        let v = c + 1.;
        TWO * (1. - c) * v * v * z * u * u
    }

    #[inline]
    fn parameter_derivative(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        let r = c * c - z * z;
        let u2 = 1. / (r * r);
        (c + 1.) * u2 * (r - c * (r + TWO * (ONE_COMPLEX - z * z)))
    }

    #[inline]
    fn gradient(&self, z: Self::Var, c: Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        let z2 = z * z;
        let c2 = c * c;
        let r = c2 - z2;
        let u = r.inv();
        let u2 = u * u;
        let v = c + 1.;

        let f = u * (z2 + c2 * c - v);
        let df_dz = TWO * (1. - c) * v * v * z * u2;
        let df_dc = v * u2 * (r - c * (r + TWO * (ONE_COMPLEX - z * z)));
        (f, df_dz, df_dc)
    }

    #[inline]
    fn critical_points(&self, _param: ComplexNum) -> ComplexVec
    {
        vec![(0.).into()]
    }

    fn cycles(&self, c: ComplexNum, period: Period) -> ComplexVec
    {
        match period
        {
            1 =>
            {
                let x0 = c * c;
                let x1 = c * x0;
                let x2 = 3. * x0 + 1.;
                let u = 27. * (c - x1) - 9. * x0 + 25.;
                let x3 = (0.5 * (u + (-4. * x2 * x2 * x2 + u * u).sqrt())).powf(ONE_THIRD);
                let x4 = x3 / 3.;
                let x5 = x2 / (3. * x3);
                let r1 = -x4 * OMEGA_BAR - x5 * OMEGA + ONE_THIRD;
                let r2 = -x4 * OMEGA - x5 * OMEGA_BAR + ONE_THIRD;
                vec![-x4 - x5 + ONE_THIRD, r1, r2]
            }
            2 =>
            {
                let disc = (c * (5. * c + 6.) + 5.).sqrt();
                vec![-0.5 * (c - disc + 1.), -0.5 * (c + disc + 1.)]
            }
            _ => vec![],
        }
    }

    #[inline]
    fn default_julia_bounds(&self, _param: ComplexNum) -> Bounds
    {
        Bounds::centered_square(4.)
    }
}
impl HasDynamicalCovers for QuadRatPer3
{
    fn marked_cycle_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(ComplexNum) -> ComplexNum;
        let bounds: Bounds;

        match period
        {
            1 =>
            {
                param_map = |c| {
                    let pole = 1.324_717_957_244_75;
                    let c = 1. / c + pole;
                    let c2 = c * c;
                    let c3 = c2 * c;
                    (c3 - c + 1.) / (c3 - c2 - c2 + c + c + c - 1.)
                };
                bounds = Bounds {
                    min_x: -5.75,
                    max_x: 5.08,
                    min_y: -5.32,
                    max_y: 5.32,
                };
            }
            4 =>
            {
                param_map = |c| {
                    let t = (13.0 as RealNum).sqrt();
                    let g2 = ComplexNum::new(-8.0 / 3.0, 0.);
                    let g3 = ComplexNum::new(1.0 / 27.0, 0.);

                    let (p, dp) = weierstrass_p(g2, g3, c, 0.01);
                    let x = p - 1. / 3.;
                    let y = (dp + 1.) / x - t - 1.;

                    let u = x / 2.;
                    let v = y / 4.;
                    let xx = -(t + 1.) * u + (t + 3.) * v + (t + 4.);
                    let yy = u - v - (t + 1.) / 4.;
                    let zz = -x + 2. * v + (t + 3.) / 2.;

                    let s0 = xx / zz;
                    let s1 = zz / yy;

                    s0 * s1 + s1 + (t + 4.)
                    // let l = s0^2*s1 + s0*s1 + (2*t)*s0 + (t - 1);
                };
                bounds = Bounds {
                    min_x: -3.9,
                    max_x: 3.9,
                    min_y: -2.6,
                    max_y: 2.6,
                };
            }
            _ =>
            {
                param_map = |c| c;
                bounds = self.point_grid.bounds.clone();
            }
        };
        let grid = self.point_grid.with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }
}

#[derive(Clone, Debug)]
pub struct QuadRatPer4
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl QuadRatPer4
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -1.,
        max_x: 0.2,
        min_y: -0.5,
        max_y: 0.5,
    };
    fractal_impl!();
}

impl ParameterPlane for QuadRatPer4
{
    parameter_plane_impl!();
    default_name!();

    fn encode_escaping_point(
        &self,
        iters: Period,
        z: ComplexNum,
        c: ComplexNum,
    ) -> PointInfo<Self::Var, Self::Deriv>
    {
        {
            if z.is_nan()
            {
                return PointInfo::Escaping {
                    potential: f64::from(iters) - 4.,
                };
            }

            let u = self.escape_radius().log2();
            let v = z.norm_sqr().log2();
            let c2 = c * c;
            let two_c = c + c;
            let c12 = c2 - two_c + 1.; // (c-1)^2

            let d0 = c2 + c - 1.; // c^2 + c - 1
            let d1 = d0 - two_c - two_c + 2.; // c^2 - 3c + 1
            let d2 = c2 + c2 + d1; // 3c^2 - 3c + 1

            // (2*a - 1) * (a - 1)^5 * a^5 * (a^2 - 3*a + 1)^-2 * (3a^2 - 3a + 1)^-2 * (a^2 + a - 1)^-2
            let q_numer = (two_c - 1.) * c2 * c2 * c * c12 * c12 * (c - 1.);
            let q_denom = d0 * d0 + d1 * d1 + d2 * d2;
            let q = (q_numer / q_denom).norm().log2();
            let residual = ((u + q) / (v + q)).log2();
            let potential = (residual as IterCount).mul_add(4., f64::from(iters));
            PointInfo::Escaping { potential }
        }
    }

    #[inline]
    fn param_map(&self, c: ComplexNum) -> ComplexNum
    {
        let pole = 2.618_033_988_749_89;
        1. / c + pole
    }

    #[inline]
    fn start_point(&self, _point: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        (c + c) * (c + c - 1.) / (c * (c + 1.) - 1.)
    }

    #[inline]
    fn map(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        (c * z - c - c - z + 1.) * (z - c) / (z * z * (c - 1.))
    }

    #[inline]
    fn map_and_multiplier(&self, z: ComplexNum, c: ComplexNum) -> (ComplexNum, ComplexNum)
    {
        let c2 = c * c;
        let c_minus_1 = c - 1.;
        let u = (c_minus_1 * z * z).inv();
        let two_c = c + c;

        (
            (z - c) * (c * z - z - two_c + 1.) * u,
            ((4. * c2 - two_c) / z - (c2 + c_minus_1)) * u,
        )
    }

    #[inline]
    fn dynamical_derivative(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        let c2 = c * c;
        (4. * c2 - (c2 + c - 1.) * z - c - c) / ((c - 1.) * z * z * z)
    }

    #[inline]
    fn parameter_derivative(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        let v = c - 1.;
        (2. - c) * c * (z - 2.) / (v * v * z * z)
    }

    #[inline]
    fn gradient(&self, z: Self::Var, c: Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        let v = c - 1.;
        let c2 = c * c;
        let z2 = z * z;
        let u = (v * z2).inv();
        let two_c = c + c;
        (
            (z - c) * (c * z - z - two_c + 1.) * u,
            (4. * c2 - (c2 + v) * z - two_c) * u / z,
            (two_c - c2) * (z - 2.) * u / v,
        )
    }

    fn cycles(&self, c: ComplexNum, period: Period) -> ComplexVec
    {
        match period
        {
            1 =>
            {
                let x0 = c - 1.;
                let x1 = x0.inv();
                let x2 = c * c;
                let x3 = x1 * (x0 + x2);
                let x4 = x1 * (c - (x2 + x2));
                let x5 = 1. - 3. * x3;
                let s = -4. * x5.powf(3.);
                let t = 9. * x3 + 27. * x4 - 2.;
                let u = (s + t * t).sqrt();
                let x6 = (0.5 * (t + u)).powf(ONE_THIRD);
                let x7 = x6 / 3.;
                let x8 = x5 / (3. * x6);
                let r1 = -x7 * OMEGA_BAR - x8 * OMEGA + ONE_THIRD;
                let r2 = -x7 * OMEGA - x8 * OMEGA_BAR + ONE_THIRD;
                vec![-x7 - x8 + ONE_THIRD, r1, r2]
            }
            2 =>
            {
                let c2 = c * c;
                let x0 = c2 * 3.;
                let denom = 0.5 / (c - 1.);
                let disc = (x0 * x0 - c * (8. * c2 - 6. * c + 4.) + 1.).sqrt();
                vec![denom * (x0 + disc - 1.), denom * (x0 - disc - 1.)]
            }
            _ => vec![],
        }
    }

    #[inline]
    fn default_julia_bounds(&self, _param: ComplexNum) -> Bounds
    {
        Bounds::square(4., (2.).into())
    }
}

impl HasDynamicalCovers for QuadRatPer4
{
    fn marked_cycle_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(ComplexNum) -> ComplexNum;
        let grid: PointGrid;
        let bounds: Bounds;

        match period
        {
            3 =>
            {
                param_map = |c| {
                    // cbrt(12)
                    let alpha = ComplexNum::new(2.289_428_485_106_66, 0.);
                    let g2 = alpha;
                    let g3 = ComplexNum::new(-19. / 12., 0.);

                    let (p, _dp) = weierstrass_p(g2, g3, c, 0.01);
                    let x = (alpha * p + 1.) / 3.;
                    // let y = (dp - 1.5) / x;

                    x / (x + 1.)
                    // let xx = x + 1.;
                    // let yy = y - 3. * x - 3.;
                    //
                    // let x0 = yy / x;
                    // let _s1 = x0 * xx / x;

                    // x / xx
                };
                bounds = Bounds {
                    min_x: -3.6,
                    max_x: 3.6,
                    min_y: -2.4,
                    max_y: 2.4,
                };
                grid = self.point_grid.with_same_height(bounds);
            }
            _ =>
            {
                param_map = |c| c;
                grid = self.point_grid.clone();
            }
        };
        CoveringMap::new(self, param_map, grid)
    }
}

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
    ) -> PointInfo<Self::Var, Self::Deriv>
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
    fn critical_points(&self, _param: ComplexNum) -> ComplexVec
    {
        vec![(-1.).into(), (1.).into()]
    }

    #[inline]
    fn default_julia_bounds(&self, _param: ComplexNum) -> Bounds
    {
        Bounds::centered_square(4.)
    }
}

#[derive(Clone, Debug)]
pub struct QuadRatPreper21
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl QuadRatPreper21
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -1.1,
        max_x: 1.1,
        min_y: -1.1,
        max_y: 1.1,
    };
    fractal_impl!();
}

impl ParameterPlane for QuadRatPreper21
{
    parameter_plane_impl!();
    default_name!();

    fn encode_escaping_point(
        &self,
        iters: Period,
        z: ComplexNum,
        base_param: ComplexNum,
    ) -> PointInfo<Self::Var, Self::Deriv>
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
        c * (z + 2. + 1. / z)
    }

    #[inline]
    fn map_and_multiplier(&self, z: ComplexNum, c: ComplexNum) -> (ComplexNum, ComplexNum)
    {
        let u = z.inv();
        (c * (z + 2. + u), c * (1. - u * u))
    }

    fn start_point(&self, _point: ComplexNum, _c: ComplexNum) -> ComplexNum
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
        z + 1. / z + 2.
    }

    #[inline]
    fn critical_points(&self, _param: ComplexNum) -> ComplexVec
    {
        vec![(-1.).into(), (1.).into()]
    }

    #[inline]
    fn default_julia_bounds(&self, _param: ComplexNum) -> Bounds
    {
        Bounds::centered_square(4.)
    }
}

impl HasDynamicalCovers for QuadRatPreper21
{
    fn marked_cycle_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(ComplexNum) -> ComplexNum;
        let grid: PointGrid;
        let bounds: Bounds;

        match period
        {
            3 =>
            {
                param_map = |c| {
                    let u = c + 4.;
                    u * u / 4.
                };
                bounds = Bounds {
                    min_x: -6.2,
                    max_x: -1.8,
                    min_y: -2.2,
                    max_y: 2.2,
                };
                grid = self.point_grid.with_same_height(bounds);
            }
            4 =>
            {
                param_map = |c| {
                    let g2 = ComplexNum::new(-1. / 96., 0.);
                    let g3 = ComplexNum::new(-13. / 55296., 0.);
                    let (p, dp) = weierstrass_p(g2, g3, c, 0.01);

                    let x = p + 1.0 / 24.0;
                    let root_neg2_over_16 = ComplexNum::new(0., 0.088_388_347_648_318_4);
                    let mut y = dp * root_neg2_over_16;
                    // e4 = 8*x^3 - x^2 + 256*y^2 + x/16 - 1.0/1024.0

                    y += (1. - 32. * x) / 512.;
                    // e3 = 8*x^3 + 32*x*y + 256*y^2 - y

                    y /= x;
                    // e2 = 256*x*y^2 + 8*x^2 + 32*x*y - y

                    y / x
                };
                bounds = Bounds {
                    min_x: -16.,
                    max_x: 16.,
                    min_y: -12.,
                    max_y: 12.,
                };
                grid = self.point_grid.with_same_height(bounds);
            }
            _ =>
            {
                param_map = |c| c;
                grid = self.point_grid.clone();
            }
        }

        CoveringMap::new(self, param_map, grid)
    }
}

#[derive(Clone, Debug)]
pub struct CubicPer1_1
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl CubicPer1_1
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -2.5,
        max_x: 2.5,
        min_y: -2.2,
        max_y: 2.2,
    };
    fractal_impl!();
}

impl ParameterPlane for CubicPer1_1
{
    parameter_plane_impl!();
    default_name!();

    fn periodicity_tolerance(&self) -> RealNum
    {
        1e-6
    }
    fn encode_escaping_point(
        &self,
        iters: Period,
        z: ComplexNum,
        _base_param: ComplexNum,
    ) -> PointInfo<Self::Var, Self::Deriv>
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
        z * (z * (z + c) + 1.)
    }

    #[inline]
    fn start_point(&self, _point: ComplexNum, param: ComplexNum) -> ComplexNum
    {
        let mut u = (param * param - 3.).sqrt();
        if param.re < 0.
        {
            u = -u
        }
        -(param + u) / 3.
    }

    #[inline]
    fn dynamical_derivative(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        z * (2. * c + 3. * z) + 1.
    }

    #[inline]
    fn parameter_derivative(&self, z: ComplexNum, _c: ComplexNum) -> ComplexNum
    {
        z * z
    }

    #[inline]
    fn critical_points(&self, param: ComplexNum) -> ComplexVec
    {
        let u = (param * param - 3.).sqrt();
        vec![-(param + u) / 3., (u - param) / 3.]
    }

    #[inline]
    fn default_julia_bounds(&self, _param: ComplexNum) -> Bounds
    {
        Bounds::centered_square(2.2)
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
    ) -> PointInfo<Self::Var, Self::Deriv>
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
    fn critical_points(&self, param: ComplexNum) -> ComplexVec
    {
        let sqrt_c = param.sqrt();
        vec![-sqrt_c, sqrt_c]
    }

    #[inline]
    fn default_julia_bounds(&self, _param: ComplexNum) -> Bounds
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
    ) -> PointInfo<Self::Var, Self::Deriv>
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
    fn critical_points(&self, c: Self::Param) -> Vec<Self::Var>
    {
        let u = c + c.inv();
        vec![(0.).into(), TWO_THIRDS * u]
    }

    #[inline]
    fn default_julia_bounds(&self, param: ComplexNum) -> Bounds
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
    ) -> PointInfo<Self::Var, Self::Deriv>
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
    fn critical_points(&self, c: ComplexNum) -> ComplexVec
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
    fn default_julia_bounds(&self, param: ComplexNum) -> Bounds
    {
        Bounds::square(2.2, -param / 3.)
    }
}

#[derive(Clone, Debug)]
pub struct Biquadratic
{
    point_grid: PointGrid,
    max_iter: Period,
    param: ComplexNum,
}

impl Biquadratic
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -1.6,
        max_x: 1.25,
        min_y: -1.25,
        max_y: 1.25,
    };
    const JULIA_BOUNDS: Bounds = Bounds::centered_square(2.5);

    #[must_use]
    pub const fn new(
        res_x: usize,
        res_y: usize,
        max_iter: Period,
        param: ComplexNum,
        bounds: Bounds,
    ) -> Self
    {
        let point_grid = PointGrid::new(res_x, res_y, bounds);

        Self {
            point_grid,
            max_iter,
            param,
        }
    }

    #[must_use]
    pub const fn with_res_y(
        res_y: usize,
        max_iter: Period,
        param: ComplexNum,
        bounds: Bounds,
    ) -> Self
    {
        let point_grid = PointGrid::with_res_y(res_y, bounds);

        Self {
            point_grid,
            max_iter,
            param,
        }
    }

    #[must_use]
    pub const fn with_res_x(
        res_x: usize,
        max_iter: Period,
        param: ComplexNum,
        bounds: Bounds,
    ) -> Self
    {
        let point_grid = PointGrid::with_res_x(res_x, bounds);
        Self {
            point_grid,
            max_iter,
            param,
        }
    }

    #[must_use]
    pub const fn new_default(res_y: usize, max_iter: Period, param: ComplexNum) -> Self
    {
        let bounds = Self::DEFAULT_BOUNDS;
        Self::with_res_y(res_y, max_iter, param, bounds)
    }
}

impl ParameterPlane for Biquadratic
{
    parameter_plane_impl!();

    #[inline]
    fn name(&self) -> String
    {
        let param = self.param;
        format!("Biquadratic({param})")
    }

    fn encode_escaping_point(
        &self,
        iters: Period,
        z: ComplexNum,
        _base_param: ComplexNum,
    ) -> PointInfo<Self::Var, Self::Deriv>
    {
        if z.is_nan()
        {
            return PointInfo::Escaping {
                potential: f64::from(iters) - 1.,
            };
        }

        let u = self.escape_radius().log2();
        let v = z.norm_sqr().log2();
        let residual = (v / u).log2() / 2.;
        let potential = f64::from(iters) - (residual as IterCount);
        PointInfo::Escaping { potential }
    }

    #[inline]
    fn param_map(&self, c: ComplexNum) -> ComplexNum
    {
        c
    }

    #[inline]
    fn start_point(&self, _point: ComplexNum, _c: ComplexNum) -> ComplexNum
    {
        ComplexNum::new(0., 0.)
    }

    #[inline]
    fn map(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        let u = z * z + c;
        u * u + self.param
    }

    #[inline]
    fn map_and_multiplier(&self, z: ComplexNum, c: ComplexNum) -> (ComplexNum, ComplexNum)
    {
        let u = z * z + c;
        (u * u + self.param, 4. * u * z)
    }

    #[inline]
    fn dynamical_derivative(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        4. * z * (z * z + c)
    }

    #[inline]
    fn parameter_derivative(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        2. * (z * z + c)
    }

    #[inline]
    fn gradient(&self, z: ComplexNum, c: ComplexNum) -> (ComplexNum, ComplexNum, ComplexNum)
    {
        let z2 = z * z;
        let w = z2 + c;
        let w2 = w * w;
        (w2 + self.param, 4. * z2, w + w)
    }
}

#[derive(Clone, Debug)]
pub struct BiquadraticMult
{
    point_grid: PointGrid,
    max_iter: Period,
    param: ComplexNum,
}

impl BiquadraticMult
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -2.6,
        max_x: 3.25,
        min_y: -2.25,
        max_y: 2.25,
    };
    const JULIA_BOUNDS: Bounds = Bounds::centered_square(2.5);

    #[must_use]
    pub const fn new(
        res_x: usize,
        res_y: usize,
        max_iter: Period,
        param: ComplexNum,
        bounds: Bounds,
    ) -> Self
    {
        let point_grid = PointGrid::new(res_x, res_y, bounds);

        Self {
            point_grid,
            max_iter,
            param,
        }
    }

    #[must_use]
    pub const fn with_res_y(
        res_y: usize,
        max_iter: Period,
        param: ComplexNum,
        bounds: Bounds,
    ) -> Self
    {
        let point_grid = PointGrid::with_res_y(res_y, bounds);

        Self {
            point_grid,
            max_iter,
            param,
        }
    }

    #[must_use]
    pub const fn with_res_x(
        res_x: usize,
        max_iter: Period,
        param: ComplexNum,
        bounds: Bounds,
    ) -> Self
    {
        let point_grid = PointGrid::with_res_x(res_x, bounds);
        Self {
            point_grid,
            max_iter,
            param,
        }
    }

    #[must_use]
    pub const fn new_default(res_y: usize, max_iter: Period, param: ComplexNum) -> Self
    {
        let bounds = Self::DEFAULT_BOUNDS;
        Self::with_res_y(res_y, max_iter, param, bounds)
    }
}

impl ParameterPlane for BiquadraticMult
{
    parameter_plane_impl!();

    #[inline]
    fn name(&self) -> String
    {
        let param = self.param;
        format!("BiquadraticMult({param})")
    }

    fn encode_escaping_point(
        &self,
        iters: Period,
        z: ComplexNum,
        _base_param: ComplexNum,
    ) -> PointInfo<Self::Var, Self::Deriv>
    {
        if z.is_nan()
        {
            return PointInfo::Escaping {
                potential: f64::from(iters) - 1.,
            };
        }

        let u = self.escape_radius().log2();
        let v = z.norm_sqr().log2();
        let residual = (v / u).log2() / 2.;
        let potential = f64::from(iters) - (residual as IterCount);
        PointInfo::Escaping { potential }
    }

    #[inline]
    fn param_map(&self, c: ComplexNum) -> ComplexNum
    {
        c
    }

    #[inline]
    fn start_point(&self, _point: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        -0.5 * c
    }

    #[inline]
    fn map(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        let w = z * (z + c);
        w * (w + self.param / c)
    }

    #[inline]
    fn map_and_multiplier(&self, z: ComplexNum, c: ComplexNum) -> (ComplexNum, ComplexNum)
    {
        let a = self.param / c;
        let w = z * (z + c);
        (w * (w + a), (c + z + z) * (a + w + w))
    }

    #[inline]
    fn dynamical_derivative(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        let a = self.param / c;
        let w = z * (z + c);
        (c + z + z) * (a + w + w)
    }

    #[inline]
    fn parameter_derivative(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        2. * (z * z + c)
    }

    #[inline]
    fn gradient(&self, z: Self::Var, c: Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        let a = self.param / c;
        let x0 = c + z;
        let w = z * x0;
        let x2 = w + a;
        let x2z = x2 * z;
        (
            w * x2,
            x0 * x2 + w * (c + z + z) + x2z,
            w * (z - a * a) + x2z,
        )
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
    ) -> PointInfo<Self::Var, Self::Deriv>
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
        ONE_COMPLEX //TODO
    }

    #[inline]
    fn parameter_derivative(&self, _z: ComplexNum, _c: ComplexNum) -> ComplexNum
    {
        ONE_COMPLEX //TODO
    }

    #[inline]
    fn gradient(&self, _z: ComplexNum, _c: ComplexNum) -> (ComplexNum, ComplexNum, ComplexNum)
    {
        (ONE_COMPLEX, ONE_COMPLEX, ONE_COMPLEX) //TODO
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

    #[must_use]
    pub const fn new(
        res_x: usize,
        res_y: usize,
        max_iter: Period,
        shift: ComplexNum,
        bounds: Bounds,
    ) -> Self
    {
        let point_grid = PointGrid::new(res_x, res_y, bounds);

        Self {
            point_grid,
            max_iter,
            shift,
        }
    }

    #[must_use]
    pub const fn with_res_y(
        res_y: usize,
        max_iter: Period,
        shift: ComplexNum,
        bounds: Bounds,
    ) -> Self
    {
        let point_grid = PointGrid::with_res_y(res_y, bounds);

        Self {
            point_grid,
            max_iter,
            shift,
        }
    }

    #[must_use]
    pub const fn with_res_x(
        res_x: usize,
        max_iter: Period,
        shift: ComplexNum,
        bounds: Bounds,
    ) -> Self
    {
        let point_grid = PointGrid::with_res_x(res_x, bounds);
        Self {
            point_grid,
            max_iter,
            shift,
        }
    }

    #[must_use]
    pub const fn new_default(res_y: usize, max_iter: Period, shift: ComplexNum) -> Self
    {
        let bounds = Self::DEFAULT_BOUNDS;
        Self::with_res_y(res_y, max_iter, shift, bounds)
    }
}

impl ParameterPlane for Sailboat
{
    parameter_plane_impl!();

    fn encode_escaping_point(
        &self,
        iters: Period,
        z: ComplexNum,
        _base_param: ComplexNum,
    ) -> PointInfo<Self::Var, Self::Deriv>
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
        ONE_COMPLEX //TODO
    }

    #[inline]
    fn parameter_derivative(&self, _z: ComplexNum, _c: ComplexNum) -> ComplexNum
    {
        ONE_COMPLEX //TODO
    }

    #[inline]
    fn gradient(&self, _z: Self::Var, _c: Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        (ONE_COMPLEX, ONE_COMPLEX, ONE_COMPLEX) //TODO
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
    ) -> PointInfo<Self::Var, Self::Deriv>
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
                final_error: (1e-8).into(),
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
        ONE_COMPLEX
    }

    #[inline]
    fn gradient(&self, z: ComplexNum, c: ComplexNum) -> (ComplexNum, ComplexNum, ComplexNum)
    {
        let u = z.exp();
        (u + c, u, ONE_COMPLEX)
    }

    #[inline]
    fn param_map(&self, c: ComplexNum) -> ComplexNum
    {
        c
    }
}
