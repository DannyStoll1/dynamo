use crate::macros::{horner, horner_monic, profile_imports};
use dynamo_common::math_utils::weierstrass_p;
profile_imports!();

const G2: Cplx = Cplx::new(2.75, 0.);
const G3: Cplx = Cplx::new(-0.375, 0.);

// Utility function for determining smooth coloring rate
fn top_coeff(a: Cplx, b: Cplx) -> Cplx
{
    let a2 = a.powi(2);
    let x = a + 1.;
    let x2 = x.powi(2);
    let x3 = x2 * x;
    let y = a + x;

    let c7 = 3. * x * (16. * a + 17.);
    let c6 = x * horner!(a, 157., 304., 142.);
    let c5 = x * horner!(a, 295., 911., 898., 281.);
    let c4 = x2 * horner!(a, 332., 1155., 1221., 382.);
    let c3 = x2 * horner!(a, 215., 1118., 2005., 1458., 354.);
    let c2 = x3 * horner!(a, 77., 479., 1020., 845., 214.);
    let c1 = x3 * y * horner!(a, 14., 91., 197., 159., 38.);
    let c0 = (x2 * y).powi(2) * (a + y) * (y + a * x);

    let d0 = a2 * x2 * horner!(x, 6., 4., -145., 200., 423., -782., -181., 450., 145.);
    let d1 =
        (x2 + x2) * horner!(x, -30., 10., 575., -1073., -991., 3069., -236., -2262., 494., 452.);
    let d2 =
        2. * x * horner!(x, -3., 96., 206., -1496., -233., 4850., -1085., -4841., 1371., 1239.);
    let d3 = 2. * x * horner!(x, 21., -39., -847., 162., 4979., -1887., -6105., 2292., 1960.);
    let d4 = horner!(x, 1., -52., -624., 336., 7378., -3342., -10034., 5234., 3935.);
    let d5 = horner!(x, -2., -144., 288., 3928., -1398., -5412., 4368., 2580.);
    let d6 = horner!(x, -16., 144., 1430., 30., -1682., 2738., 1084.);
    let d7 = horner!(x, 26., 322., 286., -78., 1272., 268.);
    let d8 = horner!(x, 36., 106., 166., 414., 30.);
    let d9 = 4. * (7. * x + 3.) * (3. * x + 1.);
    let d10 = 8. * (x + 1.);

    let n0 = horner_monic!(b, c0, c1, c2, c3, c4, c5, c6, c7, 10. * x);
    let n1 = a + b + 1.;
    let denom = horner!(b, d0, d1, d2, d3, d4, d5, d6, d7, d8, d9, d10);
    n0 * n1 / denom
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct QuadRatPer5
{
    point_grid: PointGrid,
    compute_mode: ComputeMode,
    max_iter: IterCount,
}
impl QuadRatPer5
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -2.5,
        max_x: 2.5,
        min_y: -2.5,
        max_y: 2.5,
    };
}
impl Default for QuadRatPer5
{
    fractal_impl!();
}

impl DynamicalFamily for QuadRatPer5
{
    type Var = Cplx;
    type Param = CplxPair;
    type Deriv = Cplx;
    type MetaParam = NoParam;

    basic_plane_impl!();
    default_name!();

