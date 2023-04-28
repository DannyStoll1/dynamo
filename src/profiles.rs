use crate::dynamics::{HasDynamicalCovers, ParameterPlane, covering_maps::CoveringMap};
use crate::math_utils::{slog, weierstrass_p};
use crate::point_grid::{Bounds, PointGrid};
use crate::types::*;

use crate::macros::{default_name, fractal_impl, parameter_plane_impl};

use std::any::type_name;

#[derive(Clone, Copy, Debug)]
pub struct Mandelbrot
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl Mandelbrot
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -2.2,
        max_x: 0.65,
        min_y: -1.4,
        max_y: 1.4,
    };
    fractal_impl!();
}

impl ParameterPlane for Mandelbrot
{
    parameter_plane_impl!();
    default_name!();

    fn encode_escaping_point(
        &self,
        iters: Period,
        z: ComplexNum,
        _base_param: ComplexNum,
    ) -> IterCount
    {
        if z.is_nan()
        {
            return f64::from(iters) - 1.;
        }

        let u = self.escape_radius().log2();
        let v = z.norm_sqr().log2();
        let residual = (v / u).log2();
        f64::from(iters) - (residual as IterCount)
    }

    #[inline]
    fn map(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        z * z + c
    }

    #[inline]
    fn dynamical_derivative(&self, z: ComplexNum, _c: ComplexNum) -> ComplexNum
    {
        z + z
    }

    #[inline]
    fn parameter_derivative(&self, _z: ComplexNum, _c: ComplexNum) -> ComplexNum
    {
        ONE_COMPLEX
    }

    #[inline]
    fn gradient(&self, z: ComplexNum, _c: ComplexNum) -> (ComplexNum, ComplexNum)
    {
        (z + z, ONE_COMPLEX)
    }

    #[inline]
    fn default_julia_bounds(&self, _param: ComplexNum) -> Bounds
    {
        Bounds::centered_square(2.2)
    }
}

impl HasDynamicalCovers for Mandelbrot
{
    fn marked_cycle_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(ComplexNum) -> ComplexNum;
        let bounds: Bounds;

        match period
        {
            1 =>
            {
                param_map = |c| 0.25 - c * c;
                bounds = Bounds {
                    min_x: -1.8,
                    max_x: 1.8,
                    min_y: -1.0,
                    max_y: 1.0,
                };
            }
            3 =>
            {
                param_map = |c| -1.75 * (1. + 7. * c * c);
                bounds = Bounds {
                    min_x: -0.3,
                    max_x: 0.3,
                    min_y: -0.5,
                    max_y: 0.5,
                };
            }
            4 =>
            {
                param_map = |c| {
                    let u = c * c;
                    -0.25 * u - 0.75 - 1. / c
                };
                bounds = Bounds {
                    min_x: -2.9,
                    max_x: 2.1,
                    min_y: -3.1,
                    max_y: 3.1,
                };
            }
            _ =>
            {
                param_map = |c| c;
                bounds = self.point_grid.bounds;
            }
        };
        let grid = self.point_grid.with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }

    fn dynatomic_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(ComplexNum) -> ComplexNum;
        let bounds: Bounds;

        match period
        {
            1 =>
            {
                param_map = |c| 0.25 - c * c;
                bounds = Bounds {
                    min_x: -1.8,
                    max_x: 1.8,
                    min_y: -1.0,
                    max_y: 1.0,
                };
            }
            3 =>
            {
                param_map = |c| {
                    let c2 = c * c;
                    let v = c2 * (c2 - 3. * c + 6.) - c - c + 2.;
                    let u = v + 1. / (c2 - c);
                    -0.25 * u / (c2 - c)
                };
                bounds = Bounds {
                    min_x: -2.5,
                    max_x: 3.5,
                    min_y: -3.,
                    max_y: 3.,
                };
            }
            _ =>
            {
                param_map = |c| c;
                bounds = self.point_grid.bounds;
            }
        };
        let grid = self.point_grid.with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }
    fn misiurewicz_curve(self, preperiod: Period, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(ComplexNum) -> ComplexNum;
        let bounds: Bounds;

        match (preperiod, period)
        {
            (2, 1) =>
            {
                param_map = |c| {
                    let c2 = c * c;
                    -2. * (c2 + 1.) / ((c2 - 1.) * (c2 - 1.))
                };
                bounds = Bounds {
                    min_x: -3.5,
                    max_x: 3.5,
                    min_y: -3.0,
                    max_y: 3.0,
                };
            }
            (2, 2) =>
            {
                param_map = |c| {
                    let c2 = c * c;
                    -(c2 * (c2 + c + c + 2.) - c - c + 1.) / (4. * c2)
                };
                bounds = Bounds {
                    min_x: -4.,
                    max_x: 2.4,
                    min_y: -2.5,
                    max_y: 2.5,
                };
            }
            (_, _) =>
            {
                param_map = |c| c;
                bounds = self.point_grid.bounds;
            }
        };
        let grid = self.point_grid.with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct QuadRatPer2
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl QuadRatPer2
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -2.8,
        max_x: 3.2,
        min_y: -2.8,
        max_y: 2.8,
    };
    fractal_impl!();
}

