use crate::macros::{degree_impl, horner, horner_monic, profile_imports};
use dynamo_common::types::variables::{Bicomplex, PlaneID};
profile_imports!();

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Biquadratic
{
    point_grid: PointGrid,
    compute_mode: ComputeMode,
    max_iter: IterCount,
    multiplier: Cplx,
}

impl Biquadratic
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -1.6,
        max_x: 1.25,
        min_y: -1.25,
        max_y: 1.25,
    };
}

impl Default for Biquadratic
{
    fractal_impl!(multiplier, ZERO);
}

impl DynamicalFamily for Biquadratic
{
    type Var = Bicomplex;
    type Param = Cplx;
    type Deriv = Cplx;
    type MetaParam = Cplx;
    basic_plane_impl!();

    #[inline]
    fn name(&self) -> String
    {
        let param = self.multiplier;
        format!("Biquadratic({param})")
    }

    #[inline]
    fn param_map(&self, t: Cplx) -> Cplx
    {
        t
    }

    #[inline]
    fn start_point(&self, _point: Cplx, _c: &Cplx) -> Self::Var
    {
        Self::Var::default()
    }

    #[inline]
    fn map(&self, zw: Self::Var, c: &Cplx) -> Self::Var
    {
        match zw {
            Bicomplex::PlaneA(z) => Bicomplex::PlaneB(z.powi(2) + c),
            Bicomplex::PlaneB(w) => Bicomplex::PlaneA(w.powi(2) + self.multiplier),
        }
    }

    #[inline]
    fn map_and_multiplier(&self, zw: Self::Var, c: &Cplx) -> (Self::Var, Cplx)
    {
        match zw {
            Bicomplex::PlaneA(z) => (Bicomplex::PlaneB(z.powi(2) + c), 2. * z),
            Bicomplex::PlaneB(w) => (Bicomplex::PlaneA(w.powi(2) + self.multiplier), 2. * w),
        }
    }

    fn gradient(&self, zw: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        match zw {
            Bicomplex::PlaneA(z) => (Bicomplex::PlaneB(z.powi(2) + c), 2. * z, ONE),
            Bicomplex::PlaneB(w) => (Bicomplex::PlaneA(w.powi(2) + self.multiplier), 2. * w, ZERO),
        }
    }
}
default_bounds_impl!(Biquadratic);

