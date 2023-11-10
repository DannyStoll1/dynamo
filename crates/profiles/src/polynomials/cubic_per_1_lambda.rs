use crate::macros::{horner, horner_monic, profile_imports};
use dynamo_common::types::variables::PlaneID;
profile_imports!();

const I: Cplx = Cplx::new(0., 1.);
const I2: Cplx = Cplx::new(0., 2.);
const I10: Cplx = Cplx::new(0., 10.);
const A0: Cplx = Cplx::new(0., 27. / 64.);
const A2: Cplx = Cplx::new(0., -21. / 16.);
const A4: Cplx = Cplx::new(0., -7. / 4.);

const B2: Cplx = Cplx::new(0., -21. / 8.);
const B4: Cplx = Cplx::new(0., -7.);
const I6: Cplx = Cplx::new(0., 6.);

#[derive(Clone, Debug)]
pub struct CubicPer1Lambda
{
    point_grid: PointGrid,
    max_iter: Period,
    multiplier: Cplx,
    starting_crit: PlaneID,
}

impl CubicPer1Lambda
{
    const DEFAULT_BOUNDS: Bounds = Bounds::centered_square(6.5);
}

impl Default for CubicPer1Lambda
{
    fn default() -> Self
    {
        let bounds = Self::DEFAULT_BOUNDS;
        let point_grid = PointGrid::new_by_res_y(1024, bounds);
        Self {
            point_grid,
            max_iter: 1024,
            multiplier: ZERO,
            starting_crit: PlaneID::ZPlane,
        }
    }
}

impl DynamicalFamily for CubicPer1Lambda
{
    parameter_plane_impl!(Cplx, Cplx, Cplx, Cplx);

    #[inline]
    fn map(&self, z: Self::Var, c: &Self::Param) -> Self::Var
    {
        z * horner_monic!(z, self.multiplier, c)
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv)
    {
        let z2 = z.powi(2);
        let u = z2 + c * z + self.multiplier;
        (z * u, u + z * (c + 2. * z))
    }

    #[inline]
    fn gradient(&self, z: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        let z2 = z.powi(2);
        let u = z2 + c * z + self.multiplier;
        (z * u, u + z * (c + 2. * z), z2)
    }

    #[inline]
    fn start_point(&self, t: Cplx, _c: &Self::Param) -> Self::Var
    {
        match self.starting_crit {
            PlaneID::ZPlane => 0.5 * self.multiplier * t,
            PlaneID::WPlane => TWO_THIRDS / t,
        }
    }

    #[inline]
    fn start_point_d(&self, t: Cplx, _c: &Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        match self.starting_crit {
            PlaneID::ZPlane => (0.5 * self.multiplier * t, 0.5 * self.multiplier, ZERO),
            PlaneID::WPlane => (TWO_THIRDS / t, -ONE_THIRD / t.powi(2), ZERO),
        }
    }

    fn get_meta_params(&self) -> Self::Param
    {
        self.multiplier
    }

    fn get_param(&self) -> Self::Param
    {
        self.multiplier
    }

    fn set_meta_param(&mut self, value: Self::Param)
    {
        self.multiplier = value;
    }

    fn set_param(&mut self, value: <Self::MetaParam as ParamList>::Param)
    {
        self.multiplier = value;
    }

    fn param_map(&self, t: Cplx) -> Self::Param
    {
        -t.inv() - 0.75 * self.multiplier * t
    }

    fn name(&self) -> String
    {
        format!("Cubic Per(1, {}) {}", self.multiplier, self.starting_crit)
    }

    fn default_bounds(&self) -> Bounds
    {
        let r = 4. / (self.multiplier.norm() + 0.01);
        Bounds::centered_square(r)
    }

    fn cycle_active_plane(&mut self)
    {
        self.starting_crit = self.starting_crit.swap();
    }
}

impl MarkedPoints for CubicPer1Lambda
{
    fn critical_points(&self) -> Vec<Self::Var>
    {
        let l = self.multiplier;
        let d0 = l.sqrt();
        let d1 = (l - 2.).sqrt();
        let u = 2. / l;
        let r0 = u * d0;
        let r1 = u * d1;
        vec![r0, -r0, r1, -r1, u, -u]
    }

    fn critical_points_child(&self, c: &Self::Param) -> Vec<Self::Var>
    {
        let disc = (c.powi(2) - 3. * self.multiplier).sqrt();
        vec![-ONE_THIRD * (c + disc), -ONE_THIRD * (c - disc)]
    }

