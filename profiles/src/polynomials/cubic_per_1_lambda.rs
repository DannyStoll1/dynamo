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

impl ParameterPlane for CubicPer1Lambda
{
    parameter_plane_impl!(Cplx, Cplx, Cplx, Cplx);

    #[inline]
    fn map(&self, z: Self::Var, c: Self::Param) -> Self::Var
    {
        z * horner_monic!(z, self.multiplier, c)
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, c: Self::Param) -> (Self::Var, Self::Deriv)
    {
        let z2 = z * z;
        let u = z2 + c * z + self.multiplier;
        (z * u, u + z * (c + z + z))
    }

    #[inline]
    fn dynamical_derivative(&self, z: Self::Var, c: Self::Param) -> Self::Deriv
    {
        let z2 = z * z;
        let u = z2 + c * z + self.multiplier;
        u + z * (c + z + z)
    }

    #[inline]
    fn parameter_derivative(&self, z: Self::Var, _c: Self::Param) -> Self::Deriv
    {
        z
    }

    #[inline]
    fn start_point(&self, t: Cplx, _c: Self::Param) -> Self::Var
    {
        match self.starting_crit
        {
            PlaneID::ZPlane => 0.5 * self.multiplier * t,
            PlaneID::WPlane => TWO_THIRDS / t,
        }
    }

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

    fn critical_points_child(&self, c: Self::Param) -> Vec<Self::Var>
    {
        let disc = (c * c - 3. * self.multiplier).sqrt();
        vec![-ONE_THIRD * (c + disc), -ONE_THIRD * (c - disc)]
    }