impl ParameterPlane for QuadRatPer2
{
    parameter_plane_impl!();
    default_name!();

    fn encode_escaping_point(
        &self,
        iters: Period,
        z: ComplexNum,
        _base_param: ComplexNum,
    ) -> IterCount
    {
        if z.is_nan()
        {
            return f64::from(iters) - 2.;
        }

        let u = self.escape_radius().log2();
        let v = z.norm_sqr().log2();
        // let q = ((base_param - 1.) / (4. * base_param)).norm().log2();
        let q = -1.;
        let residual = ((u + q) / (v + q)).log2();
        // let residual = ((v - 1.) / (u + u - 1.)).log2() + 1.;
        // (F - M) / (2L - M)
        f64::from(iters) + (residual as IterCount) * 2.
    }

    #[inline]
    fn map(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        (z * z + c) / (z * z - 1.)
        // c / (z*z + 2.*z)
        // c / z + 1. / (z * z)
    }

    // fn start_point(&self, c: ComplexNum) -> ComplexNum {
    //     -2. / c
    //     (-1.).into()
    // }

    #[inline]
    fn dynamical_derivative(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        let u = 1. / (z * z - 1.);
        -TWO * (c + 1.) * z * u * u
    }

    #[inline]
    fn parameter_derivative(&self, z: ComplexNum, _c: ComplexNum) -> ComplexNum
    {
        1. / (z * z - 1.)
    }

    #[inline]
    fn gradient(&self, z: ComplexNum, c: ComplexNum) -> (ComplexNum, ComplexNum)
    {
        let u = 1. / (z * z - 1.);
        (-TWO * (c + 1.) * z * u * u, u)
    }

    #[inline]
    fn default_julia_bounds(&self, param: ComplexNum) -> Bounds
    {
        Bounds::centered_square(4.)
    }
}

impl HasDynamicalCovers for QuadRatPer2
{
    fn marked_cycle_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(ComplexNum) -> ComplexNum;
        let bounds: Bounds;

