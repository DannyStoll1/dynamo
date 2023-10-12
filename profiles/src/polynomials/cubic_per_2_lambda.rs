use crate::macros::{horner, horner_monic, profile_imports};
use fractal_common::{math_utils::weierstrass_p, types::variables::PlaneID};
profile_imports!();

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CubicPer2Lambda
{
    point_grid: PointGrid,
    max_iter: Period,
    multiplier: Cplx,
    starting_crit: PlaneID,
}

impl CubicPer2Lambda
{
    const DEFAULT_BOUNDS: Bounds = Bounds::centered_square(2.5);
}

impl Default for CubicPer2Lambda
{
    fn default() -> Self
    {
        let point_grid = PointGrid::default().with_same_height(Self::DEFAULT_BOUNDS);
        Self {
            point_grid,
            max_iter: 1024,
            multiplier: ZERO,
            starting_crit: PlaneID::ZPlane,
        }
    }
}

impl ParameterPlane for CubicPer2Lambda
{
    parameter_plane_impl!(Cplx, CplxPair, Cplx, Cplx);
    basic_escape_encoding!(3.);

    #[inline]
    fn degree_real(&self) -> f64
    {
        3.0
    }

    #[inline]
    fn map(&self, z: Self::Var, c: Self::Param) -> Self::Var
    {
        horner!(z, c.b, -(1. + c.a), -c.b, c.a)
    }

    #[inline]
    fn map_and_multiplier(
        &self,
        z: Self::Var,
        CplxPair { a, b }: Self::Param,
    ) -> (Self::Var, Self::Deriv)
    {
        let x1 = -a - 1.;
        (horner!(z, b, x1, -b, a), horner!(z, x1, -(b + b), 3. * a))
    }

    #[inline]
    fn dynamical_derivative(&self, z: Self::Var, CplxPair { a, b }: Self::Param) -> Self::Deriv
    {
        horner!(z, -a - 1., -(b + b), 3. * a)
    }

    #[inline]
    fn parameter_derivative(&self, z: Self::Var, _c: Self::Param) -> Self::Deriv
    {
        z * (1. + z * z)
    }

    #[inline]
    fn start_point(&self, _m: Cplx, CplxPair { a, b }: Self::Param) -> Self::Var
    {
        let disc = (3. * a * (a + 1.) + b * b).sqrt();

        match self.starting_crit
        {
            PlaneID::ZPlane => (b + disc) / (3. * a),
            PlaneID::WPlane => (b - disc) / (3. * a),
        }
    }

    fn critical_points_child(&self, c: Self::Param) -> Vec<Self::Var>
    {
        let disc = (3. * c.a * (c.a + 1.) + c.b * c.b).sqrt();
        let denom = 3. * c.a;
        vec![(c.b + disc) / denom, (c.b - disc) / denom]
    }

    fn cycles_child(&self, Self::Param { a, b }: Self::Param, period: Period) -> Vec<Self::Var>
    {
        match period
        {
            1 =>
            {
                let u = b / a;
                solve_cubic(u, 2. / a - 1., -u).to_vec()
            }
            2 =>
            {
                let b2 = b * b;
                let u = 2. * b * a * a;
                let coeffs = [
                    b2 * (1. - a) + a,
                    u,
                    -a * horner_monic!(a, -b2, 1.),
                    -u,
                    a * a * a,
                ];
                solve_polynomial(coeffs)
            }
            _ => vec![],
        }
    }

    fn get_meta_params(&self) -> Self::MetaParam
    {
        self.multiplier
    }

    fn set_meta_param(&mut self, value: Self::MetaParam)
    {
        self.multiplier = value;
    }

    fn set_param(&mut self, value: Self::MetaParam)
    {
        self.multiplier = value;
    }

    fn param_map(&self, t: Cplx) -> Self::Param
    {
        let s = (1. - self.multiplier) / 4.;
        let t2 = t * t;
        let denom = t + t + 1.;
        CplxPair {
            a: (s - t2) / denom,
            b: (t2 + t + s) / denom,
        }
    }

