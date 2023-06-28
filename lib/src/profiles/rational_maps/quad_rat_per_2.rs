use crate::macros::profile_imports;
profile_imports!();

#[inline]
fn map(z: Cplx, c: Cplx) -> Cplx
{
    (z * z + c) / (z * z - 1.)
}
#[inline]
fn map_and_multiplier(z: Cplx, c: Cplx) -> (Cplx, Cplx)
{
    let z2 = z * z;
    let u = z2 - 1.;
    ((c + z2) / u, -2.0 * z * (c + 1.) / (u * u))
}

#[inline]
fn dynamical_derivative(z: Cplx, c: Cplx) -> Cplx
{
    let u = 1. / (z * z - 1.);
    -2.0 * (c + 1.) * z * u * u
}

#[inline]
fn parameter_derivative(z: Cplx, _c: Cplx) -> Cplx
{
    1. / (z * z - 1.)
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct QuadRatPer2
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl Default for QuadRatPer2
{
    fractal_impl!(-2.8, 3.2, -2.8, 2.8);
}

impl ParameterPlane for QuadRatPer2
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
                potential: f64::from(iters) - 2.,
            };
        }

        let u = self.escape_radius().log2();
        let v = z.norm_sqr().log2();
        // let q = ((base_param - 1.) / (4. * base_param)).norm().log2();
        let q = -1.;
        let residual = ((u + q) / (v + q)).log2();
        // let residual = ((v - 1.) / (u + u - 1.)).log2() + 1.;
        // (F - M) / (2L - M)
        let potential = (residual as IterCount).mul_add(2., f64::from(iters));
        PointInfo::Escaping { potential }
    }

    #[inline]
    fn map(&self, z: Self::Var, c: Self::Param) -> Self::Var
    {
        map(z, c)
    }

    // fn start_point(&self, c: ComplexNum) -> ComplexNum {
    //     -2. / c
    //     (-1.).into()
    // }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, c: Self::Param) -> (Self::Var, Self::Deriv)
    {
        map_and_multiplier(z, c)
    }

    #[inline]
    fn dynamical_derivative(&self, z: Self::Var, c: Self::Param) -> Self::Deriv
    {
        dynamical_derivative(z, c)
    }

    #[inline]
    fn parameter_derivative(&self, z: Self::Var, c: Self::Param) -> Self::Deriv
    {
        parameter_derivative(z, c)
    }

    #[inline]
    fn critical_points_child(&self, _param: Cplx) -> ComplexVec
    {
        vec![(0.).into()]
    }

    fn cycles_child(&self, c: Cplx, period: Period) -> ComplexVec
    {
        match period
        {
            1 =>
            {
                let u = -27. * c;
                let v = u - 11.;
                let x0 = (0.5 * (u + (v * v - 256.).sqrt() - 11.)).powf(ONE_THIRD);
                let x1 = 4. / x0 * ONE_THIRD;
                let x2 = x0 * ONE_THIRD;
                let r1 = -x1 * OMEGA_BAR - x2 * OMEGA + ONE_THIRD;
                let r2 = -x1 * OMEGA - x2 * OMEGA_BAR + ONE_THIRD;
                vec![-x1 - x2 + ONE_THIRD, r1, r2]
            }
            2 =>
            {
                vec![(1.).into()]
            }
            _ => vec![],
        }
    }

    #[inline]
    fn default_julia_bounds(&self, _point: Cplx, _param: Cplx) -> Bounds
    {
        Bounds::centered_square(4.)
    }
}

const A0: Cplx = Cplx::new(-5448., 6_051.300_686_629_28);
const A1: Cplx = Cplx::new(-29_961.795_134_443_0, 43_861.639_473_933_7);
const A2: Cplx = Cplx::new(-65_413.655_299_273_2, 128_711.643_030_672);
const A3: Cplx = Cplx::new(-70_918.940_786_376_0, 196_781.349_743_989);
const A4: Cplx = Cplx::new(-38_246.235_127_179_3, 165_912.340_564_512);
const A5: Cplx = Cplx::new(-8_271.848_132_127_45, 73_334.197_922_255_2);
const A6: Cplx = Cplx::new(-44.432_836_932_486_6, 13_302.145_857_037_4);

const B0: Cplx = Cplx::new(-6174., 0.);
const B1: Cplx = Cplx::new(-38_914.156_209_987_2, 1_067.791_134_284_38);
const B2: Cplx = Cplx::new(-102_108.377_281_498, 5_375.650_615_514_38);
const B3: Cplx = Cplx::new(-142_796.822_391_875, 10_800.604_008_295_7);
const B4: Cplx = Cplx::new(-112_272.282_050_380, 10_824.434_074_704_7);
const B5: Cplx = Cplx::new(-47_060.675_356_870_1, 5_410.564_894_838_89);
const B6: Cplx = Cplx::new(-8_216.992_738_080_66, 1_078.880_698_179_05);

impl HasDynamicalCovers for QuadRatPer2
{
    fn marked_cycle_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> Cplx;
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
                    let pole = Cplx::new(-1.029_131_872_704_64, 0.051_564_155_271_414_3);
                    let angle = Cplx::new(1., 0.);

                    let c = angle / c + pole;

                    let numer = A0 + c * (A1 + c * (A2 + c * (A3 + c * (A4 + c * (A5 + c * A6)))));
                    let denom = B0 + c * (B1 + c * (B2 + c * (B3 + c * (B4 + c * (B5 + c * B6)))));

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
                bounds = self.point_grid.bounds.clone();
            }
        };
        let grid = self.point_grid.new_with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }
}