        match period
        {
            1 =>
            {
                param_map = |c| (4. - c * (c + 2.)) * c / 8.;
                bounds = Bounds {
                    min_x: -5.0,
                    max_x: 3.0,
                    min_y: -3.0,
                    max_y: 3.0,
                };
            }
            4 =>
            {
                param_map = |c| {
                    let u = c * c;
                    u * c - 2. * u + 4. * c - 1.
                };
                bounds = Bounds {
                    min_x: -1.,
                    max_x: 1.4,
                    min_y: -2.2,
                    max_y: 2.2,
                };
            }
            5 =>
            {
                param_map = |c| {
                    // t = sqrt(-2235)
                    // ((-2043332879690812551104*t + 322671215001188162496)*c^6 + (-7211787718815174272*t + 38457203855637713472)*c^5 + (-10445615819508480*t + 113836835145028800)*c^4 + (-7931553616080*t + 135137329840080)*c^3 + (-3321323160*t + 79799557200)*c^2 + (-724598*t + 23400162)*c + (-64*t + 2724))/((-165726073638468871360*t + 59671792608719217337728)*c^6 + (-532082528560799520*t + 218792941658814953376)*c^5 + (-681491680626360*t + 334169395252260120)*c^4 + (-435333784880*t + 272101938829200)*c^3 + (-138715290*t + 124564255830)*c^2 + (-17640*t + 30391956)*c + 3087)
                    let pole = ComplexNum::new(-1.029_131_872_704_64, 0.051_564_155_271_414_3);
                    let angle = ComplexNum::new(1., 0.);

                    let c = angle / c + pole;

                    let a0 = ComplexNum::new(-5448., 6_051.300_686_629_28);
                    let a1 = ComplexNum::new(-29_961.795_134_443_0, 43_861.639_473_933_7);
                    let a2 = ComplexNum::new(-65_413.655_299_273_2, 128_711.643_030_672);
                    let a3 = ComplexNum::new(-70_918.940_786_376_0, 196_781.349_743_989);
                    let a4 = ComplexNum::new(-38_246.235_127_179_3, 165_912.340_564_512);
                    let a5 = ComplexNum::new(-8_271.848_132_127_45, 73_334.197_922_255_2);
                    let a6 = ComplexNum::new(-44.432_836_932_486_6, 13_302.145_857_037_4);

                    let b0 = ComplexNum::new(-6174., 0.);
                    let b1 = ComplexNum::new(-38_914.156_209_987_2, 1_067.791_134_284_38);
                    let b2 = ComplexNum::new(-102_108.377_281_498, 5_375.650_615_514_38);
                    let b3 = ComplexNum::new(-142_796.822_391_875, 10_800.604_008_295_7);
                    let b4 = ComplexNum::new(-112_272.282_050_380, 10_824.434_074_704_7);
                    let b5 = ComplexNum::new(-47_060.675_356_870_1, 5_410.564_894_838_89);
                    let b6 = ComplexNum::new(-8_216.992_738_080_66, 1_078.880_698_179_05);

                    let numer = a0 + c * (a1 + c * (a2 + c * (a3 + c * (a4 + c * (a5 + c * a6)))));
                    let denom = b0 + c * (b1 + c * (b2 + c * (b3 + c * (b4 + c * (b5 + c * b6)))));

                    -numer / denom
                };
                bounds = Bounds {
                    min_x: -8.,
                    max_x: 5.5,
                    min_y: -1.5,
                    max_y: 8.,
                };
            }
            _ =>
            {
                param_map = |c| c;
                bounds = self.point_grid.bounds;
            }
        };
        let grid = self.point_grid.with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }

    fn misiurewicz_curve(self, preperiod: Period, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(ComplexNum) -> ComplexNum;
        let bounds: Bounds;

        match (preperiod, period)
        {
            (2, 1) =>
            {
                param_map = |c| {
                    let c2 = c * c;
                    // -25*(131*t^4 - 102*t^3 - 106*t^2 - 8*t - 4)*t^2/(13*t^2 + 2*t + 2)^3
                    let denom = 13. * c2 + c + c + 2.;
                    let numer = c2 * (131. * c2 - 102. * c - 106.) - 8. * c - 4.;
                    25. * c2 * numer / (denom * denom * denom)
                };
                bounds = Bounds {
                    min_x: -3.4,
                    max_x: 3.4,
                    min_y: -5.1,
                    max_y: 5.1,
                };
            }
            (2, 2) =>
            {
                param_map = |c| {
                    //(-t^4 + 2*t^2 + 1)/(2*t^4)
                    let c2 = c * c;
                    0.5 - (c2 + 0.5) / (c2 * c2)
                };
                bounds = Bounds {
                    min_x: -4.,
                    max_x: 4.,
                    min_y: -4.,
                    max_y: 4.,
                };
            }
            (_, _) =>
            {
                param_map = |c| c;
                bounds = self.point_grid.bounds;
            }
        };
        let grid = self.point_grid.with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }
}