    fn cycles(&self, period: Period) -> Vec<Self::Var>
    {
        match period {
            1 => {
                let u = (self.multiplier - 2.).sqrt() * 2. / self.multiplier;
                vec![u, -u, ZERO]
            }
            2 => {
                let l_n1 = self.multiplier.inv();
                let l_n2 = l_n1 * l_n1;
                let l_n4 = l_n2 * l_n2;
                solve_cubic(
                    l_n4 * horner!(l_n1, -64., 128., 256.),
                    16. * l_n2 - 32. * l_n4,
                    -8. * l_n1 + 4. * l_n2,
                )
                .into_iter()
                .flat_map(|x| {
                    let u = x.sqrt();
                    [u, -u].into_iter()
                })
                .collect()
            }
            3 => {
                let l = self.multiplier;
                let l2 = l.powi(2);
                let l4 = l2.powi(2);
                let l8 = l4.powi(2);
                let l12 = l8 * l4;
                let l16 = l8.powi(2);
                let l20 = l12 * l8;
                let l24 = l12.powi(2);
                let coeffs = [
                    horner!(
                        l,
                        4_294_967_296.,
                        2_147_483_648.,
                        1_073_741_824.,
                        -1_610_612_736.,
                        -268_435_456.,
                        402_653_184.,
                        -67_108_864.
                    ),
                    l2 * horner!(
                        l,
                        -536_870_912.,
                        -536_870_912.,
                        134_217_728.,
                        402_653_184.,
                        234_881_024.,
                        -134_217_728.,
                        -83_886_080.,
                        41_943_040.,
                        -4_194_304.
                    ),
                    l4 * horner!(
                        l,
                        67_108_864.,
                        100_663_296.,
                        -134_217_728.,
                        -184_549_376.,
                        -20_971_520.,
                        83_886_080.,
                        23_068_672.,
                        -23_068_672.,
                        3_145_728.
                    ),
                    l4 * l2
                        * horner!(
                            l,
                            -8_388_608.,
                            8_388_608.,
                            20_971_520.,
                            75_497_472.,
                            -26_214_400.,
                            -49_807_360.,
                            19_398_656.,
                            1_572_864.,
                            -786_432.
                        ),
                    l8 * horner!(
                        l,
                        1_048_576.,
                        1_572_864.,
                        -23_855_104.,
                        -2_097_152.,
                        22_937_600.,
                        -393_216.,
                        -5_636_096.,
                        917_504.,
                        65536.
                    ),
                    l8 * l2
                        * horner!(
                            l,
                            -131_072.,
                            2_097_152.,
                            3_211_264.,
                            -4_751_360.,
                            -3_932_160.,
                            3_637_248.,
                            -229_376.,
                            -131_072.
                        ),
                    l12 * horner!(
                        l, 16384., -720_896., 532_480., 1_499_136., -983_040., -172_032., 114_688.
                    ),
                    l12 * l2 * horner!(l, 53248., -43008., -223_232., 83968., 143_360., -57344.),
                    l16 * horner!(l, 2560., 11776., 13824., -46592., 17920.),
                    l16 * l2 * horner!(l, 64., -3456., 8064., -3584.),
                    l20 * horner!(l, 208., -736., 448.),
                    l20 * l2 * horner!(l, 28., -32.),
                    l24,
                ];
                solve_polynomial(coeffs)
                    .iter()
                    .flat_map(|x| {
                        let u = x.sqrt();
                        [u, -u].into_iter()
                    })
                    .collect()
            }
            _ => vec![],
        }
    }

    fn cycles_child(&self, c: &Self::Param, period: Period) -> Vec<Self::Var>
    {
        match period {
            1 => {
                let disc = (c.powi(2) + 4. * (4. - self.multiplier)).sqrt();
                vec![ZERO, -0.5 * (c + disc), 0.5 * (disc - c)]
            }
            2 => {
                let c2 = c.powi(2);
                let u = self.multiplier + 1.;
                let coeffs = [
                    u,
                    c * u,
                    c2 + self.multiplier * u + 1.,
                    2. * c * u,
                    c2 + self.multiplier + u,
                    2. * c,
                    ONE,
                ];
                solve_polynomial(coeffs)
            }
            _ => vec![],
        }
    }
}

impl EscapeEncoding for CubicPer1Lambda
{
    basic_escape_encoding!(3., 1.);
}

impl InfinityFirstReturnMap for CubicPer1Lambda
{
    #[inline]
    fn degree_real(&self) -> f64
    {
        3.0
    }
}

#[derive(Clone, Debug)]
pub struct CubicPer1LambdaParam
{
    point_grid: PointGrid,
    max_iter: Period,
    starting_crit: PlaneID,
}

impl CubicPer1LambdaParam
{
    const BASE_POINT: Cplx = Cplx::new(1e-4, 0.);
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -2.2,
        max_x: 4.2,
        min_y: -2.5,
        max_y: 2.5,
    };

    fn base_param(lambda: Cplx) -> Cplx
    {
        -Self::BASE_POINT.inv() - 0.75 * lambda * Self::BASE_POINT
    }
}
impl Default for CubicPer1LambdaParam
{
    fn default() -> Self
    {
        let bounds = Self::DEFAULT_BOUNDS;
        let point_grid = PointGrid::new_by_res_y(1024, bounds);
        Self {
            point_grid,
            max_iter: 1024,
            starting_crit: PlaneID::ZPlane,
        }
    }
}

impl DynamicalFamily for CubicPer1LambdaParam
{
    parameter_plane_impl!(CubicPer1Lambda);
    default_bounds!();

    #[inline]
    fn map(&self, z: Self::Var, a: &Self::Param) -> Self::Var
    {
        let c = Self::base_param(*a);
        z * horner_monic!(z, a, c)
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, a: &Self::Param) -> (Self::Var, Self::Deriv)
    {
        let c = Self::base_param(*a);
        let z2 = z.powi(2);
        let u = z2 + c * z + a;
        (z * u, u + z * (c + 2. * z))
    }

