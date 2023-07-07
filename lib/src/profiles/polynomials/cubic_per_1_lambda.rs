use crate::macros::{basic_escape_encoding, horner, horner_monic, profile_imports};
use crate::math_utils::poly_solve::solve_polynomial;
use crate::math_utils::solve_quadratic;
use crate::types::variables::PlaneID;
profile_imports!();

const I: Cplx = Cplx::new(0., 1.);
const I2: Cplx = Cplx::new(0., 2.);
const A0: Cplx = Cplx::new(0., 27. / 64.);
const A2: Cplx = Cplx::new(0., -21. / 16.);
const A4: Cplx = Cplx::new(0., -7. / 4.);

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
    basic_escape_encoding!(3., 1.);

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
    fn start_point(&self, m: Cplx, _c: Self::Param) -> Self::Var
    {
        match self.starting_crit
        {
            PlaneID::ZPlane => 0.5 * self.multiplier * m,
            PlaneID::WPlane => TWO_THIRDS / m,
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
                solve_polynomial(&coeffs)
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

    fn param_map(&self, m: Cplx) -> Self::Param
    {
        -m.inv() - 0.75 * self.multiplier * m
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

    fn base_param(lambda: Cplx) -> Cplx
    {
        -Self::BASE_POINT.inv() - 0.75 * lambda * Self::BASE_POINT
    }
}
impl Default for CubicPer1LambdaParam
{
    fn default() -> Self
    {
        let bounds = Bounds {
            min_x: -2.2,
            max_x: 4.2,
            min_y: -2.5,
            max_y: 2.5,
        };
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
    basic_escape_encoding!(3., 1.);

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

    fn periodicity_tolerance(&self) -> Real
    {
        1e-6
    }
    fn min_iter(&self) -> Period
    {
        self.max_iter() / 3
    }
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
                solve_polynomial(&coeffs)
            }
            3 =>
            {
                let c2 = c * c;
                let coeffs = [
                    Cplx::new(3., 0.),
                    6.*c,
                    horner!(c2, 9., 9.),
                    c*horner!(c2, 32., 10.),
                    horner!(c2, 24., 64., 8.),
                    c*horner!(c2, 108., 86., 4.),
                    horner!(c2, 54., 248., 78., 1.),
                    c*horner!(c2, 272., 352., 48.),
                    horner!(c2, 102., 642., 331., 20.),
                    c*horner!(c2, 520., 906., 212., 6.),
                    horner!(c2, 156., 1198., 831., 94., 1.),
                    c*horner!(c2, 768., 1610., 512., 26.),
                    horner!(c2, 192., 1664., 1375., 202., 3.),
                    c*horner!(c2, 882., 2050., 742., 42.),
                    horner!(c2, 189., 1736., 1520., 225., 3.),
                    c*horner!(c2, 784., 1846., 636., 30.),
                    horner!(c2, 147., 1326., 1065., 126., 1.),
                    c*horner!(c2, 522., 1098., 294., 8.),
                    horner!(c2, 87., 687., 420., 28.),
                    c*horner!(c2, 240., 378., 56.),
                    horner!(c2, 36., 210., 70.),
                    c*horner!(c2, 66., 56.),
                    horner!(c2, 9., 28.),
                    8.*c,
                    ONE,
                ];
                solve_polynomial(&coeffs)
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

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CubicPer1_0
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl Default for CubicPer1_0
{
    fractal_impl!(-2.5, 2.5, -2.5, 2.5);
}

impl ParameterPlane for CubicPer1_0
{
    parameter_plane_impl!();
    default_name!();
    basic_escape_encoding!(3.);

    fn map(&self, z: Self::Var, c: Self::Param) -> Self::Var
    {
        z * z * (z + c)
    }

    #[inline]
    fn start_point(&self, _point: Cplx, c: Self::Param) -> Self::Var
    {
        -TWO_THIRDS * c
    }

    fn dynamical_derivative(&self, z: Self::Var, c: Self::Param) -> Self::Deriv
    {
        z * (c + c + 3. * z)
    }

    fn parameter_derivative(&self, z: Self::Var, _c: Self::Param) -> Self::Deriv
    {
        z * z
    }

    fn critical_points_child(&self, c: Self::Param) -> Vec<Self::Var>
    {
        vec![ZERO, -TWO_THIRDS * c]
    }

    #[inline]
    fn cycles_child(&self, c: Self::Param, period: Period) -> Vec<Self::Var>
    {
        match period
        {
            1 =>
            {
                let [r1, r2] = solve_quadratic(-ONE, c);
                vec![ZERO, r1, r2]
            }
            _ => vec![],
        }
    }

    fn default_julia_bounds(&self, _point: Cplx, c: Self::Param) -> Bounds
    {
        Bounds::square(2.5, -ONE_THIRD * c)
    }
}

impl HasDynamicalCovers for CubicPer1_0
{
    fn marked_cycle_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> Cplx;
        let bounds: Bounds;

        match period
        {
            1 =>
            {
                param_map = |t| t.inv() - t;
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
                    (t2 + 2.) * t / (t2 + 1.)
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
                param_map = |t| t.inv() - t;
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
                    let numer = t * horner!(t2, 2.25, -2., 4.);
                    let denom = horner!(t2, A0, A2, A4, I);
                    numer / denom
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
                    let t2 = t * t;
                    -(t2 * t2 + t2 + 1.) / (t * t2 + t)
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
                param_map = |c| c;
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
        let param_map: fn(Cplx) -> Cplx;
        let bounds: Bounds;

        match period
        {
            2 =>
            {
                param_map = |t| {
                    let t2 = t * t;
                    (t2 + 3.) * t / (t2 + 1.)
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

    fn dynatomic_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> Cplx;
        let bounds: Bounds;

        match period
        {
            2 =>
            {
                param_map = |t| {
                    let t2 = t * t;
                    let numer = -1. + t2 * (3. - t * (8. + t * (3. - t2)));
                    let denom = t * I2 * (t2 * t2 - 1.);
                    numer / denom
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
                param_map = |t| t + t.inv();
                bounds = Bounds {
                    min_x: -2.5,
                    max_x: 2.5,
                    min_y: -2.5,
                    max_y: 2.5,
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