    #[inline]
    fn map(&self, z: Self::Var, c: &Self::Param) -> Self::Var
    {
        let z2 = z.powi(2);
        1. + (c.a * z + c.b) / z2
    }
    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv)
    {
        let z2 = z.powi(2);
        let az = c.a * z;
        (1. + (az + c.b) / z2, -(az + 2. * c.b) / (z2 * z))
    }

    #[inline]
    fn start_point(&self, _point: Cplx, CplxPair { a, b }: &Self::Param) -> Self::Var
    {
        -2. * b / a
    }

    fn param_map(&self, t: Cplx) -> Self::Param
    {
        let (mut x, mut y) = weierstrass_p(G2, G3, t, 0.01);

        // F5 = 4*x^3 - y^2 - 11/4*x + 3/8

        x = -x - 0.25;

        y += x + 1.;
        y /= 2. * x;

        // F1 = x*y^2 + x^2 - x*y + x - y

        let mut tmp = y - x - 1.;
        x /= tmp;
        y = x - tmp.inv();
        // E5 = 2*x^3 + x^2*y - 3*x*y^2 + y^3 + 2*x^2 + x*y - y^2 + x

        tmp = (x + 1.) * y;
        let b = x / tmp;
        let a = x * b - 1.;

        // E0 = 2*a^5 + 8*a^4*b + 13*a^3*b^2 + 11*a^2*b^3 + 5*a*b^4 + b^5 + 11*a^4 + 35*a^3*b + 42*a^2*b^2 + 23*a*b^3 + 5*b^4 + 21*a^3 + 53*a^2*b + 44*a*b^2 + 12*b^3 + 18*a^2 + 33*a*b + 15*b^2 + 7*a + 7*b + 1

        CplxPair::from((a, b))
    }

    fn escape_radius(&self) -> Real
    {
        1e24
    }
}

impl FamilyDefaults for QuadRatPer5
{
    default_bounds!();

    fn default_selection(&self) -> Cplx
    {
        ONE
    }
}

impl HasJulia for QuadRatPer5
{
    fn default_bounds_child(&self, _point: Cplx, param: &Self::Param) -> Bounds
    {
        Bounds::square(20., self.start_point(ONE, param))
    }
}

impl MarkedPoints for QuadRatPer5
{
    fn cycles_child(&self, CplxPair { a, b }: &Self::Param, period: Period) -> Vec<Self::Var>
    {
        match period {
            1 => solve_cubic(-b, -a, -ONE).to_vec(),
            2 => solve_quadratic(*b, a - b).to_vec(),
            3 => {
                let b2 = b.powi(2);
                let b3 = b * b2;
                let u = 3. * (b + 1.);
                let ub = u * b;
                let coeffs = [
                    b3 * (1. + a + b),
                    b2 * horner!(a, -b, u, 3.),
                    b * horner!(a, ub, 2. * b, u + b, 3.),
                    horner_monic!(a, b2 * (b - 2.), b * (5. * b + 6.), 7. * b, u - 2.),
                    horner_monic!(a, ub + b2, ub, u + b, 4.),
                    horner!(a, -b, u, 2. * b + 5., 2.),
                    horner_monic!(a, b2 + 2. * b + 1., 2. * b + 2.),
                ];
                solve_polynomial(coeffs)
            }
            _ => vec![],
        }
    }

    #[inline]
    fn critical_points_child(&self, param: &Self::Param) -> Vec<Self::Var>
    {
        vec![self.start_point(ONE, param)]
    }
}

impl InfinityFirstReturnMap for QuadRatPer5
{
    #[inline]
    fn degree_real(&self) -> Real
    {
        2.0
    }

    #[inline]
    fn degree(&self) -> AngleNum
    {
        2
    }

    #[inline]
    fn escaping_period(&self) -> Period
    {
        5
    }
}

impl EscapeEncoding for QuadRatPer5
{
    fn encode_escaping_point(
        &self,
        iters: IterCount,
        z: Cplx,
        CplxPair { a, b }: &Self::Param,
    ) -> PointInfo<Self::Deriv>
    {
        let phase = Some((iters % 5) as Period);
        if z.is_nan() {
            return PointInfo::Escaping {
                potential: (iters as f64) - 5.,
                phase,
            };
        }

        let u = self.escape_radius().log2();
        let v = z.norm_sqr().log2();
        let delta = top_coeff(*a, *b).norm_sqr().log2();
        let residual = ((u + delta) / (v + delta)).log2();
        let potential = (residual as IterCountSmooth).mul_add(5., iters as f64);
        PointInfo::Escaping { potential, phase }
    }
}
impl ExternalRays for QuadRatPer5 {}