    #[inline]
    fn gradient(&self, z: Self::Var, a: &Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        let c = Self::base_param(*a);
        let z2 = z.powi(2);
        let u = z2 + c * z + a;
        (z * u, u + z * (c + 2. * z), z2)
    }

    #[inline]
    fn start_point(&self, _point: Cplx, c: &Self::Param) -> Self::Var
    {
        match self.starting_crit {
            PlaneID::ZPlane => 0.5 * c * Self::BASE_POINT,
            PlaneID::WPlane => TWO_THIRDS / Self::BASE_POINT,
        }
    }

    fn start_point_d(&self, _point: Cplx, c: &Self::Param)
        -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        match self.starting_crit {
            PlaneID::ZPlane => (0.5 * c * Self::BASE_POINT, ZERO, 0.5 * Self::BASE_POINT),
            PlaneID::WPlane => (TWO_THIRDS / Self::BASE_POINT, ZERO, ZERO),
        }
    }

    #[inline]
    fn param_map(&self, point: Cplx) -> Self::Param
    {
        point
    }

    fn name(&self) -> String
    {
        "Cubic Per(1, lambda) lambda-plane".to_owned()
    }

    fn default_selection(&self) -> Cplx
    {
        ZERO
    }

    fn default_julia_bounds(&self, point: Cplx, _param: &Self::Param) -> Bounds
    {
        let r = 4. / (point.norm() + 0.01);
        Bounds::centered_square(r)
    }

    fn cycle_active_plane(&mut self)
    {
        self.starting_crit = self.starting_crit.swap();
    }
}

impl MarkedPoints for CubicPer1LambdaParam
{
    #[inline]
    fn critical_points_child(&self, _param: &Self::Param) -> Vec<Self::Var>
    {
        vec![ZERO, ONE]
    }
}

impl EscapeEncoding for CubicPer1LambdaParam
{
    basic_escape_encoding!(3., 1.);
}

impl InfinityFirstReturnMap for CubicPer1LambdaParam
{
    #[inline]
    fn degree_real(&self) -> f64
    {
        3.0
    }
}

impl From<CubicPer1LambdaParam> for CubicPer1Lambda
{
    fn from(parent: CubicPer1LambdaParam) -> Self
    {
        let point = parent.default_selection();
        let param = parent.param_map(point);
        let point_grid = parent
            .point_grid()
            .new_with_same_height(parent.default_julia_bounds(point, &param));
        Self {
            point_grid,
            max_iter: parent.max_iter(),
            multiplier: param,
            starting_crit: parent.starting_crit,
        }
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
}

impl Default for CubicPer1_1
{
    fractal_impl!();
}

impl DynamicalFamily for CubicPer1_1
{
    parameter_plane_impl!();
    default_name!();
    default_bounds!();

    fn periodicity_tolerance(&self) -> Real
    {
        1e-6
    }
    fn min_iter(&self) -> Period
    {
        self.max_iter() / 3
    }

    #[inline]
    fn map(&self, z: Cplx, c: &Cplx) -> Cplx
    {
        z.powi(2) * (z + c) + z
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv)
    {
        (z.powi(2) * (z + c) + z, z * (3. * z + 2. * c) + 1.)
    }

    #[inline]
    fn gradient(&self, z: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        let z2 = z.powi(2);
        (z2 * (z + c) + z, z * (3. * z + 2. * c) + 1., z2)
    }

    #[inline]
    fn start_point(&self, _point: Cplx, c: &Cplx) -> Cplx
    {
        let u = (c.powi(2) - 3.).sqrt();
        -(c + u * c.re.signum()) / 3.
    }

    fn start_point_d(&self, _point: Cplx, c: &Self::Param)
        -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        let u = (c.powi(2) - 3.).sqrt();
        let s = c.re.signum();
        (
            -ONE_THIRD * (c + u * s),
            ZERO,
            -ONE_THIRD * (c * s / u - 1.),
        )
    }

    #[inline]
    fn default_julia_bounds(&self, _point: Cplx, _param: &Cplx) -> Bounds
    {
        Bounds::centered_square(2.2)
    }
}

impl MarkedPoints for CubicPer1_1
{
    #[inline]
    fn critical_points_child(&self, param: &Cplx) -> ComplexVec
    {
        let u = (param * param - 3.).sqrt();
        vec![-(param + u) / 3., (u - param) / 3.]
    }