impl EscapeEncoding for Biquadratic
{
    fn encode_escaping_point(
        &self,
        iters: IterCount,
        z: Self::Var,
        _base_param: &Cplx,
    ) -> PointInfo<Self::Deriv>
    {
        if z.is_nan() {
            return PointInfo::Escaping {
                potential: (iters as f64) - 1.,
                phase: None,
            };
        }

        let u = self.escape_radius().log2();
        let v = z.norm_sqr().log2();
        let residual = (v / u).log2() / 2.;
        let potential = (iters as f64) - (residual as IterCountSmooth);
        PointInfo::Escaping {
            potential,
            phase: None,
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BiquadraticMult
{
    point_grid: PointGrid,
    compute_mode: ComputeMode,
    max_iter: IterCount,
    multiplier: Cplx,
    starting_plane: PlaneID,
}

// impl BiquadraticMult
// {
//     const DEFAULT_BOUNDS: Bounds = Bounds {
//         min_x: -2.8,
//         max_x: 2.8,
//         min_y: -2.55,
//         max_y: 2.55,
//     };
// }

impl Default for BiquadraticMult
{
    fn default() -> Self
    {
        let multiplier = Cplx::new(0.5, 0.0);
        let bounds = Bounds::centered_square(2.5);
        let point_grid = PointGrid::new_by_res_y(1024, bounds);
        Self {
            point_grid,
            compute_mode: ComputeMode::default(),
            max_iter: 1024,
            multiplier,
            starting_plane: PlaneID::ZPlane,
        }
        .with_default_bounds()
    }
}

impl DynamicalFamily for BiquadraticMult
{
    type Var = Bicomplex;
    type Param = CplxPair;
    type Deriv = Cplx;
    type MetaParam = Cplx;
    basic_plane_impl!();

    #[inline]
    fn name(&self) -> String
    {
        let param = self.multiplier;
        format!("Biquadratic({param})")
    }

    #[inline]
    fn param_map(&self, point: Cplx) -> Self::Param
    {
        let point = -point.powf(ONE_THIRD);
        CplxPair {
            a: point,
            b: self.multiplier / point,
        }
    }

    #[inline]
    fn start_point(&self, _point: Cplx, c: &Self::Param) -> Self::Var
    {
        match self.starting_plane {
            PlaneID::ZPlane => Bicomplex::PlaneA(-0.5 * c.a),
            PlaneID::WPlane => Bicomplex::PlaneB(-0.5 * c.b),
        }
    }

    #[inline]
    fn map(&self, zw: Self::Var, c: &Self::Param) -> Self::Var
    {
        match zw {
            Bicomplex::PlaneA(z) => Bicomplex::PlaneB(z * (z + c.a)),
            Bicomplex::PlaneB(w) => Bicomplex::PlaneA(w * (w + c.b)),
        }
    }

    #[inline]
    fn map_and_multiplier(&self, zw: Self::Var, c: &Self::Param) -> (Self::Var, Cplx)
    {
        match zw {
            Bicomplex::PlaneA(z) => (Bicomplex::PlaneB(z * (z + c.a)), 2. * z + c.a),
            Bicomplex::PlaneB(w) => (Bicomplex::PlaneA(w * (w + c.b)), 2. * w + c.b),
        }
    }

    fn gradient(&self, zw: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        match zw {
            Bicomplex::PlaneA(z) => (Bicomplex::PlaneB(z * (z + c.a)), 2. * z + c.a, ONE),
            Bicomplex::PlaneB(w) => (
                Bicomplex::PlaneA(w * (w + c.b)),
                2. * w + c.b,
                -c.b.powi(2) / self.multiplier, // -l/t^2
            ),
        }
    }

    fn cycle_active_plane(&mut self)
    {
        self.starting_plane = self.starting_plane.swap();
    }

    fn periodicity_tolerance(&self) -> Real
    {
        1e-14
    }

    fn set_meta_param(&mut self, meta_param: Self::MetaParam)
    {
        self.multiplier = meta_param;
    }

    fn set_param(&mut self, multiplier: Cplx)
    {
        self.multiplier = multiplier;
    }

    fn get_meta_params(&self) -> Self::MetaParam
    {
        self.multiplier
    }

    fn get_param(&self) -> <Self::MetaParam as ParamList>::Param
    {
        self.multiplier
    }
}

impl FamilyDefaults for BiquadraticMult
{
    fn default_bounds(&self) -> Bounds
    {
        Bounds::square(14., self.default_selection())
    }

    fn default_selection(&self) -> Cplx
    {
        8. - 4. * self.multiplier
        // Cplx::new(1.062_658_8, 0.)
    }
}

impl HasJulia for BiquadraticMult
{
    fn default_bounds_child(&self, _point: Cplx, c: &Self::Param) -> Bounds
    {
        Bounds::square(2.5, -0.5 * c.a)
    }

    fn dynam_map(&self, point: Cplx) -> Self::Var
    {
        match self.starting_plane {
            PlaneID::ZPlane => Bicomplex::PlaneA(point),
            PlaneID::WPlane => Bicomplex::PlaneB(point),
        }
    }

    fn dynam_map_d(&self, point: Cplx) -> (Self::Var, Self::Deriv)
    {
        (self.dynam_map(point), ONE)
    }
}

impl MarkedPoints for BiquadraticMult
{
    fn critical_points(&self) -> Vec<Self::Var>
    {
        let l = self.multiplier;
        let d0 = ((l - 8.) * l + 32.).sqrt();
        let q0 = -2. * (l - 4. + d0);
        let q1 = -2. * (l - 4. - d0);
        [
            q0,
            q1,
            // q0 * OMEGA,
            // q1 * OMEGA,
            // q0 * OMEGA_BAR,
            // q1 * OMEGA_BAR,
        ]
        .iter()
        .copied()
        .map(Bicomplex::PlaneA)
        .collect()
    }

    #[inline]
    fn critical_points_child(&self, c: &Self::Param) -> Vec<Self::Var>
    {
        match self.starting_plane {
            PlaneID::ZPlane => {
                let disc = (c.a * c.a - c.b - c.b).sqrt();
                vec![
                    Bicomplex::PlaneA(-0.5 * c.a),
                    Bicomplex::PlaneA(-0.5 * (c.a + disc)),
                    Bicomplex::PlaneA(-0.5 * (c.a - disc)),
                ]
            }
            PlaneID::WPlane => {
                let disc = (c.b * c.b - c.a - c.a).sqrt();
                vec![
                    Bicomplex::PlaneB(-0.5 * c.b),
                    Bicomplex::PlaneB(-0.5 * (c.b + disc)),
                    Bicomplex::PlaneB(-0.5 * (c.b - disc)),
                ]
            }
        }
    }

    fn cycles_child(&self, CplxPair { a, b }: &Self::Param, period: Period) -> Vec<Self::Var>
    {
        match period {
            2 => match self.starting_plane {
                PlaneID::ZPlane => {
                    let [r0, r1, r2] = solve_cubic(a * b - 1., a.powi(2) + b, 2. * a);
                    vec![
                        Bicomplex::PlaneA(ZERO),
                        Bicomplex::PlaneA(r0),
                        Bicomplex::PlaneA(r1),
                        Bicomplex::PlaneA(r2),
                    ]
                }
                PlaneID::WPlane => {
                    let [r0, r1, r2] = solve_cubic(b * a - 1., b.powi(2) + a, 2. * b);
                    vec![
                        Bicomplex::PlaneB(ZERO),
                        Bicomplex::PlaneB(r0),
                        Bicomplex::PlaneB(r1),
                        Bicomplex::PlaneB(r2),
                    ]
                }
            },
            4 => {
                let b2 = b.powi(2);
                let b3 = b * b2;
                let coeffs = [
                    a * b + 1.,
                    horner!(a, b, b2, 1., b),
                    horner_monic!(a, b2, 2., 4. * b, 2. * b2),
                    horner!(a, 1., 5. * b, 5. * b2, b3 + 4., 4. * b),
                    horner!(a, 2. * b, 4. * b2, 3. * b3 + 6., 14. * b, 3. * b2, 2.),
                    horner!(a, b2, 3. * b3 + 4., 18. * b, 12. * b2, 9., 3. * b),
                    horner_monic!(a, b3 + 1., 10. * b, 18. * b2, 16., 15. * b, 0.),
                    horner!(a, 2. * b, 12. * b2, 14., 30. * b, 0., 6.),
                    horner!(a, 3. * b2, 6., 30. * b, 0., 15.),
                    horner!(a, 1., 15. * b, 0., 20.),
                    horner!(a, 3. * b, 0., 15.),
                    6. * a,
                    ONE,
                ];
                solve_polynomial(coeffs)
                    .into_iter()
                    .map(Bicomplex::PlaneA)
                    .collect()
            }
            _ => vec![],
        }
    }
}

impl EscapeEncoding for BiquadraticMult
{
    fn encode_escaping_point(
        &self,
        iters: IterCount,
        z: Self::Var,
        _base_param: &Self::Param,
    ) -> PointInfo<Self::Deriv>
    {
        if z.is_nan() {
            return PointInfo::Escaping {
                potential: (iters as f64) - 1.,
                phase: None,
            };
        }

        let u = self.escape_radius().log2();
        let v = z.norm_sqr().log2();
        let residual = (v / u).log2();
        let potential = (iters as f64) - (residual as IterCountSmooth);
        PointInfo::Escaping {
            potential,
            phase: None,
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BiquadraticMultParam
{
    point_grid: PointGrid,
    compute_mode: ComputeMode,
    max_iter: IterCount,
    starting_plane: PlaneID,
}

impl BiquadraticMultParam
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -2.2,
        max_x: 4.2,
        min_y: -2.5,
        max_y: 2.5,
    };
}
impl Default for BiquadraticMultParam
{
    fn default() -> Self
    {
        let bounds = Self::DEFAULT_BOUNDS;
        let point_grid = PointGrid::new_by_res_y(1024, bounds);
        Self {
            point_grid,
            compute_mode: ComputeMode::default(),
            max_iter: 1024,
            starting_plane: PlaneID::ZPlane,
        }
    }
}

impl DynamicalFamily for BiquadraticMultParam
{
    type Param = CplxPair;
    type Var = Bicomplex;
    type Deriv = Cplx;
    type MetaParam = NoParam;
    basic_plane_impl!();

    #[inline]
    fn param_map(&self, t: Cplx) -> Self::Param
    {
        CplxPair {
            a: 1e-4.into(),
            b: 1e4 * t,
        }
    }

    #[inline]
    fn map(&self, zw: Self::Var, c: &Self::Param) -> Self::Var
    {
        match zw {
            Bicomplex::PlaneA(z) => Bicomplex::PlaneB(z * (z + c.a)),
            Bicomplex::PlaneB(w) => Bicomplex::PlaneA(w * (w + c.b)),
        }
    }

    #[inline]
    fn map_and_multiplier(&self, zw: Self::Var, c: &Self::Param) -> (Self::Var, Cplx)
    {
        match zw {
            Bicomplex::PlaneA(z) => (Bicomplex::PlaneB(z * (z + c.a)), 2. * z + c.a),
            Bicomplex::PlaneB(w) => (Bicomplex::PlaneA(w * (w + c.b)), 2. * w + c.b),
        }
    }

    #[inline]
    fn start_point(&self, _point: Cplx, c: &Self::Param) -> Self::Var
    {
        Bicomplex::PlaneA(-0.5 * c.a)
    }

    fn cycle_active_plane(&mut self)
    {
        self.starting_plane = self.starting_plane.swap();
    }

    fn name(&self) -> String
    {
        "Biquadratic Param".to_owned()
    }
}

impl FamilyDefaults for BiquadraticMultParam
{
    default_bounds!();

    fn default_selection(&self) -> Cplx
    {
        // (1.0 - 5.0_f64.sqrt()).into()
        Cplx::new(0., 0.99)
    }
}

impl HasChild<BiquadraticMult> for BiquadraticMultParam
{
    fn to_child_param(
        CplxPair { a, b }: Self::Param,
    ) -> <<BiquadraticMult as DynamicalFamily>::MetaParam as ParamList>::Param
    {
        a * b
    }
}

impl HasJulia for BiquadraticMultParam
{
    fn default_bounds_child(&self, _point: Cplx, _param: &Self::Param) -> Bounds
    {
        Bounds::centered_square(3.5)
    }
}

impl InfinityFirstReturnMap for BiquadraticMultParam
{
    degree_impl!(2);
}

impl EscapeEncoding for BiquadraticMultParam
{
    fn encode_escaping_point(
        &self,
        iters: IterCount,
        z: Self::Var,
        _base_param: &Self::Param,
    ) -> PointInfo<Self::Deriv>
    {
        if z.is_nan() {
            return PointInfo::Escaping {
                potential: (iters as f64) - 1.,
                phase: None,
            };
        }

        let u = self.escape_radius().log2();
        let v = z.norm_sqr().log2();
        let residual = (v / u).log2();
        let potential = (iters as f64) - (residual as IterCountSmooth);
        PointInfo::Escaping {
            potential,
            phase: None,
        }
    }
}

impl From<BiquadraticMultParam> for BiquadraticMult
{
    fn from(parent: BiquadraticMultParam) -> Self
    {
        let point = parent.default_selection();
        let param = parent.param_map(point);
        let point_grid = parent.point_grid().clone();
        Self {
            point_grid,
            compute_mode: ComputeMode::default(),
            max_iter: parent.max_iter(),
            multiplier: param.a * param.b,
            starting_plane: parent.starting_plane,
        }
        .with_default_bounds()
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BiquadraticMultSecondIterate
{
    point_grid: PointGrid,
    compute_mode: ComputeMode,
    max_iter: IterCount,
    multiplier: Cplx,
}

impl BiquadraticMultSecondIterate
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -2.6,
        max_x: 3.25,
        min_y: -2.25,
        max_y: 2.25,
    };
}
impl Default for BiquadraticMultSecondIterate
{
    fractal_impl!(multiplier, ZERO);
}

impl DynamicalFamily for BiquadraticMultSecondIterate
{
    type Var = Cplx;
    type Param = Cplx;
    type Deriv = Cplx;
    type MetaParam = Cplx;
    basic_plane_impl!();

    #[inline]
    fn name(&self) -> String
    {
        let param = self.multiplier;
        format!("BiquadraticMult({param})")
    }

    #[inline]
    fn param_map(&self, t: Cplx) -> Cplx
    {
        t
    }

    #[inline]
    fn start_point(&self, _point: Cplx, c: &Cplx) -> Cplx
    {
        -0.5 * c
    }

    #[inline]
    fn map(&self, z: Cplx, c: &Cplx) -> Cplx
    {
        let w = z * (z + c);
        w * (w + self.multiplier / c)
    }

    #[inline]
    fn map_and_multiplier(&self, z: Cplx, c: &Cplx) -> (Cplx, Cplx)
    {
        let a = self.multiplier / c;
        let w = z * (z + c);
        (w * (w + a), (c + 2. * z) * (a + 2. * w))
    }

    #[inline]
    fn gradient(&self, z: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        let a = self.multiplier / c;
        let x0 = c + z;
        let w = z * x0;
        let x2 = w + a;
        let x2z = x2 * z;
        (
            w * x2,
            x0 * x2 + w * (c + 2. * z) + x2z,
            w * (z - a.powi(2)) + x2z,
        )
    }

    fn set_meta_param(&mut self, value: Self::Param)
    {
        self.multiplier = value;
    }

    fn get_meta_params(&self) -> Self::Param
    {
        self.multiplier
    }
}

impl FamilyDefaults for BiquadraticMultSecondIterate
{
    default_bounds!();
}

impl HasJulia for BiquadraticMultSecondIterate
{
    fn default_bounds_child(&self, _point: Cplx, _c: &Self::Param) -> Bounds
    {
        Bounds::centered_square(2.5)
    }
}

impl EscapeEncoding for BiquadraticMultSecondIterate
{
    fn encode_escaping_point(
        &self,
        iters: IterCount,
        z: Cplx,
        _base_param: &Cplx,
    ) -> PointInfo<Self::Deriv>
    {
        if z.is_nan() {
            return PointInfo::Escaping {
                potential: (iters as f64) - 1.,
                phase: None,
            };
        }

        let u = self.escape_radius().log2();
        let v = z.norm_sqr().log2();
        let residual = (v / u).log2() / 2.;
        let potential = (iters as f64) - (residual as IterCountSmooth);
        PointInfo::Escaping {
            potential,
            phase: None,
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BiquadraticMultSection
{
    point_grid: PointGrid,
    compute_mode: ComputeMode,
    max_iter: IterCount,
    starting_plane: PlaneID,
}

impl BiquadraticMultSection
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -2.8,
        max_x: 2.8,
        min_y: -2.55,
        max_y: 2.55,
    };
}

impl Default for BiquadraticMultSection
{
    fn default() -> Self
    {
        let bounds = Self::DEFAULT_BOUNDS;
        let point_grid = PointGrid::new_by_res_y(1024, bounds);
        Self {
            point_grid,
            compute_mode: ComputeMode::default(),
            max_iter: 1024,
            starting_plane: PlaneID::ZPlane,
        }
    }
}

impl DynamicalFamily for BiquadraticMultSection
{
    type Var = Bicomplex;
    type Param = Cplx;
    type Deriv = Cplx;
    type MetaParam = NoParam;
    basic_plane_impl!();

    #[inline]
    fn name(&self) -> String
    {
        "Biquadratic Section".to_string()
    }

    #[inline]
    fn param_map(&self, multiplier: Cplx) -> Self::Param
    {
        multiplier
    }

    #[inline]
    fn start_point(&self, _point: Cplx, c: &Self::Param) -> Self::Var
    {
        match self.starting_plane {
            PlaneID::ZPlane => Bicomplex::PlaneA(-0.5 * c),
            PlaneID::WPlane => Bicomplex::PlaneB(Cplx::from(-0.5)),
        }
    }

    #[inline]
    fn map(&self, zw: Self::Var, c: &Self::Param) -> Self::Var
    {
        match zw {
            Bicomplex::PlaneA(z) => Bicomplex::PlaneB(z * (z + c)),
            Bicomplex::PlaneB(w) => Bicomplex::PlaneA(w * (w + 1.)),
        }
    }

    #[inline]
    fn map_and_multiplier(&self, zw: Self::Var, c: &Self::Param) -> (Self::Var, Cplx)
    {
        match zw {
            Bicomplex::PlaneA(z) => (Bicomplex::PlaneB(z * (z + c)), 2. * z + c),
            Bicomplex::PlaneB(w) => (Bicomplex::PlaneA(w * (w + 1.)), 2. * w + 1.),
        }
    }

    #[inline]
    fn gradient(&self, zw: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        match zw {
            Bicomplex::PlaneA(z) => (Bicomplex::PlaneB(z * (z + c)), 2. * z + c, z),
            Bicomplex::PlaneB(w) => (Bicomplex::PlaneA(w * (w + 1.)), 2. * w + 1., ZERO),
        }
    }

    fn cycle_active_plane(&mut self)
    {
        self.starting_plane = self.starting_plane.swap();
    }

    fn periodicity_tolerance(&self) -> Real
    {
        1e-14
    }
}

impl FamilyDefaults for BiquadraticMultSection
{
    default_bounds!();

    fn default_selection(&self) -> Cplx
    {
        Cplx::new(1.062_658_8, 0.)
    }
}

impl HasJulia for BiquadraticMultSection
{
    fn default_bounds_child(&self, _point: Cplx, a: &Self::Param) -> Bounds
    {
        Bounds::square(2.5, -0.5 * a)
    }

    fn dynam_map(&self, point: Cplx) -> Self::Var
    {
        match self.starting_plane {
            PlaneID::ZPlane => Bicomplex::PlaneA(point),
            PlaneID::WPlane => Bicomplex::PlaneB(point),
        }
    }

    fn dynam_map_d(&self, point: Cplx) -> (Self::Var, Self::Deriv)
    {
        (self.dynam_map(point), ONE)
    }
}

impl MarkedPoints for BiquadraticMultSection
{
    #[inline]
    fn critical_points_child(&self, c: &Self::Param) -> Vec<Self::Var>
    {
        match self.starting_plane {
            PlaneID::ZPlane => {
                let disc = (c.powi(2) - 2.).sqrt();
                vec![
                    Bicomplex::PlaneA(-0.5 * c),
                    Bicomplex::PlaneA(-0.5 * (c + disc)),
                    Bicomplex::PlaneA(-0.5 * (c - disc)),
                ]
            }
            PlaneID::WPlane => {
                let disc = (1. - c - c).sqrt();
                vec![
                    Bicomplex::PlaneB((-0.5).into()),
                    Bicomplex::PlaneB(-0.5 * (1. + disc)),
                    Bicomplex::PlaneB(-0.5 * (1. - disc)),
                ]
            }
        }
    }

    fn cycles_child(&self, a: &Self::Param, period: Period) -> Vec<Self::Var>
    {
        match period {
            2 => match self.starting_plane {
                PlaneID::ZPlane => {
                    let [r0, r1, r2] = solve_cubic(a - 1., a.powi(2) + 1., 2. * a);
                    vec![
                        Bicomplex::PlaneA(ZERO),
                        Bicomplex::PlaneA(r0),
                        Bicomplex::PlaneA(r1),
                        Bicomplex::PlaneA(r2),
                    ]
                }
                PlaneID::WPlane => {
                    let [r0, r1, r2] = solve_cubic(a - 1., 1. + a, TWO);
                    vec![
                        Bicomplex::PlaneB(ZERO),
                        Bicomplex::PlaneB(r0),
                        Bicomplex::PlaneB(r1),
                        Bicomplex::PlaneB(r2),
                    ]
                }
            },
            _ => vec![],
        }
    }
}

impl EscapeEncoding for BiquadraticMultSection
{
    fn encode_escaping_point(
        &self,
        iters: IterCount,
        z: Self::Var,
        _base_param: &Self::Param,
    ) -> PointInfo<Self::Deriv>
    {
        if z.is_nan() {
            return PointInfo::Escaping {
                potential: (iters as f64) - 1.,
                phase: None,
            };
        }

        let u = self.escape_radius().log2();
        let v = z.norm_sqr().log2();
        let residual = (v / u).log2();
        let potential = (iters as f64) - (residual as IterCountSmooth);
        PointInfo::Escaping {
            potential,
            phase: None,
        }
    }
}

impl InfinityFirstReturnMap for Biquadratic
{
    degree_impl!(2);
}
impl InfinityFirstReturnMap for BiquadraticMult
{
    degree_impl!(2);
}
impl InfinityFirstReturnMap for BiquadraticMultSecondIterate
{
    degree_impl!(2);
}
impl InfinityFirstReturnMap for BiquadraticMultSection
{
    degree_impl!(2);
}

impl ExternalRays for Biquadratic {}
impl ExternalRays for BiquadraticMult {}
impl ExternalRays for BiquadraticMultParam {}
impl ExternalRays for BiquadraticMultSection {}

impl MarkedPoints for Biquadratic {}
impl MarkedPoints for BiquadraticMultParam {}
impl MarkedPoints for BiquadraticMultSecondIterate {}