    fn cycle_active_plane(&mut self)
    {
        self.starting_crit = self.starting_crit.swap();
    }

    fn default_julia_bounds(&self, _point: Cplx, _c: Self::Param) -> Bounds
    {
        Bounds::centered_square(4.)
    }

    fn name(&self) -> String
    {
        format!("Cubic Per(2, {})", self.multiplier)
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CubicPer2LambdaParam
{
    point_grid: PointGrid,
    max_iter: Period,
    starting_crit: PlaneID,
}

impl Default for CubicPer2LambdaParam
{
    fn default() -> Self
    {
        let bounds = Bounds::centered_square(2.5);
        let point_grid = PointGrid::new_by_res_y(1024, bounds);
        Self {
            point_grid,
            max_iter: 1024,
            starting_crit: PlaneID::ZPlane,
        }
    }
}

impl ParameterPlane for CubicPer2LambdaParam
{
    type Param = Cplx;
    type MetaParam = NoParam;
    type Var = Cplx;
    type Deriv = Cplx;
    type Child = CubicPer2Lambda;

    basic_plane_impl!();
    basic_escape_encoding!(3.);

    #[inline]
    fn degree_real(&self) -> f64
    {
        3.0
    }

    #[inline]
    fn map(&self, z: Self::Var, l: Self::Param) -> Self::Var
    {
        let a = 0.25 * (1.0 - l);
        horner!(z, a, -(a + 1.), -a, a)
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, l: Self::Param) -> (Self::Var, Self::Deriv)
    {
        let a = 0.25 * (1.0 - l);
        let f = horner!(z, a, -(a + 1.), -a, a);
        let df = horner!(z, -(a + 1.), -(a + a), 3. * a);
        (f, df)
    }

    #[inline]
    fn dynamical_derivative(&self, _z: Self::Var, _a: Self::Param) -> Self::Deriv
    {
        unimplemented!()
    }

    #[inline]
    fn parameter_derivative(&self, z: Self::Var, _c: Self::Param) -> Self::Deriv
    {
        z
    }

    #[inline]
    fn start_point(&self, _point: Cplx, l: Self::Param) -> Self::Var
    {
        let a = (1.0 - l) * 0.25;
        let disc = (a * (1. - l + 3.)).sqrt();
        match self.starting_crit
        {
            PlaneID::ZPlane => ONE_THIRD * (1. + disc / a),
            PlaneID::WPlane => ONE_THIRD * (1. - disc / a),
        }
    }

    fn cycle_active_plane(&mut self)
    {
        self.starting_crit = self.starting_crit.swap();
    }

    fn critical_points_child(&self, _param: Self::Param) -> Vec<Self::Var>
    {
        vec![ZERO, ONE]
    }

    fn name(&self) -> String
    {
        "Cubic Per(2, lambda) lambda-plane".to_owned()
    }

    fn default_selection(&self) -> Cplx
    {
        ZERO
    }

    fn default_julia_bounds(&self, point: Cplx, _param: Self::Param) -> Bounds
    {
        let r = 3.5 / (point.norm() + 0.01);
        Bounds::centered_square(r)
    }
}

impl From<CubicPer2LambdaParam> for CubicPer2Lambda
{
    fn from(parent: CubicPer2LambdaParam) -> Self
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

    #[inline]
    fn degree_real(&self) -> f64
    {
        3.0
    }

    #[inline]
    fn map(&self, z: Cplx, c: Cplx) -> Cplx
    {
        z * z * (z - c - 1. / c) + c
    }

    #[inline]
    fn start_point(&self, _point: Cplx, param: Cplx) -> Cplx
    {
        2. / 3. * (param + 1. / param)
    }

    #[inline]
    fn dynamical_derivative(&self, z: Cplx, c: Cplx) -> Cplx
    {
        // let u = z * (3. * z - c - c - 2. / c) * (z / c).re.signum();
        // u / u.norm()
        z * (3. * z - c - c - 2. / c)
    }

    #[inline]
    fn parameter_derivative(&self, z: Cplx, c: Cplx) -> Cplx
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
            3 =>
            {
                let c2 = c * c;
                let c3 = c * c2;
                let c4 = c2 * c2;
                let c5 = c2 * c3;
                let c6 = c3 * c3;
                let c8 = c4 * c4;
                let coeffs = [
                    c8,
                    ZERO,
                    c6 + c8,
                    c5,
                    -c4 * horner_monic!(c2, 2., 6., 0., -5., -1.),
                    c3 * horner_monic!(c2, -1., 2., 7., -1., -3.),
                    -c2 * horner_monic!(c2, -1., -7., 3., 30., 17., -9., -6.),
                    c3 * horner!(c2, -2., -18., 6., 46., 3., -16., 2.),
                    c2 * horner!(c2, -2., 8., 69., 33., -53., -33., -5., 5.),
                    c * horner!(c2, -1., 13., -24., -147., -33., 107., 36., -16.),
                    horner!(c2, 1., -2., -64., 26., 224., 75., -39., -5., -10.),
                    c * horner!(c2, -8., 26., 170., -56., -358., -160., 36., 44.),
                    horner!(c2, 3., 31., -98., -334., 41., 332., 91., -28., 10.),
                    c * horner!(c2, -21., -70., 254., 573., 95., -226., -135., -56.),
                    horner!(c2, 3., 80., 125., -429., -722., -151., 180., 95., -5.),
                    c * horner!(c2, -22., -209., -212., 492., 710., 243., 36., 34.),
                    horner!(c2, 1., 78., 400., 319., -426., -618., -307., -90., 1.),
                    c * horner!(c2, -8., -182., -601., -446., 183., 302., 98., -8.),
                    c2 * horner!(c2, 28., 308., 736., 570., 141., 28., 28.),
                    c3 * horner!(c2, -56., -378., -695., -520., -210., -56.),
                    c4 * horner!(c2, 70., 322., 449., 266., 70.),
                    c5 * horner!(c2, -56., -178., -170., -56.),
                    c6 * horner!(c2, 28., 57., 28.),
                    -8. * c6 * (c + c3),
                    c8,
                ];
                solve_polynomial(coeffs)
            }
            _ => vec![],
        }
    }