    fn cycles_child(&self, c: &Self::Param, period: Period) -> Vec<Self::Var>
    {
        match period {
            1 => {
                vec![ZERO, -c]
            }
            2 => {
                let u = c.powi(2) + 3.;
                let coeffs = [TWO, 2. * c, u, 4. * c, u, 2. * c, ONE];
                solve_polynomial(coeffs)
            }
            3 => {
                let c2 = c.powi(2);
                let coeffs = [
                    Cplx::new(3., 0.),
                    6. * c,
                    horner!(c2, 9., 9.),
                    c * horner!(c2, 32., 10.),
                    horner!(c2, 24., 64., 8.),
                    c * horner!(c2, 108., 86., 4.),
                    horner!(c2, 54., 248., 78., 1.),
                    c * horner!(c2, 272., 352., 48.),
                    horner!(c2, 102., 642., 331., 20.),
                    c * horner!(c2, 520., 906., 212., 6.),
                    horner!(c2, 156., 1198., 831., 94., 1.),
                    c * horner!(c2, 768., 1610., 512., 26.),
                    horner!(c2, 192., 1664., 1375., 202., 3.),
                    c * horner!(c2, 882., 2050., 742., 42.),
                    horner!(c2, 189., 1736., 1520., 225., 3.),
                    c * horner!(c2, 784., 1846., 636., 30.),
                    horner!(c2, 147., 1326., 1065., 126., 1.),
                    c * horner!(c2, 522., 1098., 294., 8.),
                    horner!(c2, 87., 687., 420., 28.),
                    c * horner!(c2, 240., 378., 56.),
                    horner!(c2, 36., 210., 70.),
                    c * horner!(c2, 66., 56.),
                    horner!(c2, 9., 28.),
                    8. * c,
                    ONE,
                ];
                solve_polynomial(coeffs)
            }
            _ => vec![],
        }
    }
}

impl InfinityFirstReturnMap for CubicPer1_1
{
    #[inline]
    fn degree_real(&self) -> f64
    {
        3.0
    }
}

impl EscapeEncoding for CubicPer1_1
{
    fn encode_escaping_point(
        &self,
        iters: Period,
        z: Cplx,
        _base_param: &Cplx,
    ) -> PointInfo<Self::Deriv>
    {
        if z.is_nan() {
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
}

impl ExternalRays for CubicPer1_1 {}
impl ExternalRays for CubicPer1Lambda {}
impl ExternalRays for CubicPer1LambdaParam {}
impl ExternalRays for CubicPer1LambdaModuli {}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CubicPer1_0
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl CubicPer1_0
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -2.5,
        max_x: 2.5,
        min_y: -2.5,
        max_y: 2.5,
    };
}
impl Default for CubicPer1_0
{
    fractal_impl!();
}

impl DynamicalFamily for CubicPer1_0
{
    parameter_plane_impl!();
    default_name!();
    default_bounds!();

    #[allow(clippy::suspicious_operation_groupings)]
    fn map(&self, z: Self::Var, c: &Self::Param) -> Self::Var
    {
        z.powi(2) * (z + c)
    }

    #[inline]
    fn start_point(&self, _point: Cplx, c: &Self::Param) -> Self::Var
    {
        -TWO_THIRDS * c
    }

    #[inline]
    fn start_point_d(&self, _point: Cplx, c: &Self::Param)
        -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        (-TWO_THIRDS * c, ZERO, (-TWO_THIRDS).into())
    }

    fn map_and_multiplier(&self, z: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv)
    {
        (z.powi(2) * (z + c), z * (2. * c + 3. * z))
    }

    fn gradient(&self, z: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        let z2 = z.powi(2);
        (z2 * (z + c), z * (2. * c + 3. * z), z2)
    }

    fn default_julia_bounds(&self, _point: Cplx, c: &Self::Param) -> Bounds
    {
        if c.is_nan() {
            Bounds::centered_square(2.5)
        } else {
            Bounds::square(2.5, -ONE_THIRD * c)
        }
    }
}

impl MarkedPoints for CubicPer1_0
{
    fn critical_points_child(&self, c: &Self::Param) -> Vec<Self::Var>
    {
        vec![ZERO, -TWO_THIRDS * c]
    }