#[derive(Clone, Copy, Debug)]
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

    fn start_point(&self, _c: ComplexNum) -> ComplexNum
    {
        0.0.into()
    }

    fn encode_escaping_point(
        &self,
        iters: Period,
        z: ComplexNum,
        base_param: ComplexNum,
    ) -> IterCount
    {
        if z.is_nan()
        {
            return f64::from(iters) - 3.;
        }

        let u = self.escape_radius().log2();
        let v = z.norm_sqr().log2();
        let q = ((base_param - 1.) / (4. * base_param)).norm().log2();
        let residual = ((u + q) / (v + q)).log2();
        f64::from(iters) + (residual as IterCount) * 3.
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
    fn gradient(&self, z: ComplexNum, c: ComplexNum) -> (ComplexNum, ComplexNum)
    {
        let r = c * c - z * z;
        let u = 1. / r;
        let u2 = u * u;
        let v = c + 1.;
        let df_dz = TWO * (1. - c) * v * v * z * u2;
        let df_dc = v * u2 * (r - c * (r + TWO * (ONE_COMPLEX - z * z)));
        (df_dz, df_dc)
    }

    #[inline]
    fn default_julia_bounds(&self, param: ComplexNum) -> Bounds
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

                    let x0 = yy / zz;
                    let x1 = xx * yy / (zz * zz);

                    let s0 = x1 / x0;
                    let s1 = 1. / x0;

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
                bounds = self.point_grid.bounds;
            }
        };
        let grid = self.point_grid.with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }
}

#[derive(Clone, Copy, Debug)]
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

    fn encode_escaping_point(&self, iters: Period, z: ComplexNum, c: ComplexNum) -> IterCount
    {
        {
            if z.is_nan()
            {
                return f64::from(iters) - 4.;
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
            f64::from(iters) + (residual as IterCount) * 4.
        }
    }

    #[inline]
    fn param_map(&self, c: ComplexNum) -> ComplexNum
    {
        let pole = 2.618_033_988_749_89;
        1. / c + pole
    }

    #[inline]
    fn start_point(&self, c: ComplexNum) -> ComplexNum
    {
        (c + c) * (c + c - 1.) / (c * (c + 1.) - 1.)
    }

    #[inline]
    fn map(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        (c * z - c - c - z + 1.) * (z - c) / (z * z * (c - 1.))
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
    fn gradient(&self, _z: ComplexNum, _c: ComplexNum) -> (ComplexNum, ComplexNum)
    {
        (ONE_COMPLEX, ONE_COMPLEX) //TODO
    }

    #[inline]
    fn default_julia_bounds(&self, param: ComplexNum) -> Bounds
    {
        Bounds::centered_square(4.)
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
                    // (18)^(2/3) / 3
                    let alpha = ComplexNum::new(2.289_428_485_106_66, 0.);
                    let g2 = alpha;
                    let g3 = ComplexNum::new(-19. / 12., 0.);

                    let (p, dp) = weierstrass_p(g2, g3, c, 0.01);
                    let x = (alpha * p + 1.) / 3.;
                    let y = (dp - 1.5) / x;

                    let xx = x + 1.;
                    let yy = y - 3. * x - 3.;

                    let x0 = yy / x;
                    let _s1 = x0 * xx / x;

                    x / xx
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
                grid = self.point_grid;
            }
        };
        CoveringMap::new(self, param_map, grid)
    }
}

#[derive(Clone, Copy, Debug)]
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
    ) -> IterCount
    {
        if z.is_nan()
        {
            return f64::from(iters) - 1.;
        }

        let u = self.escape_radius().log2();
        let v = z.norm_sqr().log2();
        let residual = (v / u).log(3.);
        f64::from(iters) - (residual as IterCount)
    }

    #[inline]
    fn map(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        z * (z * (z + c) + 1.)
    }

    #[inline]
    fn start_point(&self, param: ComplexNum) -> ComplexNum
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
    fn default_julia_bounds(&self, param: ComplexNum) -> Bounds
    {
        Bounds::centered_square(2.2)
    }
}