    #[inline]
    fn default_julia_bounds(&self, _point: Cplx, param: Cplx) -> Bounds
    {
        Bounds::square(2.2, param / 2.)
    }
}

impl HasDynamicalCovers for CubicPer2CritMarked
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
                    let g2 = 0.5.into();
                    let g3 = Cplx::new(-0.0625, 0.);
                    let (mut x, y) = weierstrass_p(g2, g3, t, 0.01);

                    x += x;
                    (x * (x - 1.) / (y + y - x + 0.5), ONE)
                };
                bounds = Bounds {
                    min_x: -3.5,
                    max_x: 3.5,
                    min_y: -3.5,
                    max_y: 3.5,
                };
            }
            2 =>
            {
                param_map = |t| {
                    let u = t.inv();
                    (t + u, 1. - u * u)
                };
                bounds = Bounds {
                    min_x: -2.2,
                    max_x: 2.2,
                    min_y: -2.8,
                    max_y: 2.8,
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
            1 =>
            {
                param_map = |t| {
                    let g2 = 0.5.into();
                    let g3 = Cplx::new(-0.0625, 0.);
                    let (mut x, y) = weierstrass_p(g2, g3, t, 0.01);

                    x += x;
                    (x * (x - 1.) / (y + y - x + 0.5), ONE)
                };
                bounds = Bounds {
                    min_x: -3.5,
                    max_x: 3.5,
                    min_y: -3.5,
                    max_y: 3.5,
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
}