    fn cycles_child(&self, c: &Self::Param, period: Period) -> Vec<Self::Var>
    {
        match period {
            1 => {
                let [r1, r2] = solve_quadratic(-ONE, *c);
                vec![ZERO, r1, r2]
            }
            2 => {
                let u = c.powi(2) + 1.;
                let coeffs = [ONE, *c, u, 2. * c, u, 2. * c, ONE];
                solve_polynomial(coeffs)
            }
            3 => {
                let c2 = c.powi(2);
                let coeffs = [
                    ONE,
                    *c,
                    horner_monic!(c2, 1.),
                    c * horner_monic!(c2, 2.),
                    horner_monic!(c2, 1., 3.),
                    c * horner_monic!(c2, 3., 4.),
                    horner_monic!(c2, 1., 6., 5.),
                    c * horner!(c2, 4., 10., 6.),
                    horner!(c2, 1., 10., 15., 3.),
                    c * horner_monic!(c2, 5., 20., 15.),
                    horner_monic!(c2, 1., 15., 31., 8.),
                    c * horner!(c2, 6., 34., 26., 8.),
                    horner!(c2, 1., 21., 45., 28., 3.),
                    c * horner!(c2, 7., 45., 56., 21.),
                    horner!(c2, 1., 26., 70., 64., 3.),
                    c * horner!(c2, 8., 56., 111., 22.),
                    horner_monic!(c2, 1., 28., 120., 70.),
                    c * horner!(c2, 8., 83., 126., 8.),
                    horner!(c2, 1., 36., 140., 28.),
                    c * horner!(c2, 9., 98., 56.),
                    horner!(c2, 1., 42., 70.),
                    c * horner!(c2, 10., 56.),
                    horner!(c2, 1., 28.),
                    c * 8.,
                    ONE,
                ];
                solve_polynomial(coeffs)
            }
            4 => {
                let c2 = c.powi(2);
                let c3 = c2 * c;
                let coeffs = [
                    ONE,
                    ZERO,
                    ZERO,
                    c3,
                    c2 * 2.,
                    c3 + c,
                    c3.powi(2) + 3. * c2,
                    c * horner!(c2, 3., 0., 4.),
                    horner!(c2, 1., 0., 6., 2.),
                    c3 * horner_monic!(c2, 4., 10., 0.),
                    c2 * horner!(c2, 1., 20., 1., 6.),
                    c3 * horner!(c2, 20., 6., 15., 3.),
                    c2 * horner!(c2, 10., 15., 20., 21., 0., 1.),
                    c * horner!(c2, 2., 20., 15., 63., 3., 8.),
                    c2 * horner!(c2, 15., 6., 105., 24., 28., 4.),
                    c * horner!(c2, 6., 1., 105., 84., 57., 36.),
                    horner!(c2, 1., 0., 63., 168., 79., 144., 6., 2.),
                    c3 * horner!(c2, 21., 210., 92., 336., 60., 17., 1.),
                    c2 * horner!(c2, 3., 168., 112., 504., 270., 68., 19.),
                    c3 * horner!(c2, 84., 134., 504., 720., 184., 131., 4.),
                    c2 * horner!(c2, 24., 127., 336., 1260., 416., 490., 60., 1., 1.),
                    c * horner!(c2, 3., 84., 144., 1512., 842., 1158., 390., 16., 13.),
                    c2 * horner!(c2, 36., 36., 1260., 1432., 1872., 1480., 131., 83., 5.),
                    c * horner!(c2, 9., 4., 720., 1892., 2194., 3690., 680., 345., 70.),
                    horner!(c2, 1., 0., 270., 1858., 2001., 6408., 2410., 1048., 465., 10., 3.),
                    c3 * horner!(c2, 60., 1321., 1581., 7980., 6082., 2501., 1941., 150., 39.),
                    c2 * horner!(
                        c2, 6., 660., 1195., 7200., 11232., 4966., 5680., 1060., 245., 18.
                    ),
                    c3 * horner!(c2, 220., 847., 4680., 15432., 8536., 12315., 4680., 1030., 252.),
                    c2 * horner!(
                        c2, 44., 500., 2140., 15885., 12815., 20420., 14430., 3416., 1649., 45., 3.
                    ),
                    c * horner!(
                        c2, 4., 220., 654., 12220., 16456., 26445., 32890., 9751., 6715., 675., 43.
                    ),
                    c2 * horner!(
                        c2, 66., 120., 6931., 17479., 27258., 57200., 24298., 19167., 4741., 347.,
                        21.
                    ),
                    c * horner!(
                        c2, 12., 10., 2816., 14873., 22935., 77220., 51440., 41167., 20703., 2143.,
                        322.
                    ),
                    horner!(
                        c2, 1., 0., 776., 9858., 16390., 81510., 89870., 70349., 62943., 10587.,
                        2355., 63., 1.
                    ),
                    c3 * horner!(
                        c2, 130., 4945., 10455., 67210., 127_347., 100_893., 141_411., 40907.,
                        11055., 1029., 16.
                    ),
                    c2 * horner!(
                        c2, 10., 1808., 6120., 42900., 145_068., 127_595., 243_300., 122_311.,
                        37985., 7914., 225., 8.
                    ),
                    c3 * horner!(
                        c2, 454., 3193., 20800., 132_014., 146_575., 328_173., 284_479., 103_458.,
                        38124., 2380., 136.
                    ),
                    c2 * horner!(
                        c2, 70., 1380., 7410., 95200., 152_295., 353_184., 520_221., 235_026.,
                        129_180., 16698., 1193., 28.
                    ),
                    c * horner!(
                        c2, 5., 455., 1830., 53705., 138_827., 308_529., 755_469., 457_030.,
                        328_194., 80585., 7365., 504.
                    ),
                    c2 * horner!(
                        c2, 105., 280., 23206., 107_029., 223_938., 877_305., 764_400., 652_128.,
                        282_222., 35735., 4347., 56.
                    ),
                    c * horner!(
                        c2, 15., 20., 7420., 67605., 140_075., 817_193., 1_090_830., 1_047_192.,
                        747_405., 140_539., 24066., 1064.
                    ),
                    horner!(
                        c2, 1., 0., 1655., 34039., 79248., 609_817., 1_312_168., 1_399_482.,
                        1_540_711., 448_868., 96852., 9597., 70.
                    ),
                    c3 * horner!(
                        c2, 230., 13271., 42009., 362_313., 1_315_236., 1_600_950., 2_522_340.,
                        1_162_324., 304_542., 54691., 1400.
                    ),
                    c2 * horner!(
                        c2, 15., 3855., 20604., 169_277., 1_087_320., 1_606_644., 3_325_694.,
                        2_443_580., 784_329., 221_186., 13303., 56.
                    ),
                    c3 * horner!(
                        c2, 785., 8823., 60901., 733_910., 1_435_200., 3_567_500., 4_185_740.,
                        1_705_032., 676_438., 79864., 1176.
                    ),
                    c2 * horner!(
                        c2, 100., 3075., 16300., 399_630., 1_139_892., 3_140_020., 5_864_014.,
                        3_178_728., 1_629_117., 339_801., 11760., 28.
                    ),
                    c * horner!(
                        c2, 6., 816., 3060., 172_746., 792_192., 2_289_678., 6_735_638.,
                        5_105_576., 3_181_227., 1_089_480., 74480., 616.
                    ),
                    c2 * horner!(
                        c2, 153., 360., 57907., 470_160., 1_403_912., 6_345_794., 7_045_038.,
                        5_155_080., 2_732_485., 335_160., 6468., 8.
                    ),
                    c * horner!(
                        c2, 18., 20., 14520., 232_008., 741_862., 4_892_090., 8_298_108.,
                        7_072_408., 5_493_432., 1_139_544., 43120., 184.
                    ),
                    horner_monic!(
                        c2, 1., 0., 2565., 92607., 349_623., 3_068_500., 8_279_544., 8_361_178.,
                        9_001_041., 3_038_784., 204_820., 2024.
                    ),
                    c3 * horner!(
                        c2,
                        285.,
                        28989.,
                        151_300.,
                        1_550_468.,
                        6_941_508.,
                        8_641_542.,
                        12_160_304.,
                        6_511_680.,
                        737_352.,
                        14168.,
                        24.
                    ),
                    c2 * horner!(
                        c2,
                        15.,
                        6834.,
                        59993.,
                        621_316.,
                        4_847_346.,
                        7_877_324.,
                        13_659_670.,
                        11_395_440.,
                        2_089_164.,
                        70840.,
                        276.
                    ),
                    c3 * horner!(
                        c2,
                        1140.,
                        20896.,
                        192_788.,
                        2_790_312.,
                        6_339_844.,
                        12_842_480.,
                        16_460_080.,
                        4_775_232.,
                        269_192.,
                        2024.
                    ),
                    c2 * horner!(
                        c2,
                        120.,
                        6002.,
                        44625.,
                        1_306_620.,
                        4_472_258.,
                        10_169_978.,
                        19_752_096.,
                        8_953_560.,
                        807_576.,
                        10626.
                    ),
                    c * horner!(
                        c2,
                        6.,
                        1329.,
                        7245.,
                        488_880.,
                        2_726_766.,
                        6_837_264.,
                        19_752_096.,
                        13_927_760.,
                        1_961_256.,
                        42504.
                    ),
                    c2 * horner!(
                        c2,
                        210.,
                        735.,
                        142_471.,
                        1_410_864.,
                        3_947_706.,
                        16_460_080.,
                        18_106_088.,
                        3_922_512.,
                        134_596.
                    ),
                    c * horner!(
                        c2,
                        21.,
                        35.,
                        31122.,
                        606_480.,
                        1_989_680.,
                        11_395_440.,
                        19_752_096.,
                        6_537_520.,
                        346_104.
                    ),
                    horner!(
                        c2,
                        1.,
                        0.,
                        4788.,
                        211_337.,
                        891_480.,
                        6_511_680.,
                        18_106_088.,
                        9_152_528.,
                        735_471.
                    ),
                    c3 * horner!(
                        c2,
                        462.,
                        57911.,
                        358_872.,
                        3_038_784.,
                        13_927_760.,
                        10_816_624.,
                        1_307_504.
                    ),
                    c2 * horner!(
                        c2,
                        21.,
                        11970.,
                        128_611.,
                        1_139_544.,
                        8_953_560.,
                        10_816_624.,
                        1_961_256.
                    ),
                    c3 * horner!(c2, 1750., 39704., 335_160., 4_775_232., 9_152_528., 2_496_144.),
                    c2 * horner!(c2, 161., 10045., 74480., 2_089_164., 6_537_520., 2_704_156.),
                    c * horner!(c2, 7., 1960., 11760., 737_352., 3_922_512., 2_496_144.),
                    c2 * horner!(c2, 273., 1176., 204_820., 1_961_256., 1_961_256.),
                    c * horner!(c2, 24., 56., 43120., 807_576., 1_307_504.),
                    horner!(c2, 1., 0., 6468., 269_192., 735_471.),
                    c3 * horner!(c2, 616., 70840., 346_104.),
                    c2 * horner!(c2, 28., 14168., 134_596.),
                    c3 * horner!(c2, 2024., 42504.),
                    c2 * horner!(c2, 184., 10626.),
                    c * horner!(c2, 8., 2024.),
                    c2 * horner!(c2, 276.),
                    c * horner!(c2, 24.),
                    ONE,
                ];
                for (i, x) in coeffs.iter().enumerate() {
                    println!("{}: {}", i, x.re);
                }
                let res = solve_polynomial(coeffs);
                dbg!(&res);
                res
            }
            _ => vec![],
        }
    }
}