    fn cycles(&self, period: Period) -> Vec<Self::Var>
    {
        match period
        {
            1 =>
            {
                let u = (self.multiplier - 2.).sqrt() * 2. / self.multiplier;
                vec![u, -u, ZERO]
            }
            2 =>
            {
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
            3 =>
            {
                let l = self.multiplier;
                let l2 = l * l;
                let l4 = l2 * l2;
                let l8 = l4 * l4;
                let l12 = l8 * l4;
                let l16 = l8 * l8;
                let l20 = l12 * l8;
                let l24 = l16 * l8;
                let coeffs = [
                    horner!(
                        l,
                        4294967296.,
                        2147483648.,
                        1073741824.,
                        -1610612736.,
                        -268435456.,
                        402653184.,
                        -67108864.
                    ),
                    l2 * horner!(
                        l,
                        -536870912.,
                        -536870912.,
                        134217728.,
                        402653184.,
                        234881024.,
                        -134217728.,
                        -83886080.,
                        41943040.,
                        -4194304.
                    ),
                    l4 * horner!(
                        l,
                        67108864.,
                        100663296.,
                        -134217728.,
                        -184549376.,
                        -20971520.,
                        83886080.,
                        23068672.,
                        -23068672.,
                        3145728.
                    ),
                    l4 * l2
                        * horner!(
                            l, -8388608., 8388608., 20971520., 75497472., -26214400., -49807360.,
                            19398656., 1572864., -786432.
                        ),
                    l8 * horner!(
                        l, 1048576., 1572864., -23855104., -2097152., 22937600., -393216.,
                        -5636096., 917504., 65536.
                    ),
                    l8 * l2
                        * horner!(
                            l, -131072., 2097152., 3211264., -4751360., -3932160., 3637248.,
                            -229376., -131072.
                        ),
                    l12 * horner!(
                        l, 16384., -720896., 532480., 1499136., -983040., -172032., 114688.
                    ),
                    l12 * l2 * horner!(l, 53248., -43008., -223232., 83968., 143360., -57344.),
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

    fn cycles_child(&self, c: Self::Param, period: Period) -> Vec<Self::Var>
    {
        match period
        {
            1 =>
            {
                let disc = (c * c + 4. * (4. - self.multiplier)).sqrt();
                vec![ZERO, -0.5 * (c + disc), 0.5 * (disc - c)]
            }
            2 =>
            {
                let c2 = c * c;
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

impl ParameterPlane for CubicPer1LambdaParam
{
    parameter_plane_impl!(CubicPer1Lambda);
    default_bounds!();

    #[inline]
    fn map(&self, z: Self::Var, a: Self::Param) -> Self::Var
    {
        let c = Self::base_param(a);
        z * horner_monic!(z, a, c)
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, a: Self::Param) -> (Self::Var, Self::Deriv)
    {
        let c = Self::base_param(a);
        let z2 = z * z;
        let u = z2 + c * z + a;
        (z * u, u + z * (c + z + z))
    }

    #[inline]
    fn dynamical_derivative(&self, z: Self::Var, a: Self::Param) -> Self::Deriv
    {
        let c = Self::base_param(a);
        let z2 = z * z;
        let u = z2 + c * z + a;
        u + z * (c + z + z)
    }

    #[inline]
    fn parameter_derivative(&self, z: Self::Var, _c: Self::Param) -> Self::Deriv
    {
        z
    }

    #[inline]
    fn start_point(&self, _point: Cplx, c: Self::Param) -> Self::Var
    {
        match self.starting_crit
        {
            PlaneID::ZPlane => 0.5 * c * Self::BASE_POINT,
            PlaneID::WPlane => TWO_THIRDS / Self::BASE_POINT,
        }
    }

    #[inline]
    fn param_map(&self, point: Cplx) -> Self::Param
    {
        point
    }

    fn critical_points_child(&self, _param: Self::Param) -> Vec<Self::Var>
    {
        vec![ZERO, ONE]
    }

    fn name(&self) -> String
    {
        "Cubic Per(1, lambda) lambda-plane".to_owned()
    }

    fn default_selection(&self) -> Cplx
    {
        ZERO
    }

    fn default_julia_bounds(&self, point: Cplx, _param: Self::Param) -> Bounds
    {
        let r = 4. / (point.norm() + 0.01);
        Bounds::centered_square(r)
    }

    fn cycle_active_plane(&mut self)
    {
        self.starting_crit = self.starting_crit.swap();
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
            .new_with_same_height(parent.default_julia_bounds(point, param));
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

impl ParameterPlane for CubicPer1_1
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
    fn map(&self, z: Cplx, c: Cplx) -> Cplx
    {
        z * (z * (z + c) + 1.)
    }

    #[inline]
    fn start_point(&self, _point: Cplx, param: Cplx) -> Cplx
    {
        let u = (param * param - 3.).sqrt();
        -(param + u * param.re.signum()) / 3.
    }

    #[inline]
    fn dynamical_derivative(&self, z: Cplx, c: Cplx) -> Cplx
    {
        z * (2. * c + 3. * z) + 1.
    }

    #[inline]
    fn parameter_derivative(&self, z: Cplx, _c: Cplx) -> Cplx
    {
        z * z
    }

    #[inline]
    fn critical_points_child(&self, param: Cplx) -> ComplexVec
    {
        let u = (param * param - 3.).sqrt();
        vec![-(param + u) / 3., (u - param) / 3.]
    }

    fn cycles_child(&self, c: Self::Param, period: Period) -> Vec<Self::Var>
    {
        match period
        {
            1 =>
            {
                vec![ZERO, -c]
            }
            2 =>
            {
                let u = c * c + 3.;
                let coeffs = [TWO, 2. * c, u, 4. * c, u, 2. * c, ONE];
                solve_polynomial(coeffs)
            }
            3 =>
            {
                let c2 = c * c;
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

    #[inline]
    fn default_julia_bounds(&self, _point: Cplx, _param: Cplx) -> Bounds
    {
        Bounds::centered_square(2.2)
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
        _base_param: Cplx,
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

impl ParameterPlane for CubicPer1_0
{
    parameter_plane_impl!();
    default_name!();
    default_bounds!();

    #[allow(clippy::suspicious_operation_groupings)]
    fn map(&self, z: Self::Var, c: Self::Param) -> Self::Var
    {
        z * z * (z + c)
    }

    #[inline]
    fn start_point(&self, _point: Cplx, c: Self::Param) -> Self::Var
    {
        -TWO_THIRDS * c
    }

    #[inline]
    fn start_point_d(&self, _point: Cplx, c: Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        (-TWO_THIRDS * c, ZERO, (-TWO_THIRDS).into())
    }

    fn dynamical_derivative(&self, z: Self::Var, c: Self::Param) -> Self::Deriv
    {
        z * (c + c + 3. * z)
    }

    fn parameter_derivative(&self, z: Self::Var, _c: Self::Param) -> Self::Deriv
    {
        z * z
    }

    fn gradient(&self, z: Self::Var, c: Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        let z2 = z * z;
        (z2 * (z + c), z * (2. * c + 3. * z), z2)
    }

    fn critical_points_child(&self, c: Self::Param) -> Vec<Self::Var>
    {
        vec![ZERO, -TWO_THIRDS * c]
    }

    fn cycles_child(&self, c: Self::Param, period: Period) -> Vec<Self::Var>
    {
        match period
        {
            1 =>
            {
                let [r1, r2] = solve_quadratic(-ONE, c);
                vec![ZERO, r1, r2]
            }
            2 =>
            {
                let u = c * c + 1.;
                let coeffs = [ONE, c, u, c + c, u, c + c, ONE];
                solve_polynomial(coeffs)
            }
            3 =>
            {
                let c2 = c * c;
                let coeffs = [
                    ONE,
                    c,
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
            4 =>
            {
                let c2 = c * c;
                let c3 = c2 * c;
                let coeffs = [
                    ONE,
                    ZERO,
                    ZERO,
                    c3,
                    c2 * 2.,
                    c3 + c,
                    c3 * c3 + 3. * c2,
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
                        c2, 130., 4945., 10455., 67210., 127347., 100893., 141411., 40907., 11055.,
                        1029., 16.
                    ),
                    c2 * horner!(
                        c2, 10., 1808., 6120., 42900., 145068., 127595., 243300., 122311., 37985.,
                        7914., 225., 8.
                    ),
                    c3 * horner!(
                        c2, 454., 3193., 20800., 132014., 146575., 328173., 284479., 103458.,
                        38124., 2380., 136.
                    ),
                    c2 * horner!(
                        c2, 70., 1380., 7410., 95200., 152295., 353184., 520221., 235026., 129180.,
                        16698., 1193., 28.
                    ),
                    c * horner!(
                        c2, 5., 455., 1830., 53705., 138827., 308529., 755469., 457030., 328194.,
                        80585., 7365., 504.
                    ),
                    c2 * horner!(
                        c2, 105., 280., 23206., 107029., 223938., 877305., 764400., 652128.,
                        282222., 35735., 4347., 56.
                    ),
                    c * horner!(
                        c2, 15., 20., 7420., 67605., 140075., 817193., 1090830., 1047192., 747405.,
                        140539., 24066., 1064.
                    ),
                    horner!(
                        c2, 1., 0., 1655., 34039., 79248., 609817., 1312168., 1399482., 1540711.,
                        448868., 96852., 9597., 70.
                    ),
                    c3 * horner!(
                        c2, 230., 13271., 42009., 362313., 1315236., 1600950., 2522340., 1162324.,
                        304542., 54691., 1400.
                    ),
                    c2 * horner!(
                        c2, 15., 3855., 20604., 169277., 1087320., 1606644., 3325694., 2443580.,
                        784329., 221186., 13303., 56.
                    ),
                    c3 * horner!(
                        c2, 785., 8823., 60901., 733910., 1435200., 3567500., 4185740., 1705032.,
                        676438., 79864., 1176.
                    ),
                    c2 * horner!(
                        c2, 100., 3075., 16300., 399630., 1139892., 3140020., 5864014., 3178728.,
                        1629117., 339801., 11760., 28.
                    ),
                    c * horner!(
                        c2, 6., 816., 3060., 172746., 792192., 2289678., 6735638., 5105576.,
                        3181227., 1089480., 74480., 616.
                    ),
                    c2 * horner!(
                        c2, 153., 360., 57907., 470160., 1403912., 6345794., 7045038., 5155080.,
                        2732485., 335160., 6468., 8.
                    ),
                    c * horner!(
                        c2, 18., 20., 14520., 232008., 741862., 4892090., 8298108., 7072408.,
                        5493432., 1139544., 43120., 184.
                    ),
                    horner_monic!(
                        c2, 1., 0., 2565., 92607., 349623., 3068500., 8279544., 8361178., 9001041.,
                        3038784., 204820., 2024.
                    ),
                    c3 * horner!(
                        c2, 285., 28989., 151300., 1550468., 6941508., 8641542., 12160304.,
                        6511680., 737352., 14168., 24.
                    ),
                    c2 * horner!(
                        c2, 15., 6834., 59993., 621316., 4847346., 7877324., 13659670., 11395440.,
                        2089164., 70840., 276.
                    ),
                    c3 * horner!(
                        c2, 1140., 20896., 192788., 2790312., 6339844., 12842480., 16460080.,
                        4775232., 269192., 2024.
                    ),
                    c2 * horner!(
                        c2, 120., 6002., 44625., 1306620., 4472258., 10169978., 19752096.,
                        8953560., 807576., 10626.
                    ),
                    c * horner!(
                        c2, 6., 1329., 7245., 488880., 2726766., 6837264., 19752096., 13927760.,
                        1961256., 42504.
                    ),
                    c2 * horner!(
                        c2, 210., 735., 142471., 1410864., 3947706., 16460080., 18106088.,
                        3922512., 134596.
                    ),
                    c * horner!(
                        c2, 21., 35., 31122., 606480., 1989680., 11395440., 19752096., 6537520.,
                        346104.
                    ),
                    horner!(
                        c2, 1., 0., 4788., 211337., 891480., 6511680., 18106088., 9152528., 735471.
                    ),
                    c3 * horner!(
                        c2, 462., 57911., 358872., 3038784., 13927760., 10816624., 1307504.
                    ),
                    c2 * horner!(c2, 21., 11970., 128611., 1139544., 8953560., 10816624., 1961256.),
                    c3 * horner!(c2, 1750., 39704., 335160., 4775232., 9152528., 2496144.),
                    c2 * horner!(c2, 161., 10045., 74480., 2089164., 6537520., 2704156.),
                    c * horner!(c2, 7., 1960., 11760., 737352., 3922512., 2496144.),
                    c2 * horner!(c2, 273., 1176., 204820., 1961256., 1961256.),
                    c * horner!(c2, 24., 56., 43120., 807576., 1307504.),
                    horner!(c2, 1., 0., 6468., 269192., 735471.),
                    c3 * horner!(c2, 616., 70840., 346104.),
                    c2 * horner!(c2, 28., 14168., 134596.),
                    c3 * horner!(c2, 2024., 42504.),
                    c2 * horner!(c2, 184., 10626.),
                    c * horner!(c2, 8., 2024.),
                    c2 * horner!(c2, 276.),
                    c * horner!(c2, 24.),
                    ONE,
                ];
                for (i, x) in coeffs.iter().enumerate()
                {
                    println!("{}: {}", i, x.re);
                }
                let res = solve_polynomial(coeffs);
                dbg!(&res);
                res
            }
            _ => vec![],
        }
    }

    fn default_julia_bounds(&self, _point: Cplx, c: Self::Param) -> Bounds
    {
        if c.is_nan()
        {
            Bounds::centered_square(2.5)
        }
        else
        {
            Bounds::square(2.5, -ONE_THIRD * c)
        }
    }
}

impl InfinityFirstReturnMap for CubicPer1_0 {
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

        match period
        {
            1 =>
            {
                param_map = |t| {
                    let u = t.inv();
                    (u - t, -u * u - 1.)
                };
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
                    let t2 = t * t;
                    let u = (t2 + 1.).inv();
                    ((t2 + 2.) * t * u, (2. + t2 + t2 * t2) * u * u)
                };
                bounds = Bounds {
                    min_x: -1.5,
                    max_x: 1.5,
                    min_y: -3.2,
                    max_y: 3.2,
                };
            }
            _ =>
            {
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

        match period
        {
            1 =>
            {
                param_map = |t| {
                    let u = t.inv();
                    (u - t, -u * u - 1.)
                };
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
                    let t2 = t * t;
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
            _ =>
            {
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

        match (preperiod, period)
        {
            (1, 1) =>
            {
                param_map = |t| {
                    let t2 = t * t;
                    (
                        -(t2 * t2 + t2 + 1.) / (t * t2 + t),
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
            (_, _) =>
            {
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

        match period
        {
            2 =>
            {
                param_map = |t| {
                    let t2 = t * t;
                    let u = t2 + 1.;
                    ((t2 + 3.) * t / (t2 + 1.), (t2 * t2 + 3.) / (u * u))
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
                param_map = |t| (t, ONE);
                bounds = self.point_grid.bounds.clone();
            }
        };
        let grid = self.point_grid.new_with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }

    fn dynatomic_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> (Cplx, Cplx);
        let bounds: Bounds;

        match period
        {
            2 =>
            {
                param_map = |t| {
                    let t2 = t * t;
                    let t4 = t2 * t2;
                    let u = -1. + t2 * (3. - t * (8. + t * (3. - t2)));
                    let du = 6. * t * (1. - 4. * t + (t2 - 2.) * t2);
                    let v = t * I2 * (t4 - 1.);
                    let dv = I10 * t4 - I2;
                    (u / v, (du * v - u * dv) / (v * v))
                };
                bounds = Bounds {
                    min_x: -4.8,
                    max_x: 5.5,
                    min_y: -5.0,
                    max_y: 5.0,
                };
            }
            _ =>
            {
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

        match (preperiod, period)
        {
            (1, 1) =>
            {
                param_map = |t| {
                    let u = t.inv();
                    (t + u, 1. - u * u)
                };
                bounds = Bounds {
                    min_x: -2.5,
                    max_x: 2.5,
                    min_y: -2.5,
                    max_y: 2.5,
                };
            }
            (_, _) =>
            {
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

impl ParameterPlane for CubicPer1LambdaModuli
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
    fn map(&self, z: Self::Var, CplxPair { a, b }: Self::Param) -> Self::Var
    {
        z * (a * z * z + b) + 1.
    }

    #[inline]
    fn map_and_multiplier(
        &self,
        z: Self::Var,
        CplxPair { a, b }: Self::Param,
    ) -> (Self::Var, Self::Deriv)
    {
        let az2 = a * z * z;
        (z * (az2 + b) + 1., 3. * az2 + b)
    }

    fn param_map(&self, t: Cplx) -> Self::Param
    {
        let l = self.multiplier;
        const FOUR_27: Cplx = Cplx::new(0.148148148148148, 0.);

        let u = t + 0.5 * (l - 3.);
        let a = FOUR_27 * (l - t) * u * u;
        CplxPair { a, b: t }
    }

    #[inline]
    fn start_point(&self, point: Cplx, CplxPair { a, b }: Self::Param) -> Self::Var
    {
        let u = b / (3. * a);
        let crit = (-u).sqrt();
        let sign = u.im.signum() * point.im.signum();
        use PlaneID::*;
        match self.starting_crit
        {
            ZPlane => sign * crit,
            WPlane => -sign * crit,
        }
    }

    fn dynamical_derivative(&self, z: Self::Var, CplxPair { a, b }: Self::Param) -> Self::Deriv
    {
        let z2 = z * z;
        3. * a * z2 + b
    }

    fn parameter_derivative(&self, z: Self::Var, _c: Self::Param) -> Self::Deriv
    {
        z * z
    }

    fn critical_points_child(&self, CplxPair { a, b }: Self::Param) -> Vec<Self::Var>
    {
        let disc = (-b / (3. * a)).sqrt();
        vec![disc, -disc]
    }

    fn cycles_child(&self, CplxPair { a, b }: Self::Param, period: Period) -> Vec<Self::Var>
    {
        match period
        {
            1 => solve_cubic(a.inv(), (b - 1.) / a, ZERO).to_vec(),
            2 =>
            {
                let a2 = a * a;
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

    fn cycle_active_plane(&mut self)
    {
        self.starting_crit = self.starting_crit.swap()
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

    fn default_julia_bounds(&self, _point: Cplx, CplxPair { a, b }: Self::Param) -> Bounds
    {
        let radius = (2. * (b / a).sqrt().norm()).max(6.0);
        Bounds::centered_square(radius)
    }
}

impl InfinityFirstReturnMap for CubicPer1LambdaModuli {
    #[inline]
    fn degree_real(&self) -> f64
    {
        3.0
    }
}

impl EscapeEncoding for CubicPer1LambdaModuli {
    fn encode_escaping_point(
        &self,
        iters: Period,
        z: Self::Var,
        CplxPair { a, b: _ }: Self::Param,
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
            .new_with_same_height(parent.default_julia_bounds(point, param));
        Self {
            point_grid,
            max_iter: parent.max_iter(),
            multiplier: param,
            starting_crit: PlaneID::ZPlane,
        }
    }
}