#[derive(Clone, Copy, Debug)]
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
    ) -> IterCount
    {
        if z.is_nan()
        {
            return f64::from(iters) - 1.;
        }

        let u = self.escape_radius().log2();
        let v = z.norm_sqr().log2();
        let residual = (v / u).log(3.);
        f64::from(iters) - (residual as IterCount)
    }

    #[inline]
    fn map(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        2. * (z * z * z / 3. - c * z)
    }

    #[inline]
    fn start_point(&self, param: ComplexNum) -> ComplexNum
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
    fn default_julia_bounds(&self, param: ComplexNum) -> Bounds
    {
        Bounds::centered_square(2.2)
    }
}

// Cubic polynomials with critical 2-cycle 0 <-> c
#[derive(Clone, Copy, Debug)]
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
    ) -> IterCount
    {
        if z.is_nan()
        {
            return f64::from(iters) - 1.;
        }

        let u = self.escape_radius().log2();
        let v = z.norm_sqr().log2();
        let residual = (v / u).log(3.);
        f64::from(iters) - (residual as IterCount)
    }

    #[inline]
    fn map(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        z * z * (z - c - 1. / c) + c
    }

    #[inline]
    fn start_point(&self, param: ComplexNum) -> ComplexNum
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
    fn default_julia_bounds(&self, param: ComplexNum) -> Bounds
    {
        Bounds::square(2.2, param / 2.)
    }
}
#[derive(Clone, Copy, Debug)]
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
    ) -> IterCount
    {
        if z.is_nan()
        {
            return f64::from(iters) - 1.;
        }

        let u = self.escape_radius().log2();
        let v = z.norm_sqr().log2();
        let residual = (v / u).log2() / 2.;
        f64::from(iters) - (residual as IterCount)
    }

    #[inline]
    fn param_map(&self, c: ComplexNum) -> ComplexNum
    {
        c
    }

    #[inline]
    fn start_point(&self, _c: ComplexNum) -> ComplexNum
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
    fn gradient(&self, z: ComplexNum, c: ComplexNum) -> (ComplexNum, ComplexNum)
    {
        let w = z * z + c;
        (4. * z * w, w + w)
    }
}

#[derive(Clone, Copy, Debug)]
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
    ) -> IterCount
    {
        if z.is_nan()
        {
            return f64::from(iters) - 1.;
        }

        let u = self.escape_radius().log2();
        let v = z.norm_sqr().log2();
        let residual = (v / u).log2();
        f64::from(iters) - (residual as IterCount)
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
    fn gradient(&self, _z: ComplexNum, _c: ComplexNum) -> (ComplexNum, ComplexNum)
    {
        (ONE_COMPLEX, ONE_COMPLEX) //TODO
    }

    #[inline]
    fn param_map(&self, c: ComplexNum) -> ComplexNum
    {
        c
    }
}

#[derive(Clone, Copy, Debug)]
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
    ) -> IterCount
    {
        if z.is_nan()
        {
            return f64::from(iters) - 1.;
        }

        let u = self.escape_radius().log2();
        let v = z.norm_sqr().log2();
        let residual = (v / u).log2();
        f64::from(iters) - (residual as IterCount)
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
    fn gradient(&self, _z: ComplexNum, _c: ComplexNum) -> (ComplexNum, ComplexNum)
    {
        (ONE_COMPLEX, ONE_COMPLEX) //TODO
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

#[derive(Clone, Copy, Debug)]
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
    ) -> IterCount
    {
        if z.is_nan()
        {
            return f64::from(iters) - 1.;
        }

        if z.re < 0.
        {
            return -1.;
        }
        if z.is_infinite()
        {
            return f64::from(iters + 1);
        }
        let u = slog(self.escape_radius());
        let v = slog(z.norm_sqr());
        let residual = v - u;
        f64::from(iters) - (residual as IterCount)
    }

    #[inline]
    fn map(&self, z: ComplexNum, c: ComplexNum) -> ComplexNum
    {
        z.exp() + c
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
    fn gradient(&self, z: ComplexNum, _c: ComplexNum) -> (ComplexNum, ComplexNum)
    {
        (z.exp(), ONE_COMPLEX)
    }

    #[inline]
    fn param_map(&self, c: ComplexNum) -> ComplexNum
    {
        c
    }
}