impl InfinityFirstReturnMap for CubicPer1_0
{
    #[inline]
    fn degree_real(&self) -> f64
    {
        3.0
    }

    fn angle_map_large_param(&self, angle: RationalAngle) -> RationalAngle
    {
        3 * angle
    }
}

impl EscapeEncoding for CubicPer1_0 {}
impl ExternalRays for CubicPer1_0 {}

impl HasDynamicalCovers for CubicPer1_0
{
    fn marked_cycle_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> (Cplx, Cplx);
        let bounds: Bounds;

        match period {
            1 => {
                param_map = |t| {
                    let u = t.inv();
                    (u - t, -u.powi(2) - 1.)
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
                    let t2 = t.powi(2);
                    let t4 = t2.powi(2);
                    let u = (t2 + 1.).inv();
                    ((t2 + 2.) * t * u, (2. + t2 + t4) * u.powi(2))
                };
                bounds = Bounds {
                    min_x: -1.5,
                    max_x: 1.5,
                    min_y: -3.2,
                    max_y: 3.2,
                };
            }
            _ => {
                param_map = |t| (t, ONE);
                bounds = self.point_grid.bounds.clone();
            }
        };
        let grid = self.point_grid.new_with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }

    #[allow(clippy::suspicious_operation_groupings)]
    fn dynatomic_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> (Cplx, Cplx);
        let bounds: Bounds;

        match period {
            1 => {
                param_map = |t| {
                    let u = t.inv();
                    (u - t, -u.powi(2) - 1.)
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
                    let t2 = t.powi(2);
                    let u = t * horner!(t2, 2.25, -2., 4.);
                    let du = horner!(t2, 2.25, -6., 20.);
                    let v = horner!(t2, A0, A2, A4, I);
                    let dv = t * horner!(t2, B2, B4, I6);
                    (u / v, (du * v - u * dv) / (dv * dv))
                };
                bounds = Bounds {
                    min_x: -3.2,
                    max_x: 3.2,
                    min_y: -2.5,
                    max_y: 2.5,
                };
            }
            _ => {
                param_map = |t| (t, ONE);
                bounds = self.point_grid.bounds.clone();
            }
        };
        let grid = self.point_grid.new_with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }
    fn misiurewicz_curve(self, preperiod: Period, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> (Cplx, Cplx);
        let bounds: Bounds;

        match (preperiod, period) {
            (1, 1) => {
                param_map = |t| {
                    let t2 = t.powi(2);
                    let t4 = t2.powi(2);
                    (
                        -(t4 + t2 + 1.) / (t * t2 + t),
                        -horner_monic!(t2, -1., -2., 2.) / (t2 * horner_monic!(t2, 1., 2.)),
                    )
                };
                bounds = Bounds {
                    min_x: -2.0,
                    max_x: 2.0,
                    min_y: -2.8,
                    max_y: 2.8,
                };
            }
            (_, _) => {
                param_map = |t| (t, ONE);
                bounds = self.point_grid.bounds.clone();
            }
        };
        let grid = self.point_grid.new_with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }
}

impl HasDynamicalCovers for CubicPer1_1
{
    fn marked_cycle_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> (Cplx, Cplx);
        let bounds: Bounds;

        match period {
            2 => {
                param_map = |t| {
                    let t2 = t.powi(2);
                    let u = t2 + 1.;
                    ((t2 + 3.) * t / (t2 + 1.), (t2.powi(2) + 3.) / (u.powi(2)))
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
        let grid = self.point_grid.new_with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }

    #[allow(clippy::suspicious_operation_groupings)]
    fn dynatomic_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> (Cplx, Cplx);
        let bounds: Bounds;

        match period {
            2 => {
                param_map = |t| {
                    let t2 = t.powi(2);
                    let t4 = t2.powi(2);
                    let u = -1. + t2 * (3. - t * (8. + t * (3. - t2)));
                    let du = 6. * t * (1. - 4. * t + (t2 - 2.) * t2);
                    let v = t * I2 * (t4 - 1.);
                    let dv = I10 * t4 - I2;
                    (u / v, (du * v - u * dv) / v.powi(2))
                };
                bounds = Bounds {
                    min_x: -4.8,
                    max_x: 5.5,
                    min_y: -5.0,
                    max_y: 5.0,
                };
            }
            _ => {
                param_map = |t| (t, ONE);
                bounds = self.point_grid.bounds.clone();
            }
        };
        let grid = self.point_grid.new_with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }
    fn misiurewicz_curve(self, preperiod: Period, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> (Cplx, Cplx);
        let bounds: Bounds;

        match (preperiod, period) {
            (1, 1) => {
                param_map = |t| {
                    let u = t.inv();
                    (t + u, 1. - u.powi(2))
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
        };
        let grid = self.point_grid.new_with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CubicPer1LambdaModuli
{
    point_grid: PointGrid,
    max_iter: Period,
    multiplier: Cplx,
    starting_crit: PlaneID,
}

impl CubicPer1LambdaModuli
{
    const DEFAULT_BOUNDS: Bounds = Bounds::centered_square(2.5);
}

impl Default for CubicPer1LambdaModuli
{
    fn default() -> Self
    {
        let point_grid = PointGrid::new_by_res_y(1024, Self::DEFAULT_BOUNDS);
        Self {
            point_grid,
            max_iter: 1024,
            multiplier: ZERO,
            starting_crit: PlaneID::ZPlane,
        }
    }
}

impl DynamicalFamily for CubicPer1LambdaModuli
{
    type Var = Cplx;
    type Param = CplxPair;
    type MetaParam = Cplx;
    type Deriv = Cplx;
    type Child = JuliaSet<Self>;

    basic_plane_impl!();
    default_name!();
    default_bounds!();

    #[inline]
    fn map(&self, z: Self::Var, CplxPair { a, b }: &Self::Param) -> Self::Var
    {
        z * (a * z.powi(2) + b) + 1.
    }

    #[inline]
    fn map_and_multiplier(
        &self,
        z: Self::Var,
        CplxPair { a, b }: &Self::Param,
    ) -> (Self::Var, Self::Deriv)
    {
        let az2 = a * z.powi(2);
        (z * (az2 + b) + 1., 3. * az2 + b)
    }

    fn gradient(
        &self,
        z: Self::Var,
        CplxPair { a, b }: &Self::Param,
    ) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        // Impossible to define correctly with current typing
        let az2 = a * z.powi(2);
        (z * (az2 + b) + 1., 3. * az2 + b, z.powi(3) + z)
    }

    fn param_map(&self, t: Cplx) -> Self::Param
    {
        const FOUR_27: Cplx = Cplx::new(0.148_148_148_148_148, 0.);

        let l = self.multiplier;
        let u = t + 0.5 * (l - 3.);
        let a = FOUR_27 * (l - t) * u.powi(2);
        CplxPair { a, b: t }
    }

    #[inline]
    fn start_point(&self, point: Cplx, CplxPair { a, b }: &Self::Param) -> Self::Var
    {
        use PlaneID::*;

        let u = b / (3. * a);
        let crit = (-u).sqrt();
        let sign = u.im.signum() * point.im.signum();
        match self.starting_crit {
            ZPlane => sign * crit,
            WPlane => -sign * crit,
        }
    }

    fn cycle_active_plane(&mut self)
    {
        self.starting_crit = self.starting_crit.swap();
    }

    fn get_meta_params(&self) -> Self::MetaParam
    {
        self.multiplier
    }

    fn get_param(&self) -> Cplx
    {
        self.multiplier
    }

    fn set_meta_param(&mut self, value: Self::MetaParam)
    {
        self.multiplier = value;
    }

    fn set_param(&mut self, value: <Self::MetaParam as ParamList>::Param)
    {
        self.multiplier = value;
    }

    fn default_julia_bounds(&self, _point: Cplx, CplxPair { a, b }: &Self::Param) -> Bounds
    {
        let radius = (2. * (b / a).sqrt().norm()).max(6.0);
        Bounds::centered_square(radius)
    }
}

impl MarkedPoints for CubicPer1LambdaModuli
{
    fn critical_points_child(&self, CplxPair { a, b }: &Self::Param) -> Vec<Self::Var>
    {
        let disc = (-b / (3. * a)).sqrt();
        vec![disc, -disc]
    }

    fn cycles_child(&self, CplxPair { a, b }: &Self::Param, period: Period) -> Vec<Self::Var>
    {
        match period {
            1 => solve_cubic(a.inv(), (b - 1.) / a, ZERO).to_vec(),
            2 => {
                let a2 = a.powi(2);
                let coeffs = [
                    a + b + 1.,
                    a * (2. * b + 1.),
                    a * horner_monic!(b, 1., 1.),
                    2. * a2,
                    a2 * (2. * b + 1.),
                    ZERO,
                    a2 * a,
                ];
                solve_polynomial(coeffs)
            }
            _ => vec![],
        }
    }
}

impl InfinityFirstReturnMap for CubicPer1LambdaModuli
{
    #[inline]
    fn degree_real(&self) -> f64
    {
        3.0
    }
}

impl EscapeEncoding for CubicPer1LambdaModuli
{
    fn encode_escaping_point(
        &self,
        iters: Period,
        z: Self::Var,
        CplxPair { a, b: _ }: &Self::Param,
    ) -> PointInfo<Self::Deriv>
    {
        if z.is_nan() {
            return PointInfo::Escaping {
                potential: f64::from(iters) - 1.,
            };
        }

        let u = self.escape_radius().log2();
        let v = z.norm_sqr().log2();
        let delta = 0.5 * a.norm().log2();
        let residual = ((v + delta) / (u + delta)).log(3.);
        let potential = IterCount::from(iters) - IterCount::from(residual);
        PointInfo::Escaping { potential }
    }
}

impl From<CubicPer1LambdaParam> for CubicPer1LambdaModuli
{
    fn from(parent: CubicPer1LambdaParam) -> Self
    {
        let point = parent.default_selection();
        let param = parent.param_map(point);
        let point_grid = parent
            .point_grid()
            .new_with_same_height(parent.default_julia_bounds(point, &param));
        Self {
            point_grid,
            max_iter: parent.max_iter(),
            multiplier: param,
            starting_crit: PlaneID::ZPlane,
        }
    }
}
