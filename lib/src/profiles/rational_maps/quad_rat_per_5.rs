use crate::math_utils::{solve_cubic, solve_quadratic, weierstrass_p};
use crate::{macros::*, types::param_stack::Summarize};
use derive_more::{Add, Display, From};
profile_imports!();

const G2: ComplexNum = ComplexNum::new(2.75, 0.);
const G3: ComplexNum = ComplexNum::new(-0.375, 0.);

fn top_coeff(a: ComplexNum, b: ComplexNum) -> ComplexNum
{
    let a2 = a * a;
    let x = a + 1.;
    let x2 = x * x;
    let x3 = x2 * x;
    let y = a + x;

    let c7 = 3. * x * (16. * a + 17.);
    let c6 = x * horner!(a, 157., 304., 142.);
    let c5 = x * horner!(a, 295., 911., 898., 281.);
    let c4 = x2 * horner!(a, 332., 1155., 1221., 382.);
    let c3 = x2 * horner!(a, 215., 1118., 2005., 1458., 354.);
    let c2 = x3 * horner!(a, 77., 479., 1020., 845., 214.);
    let c1 = x3 * y * horner!(a, 14., 91., 197., 159., 38.);
    let c0 = x2 * x2 * y * y * (a + y) * (y + a * x);

    let d0 = a2 * x2 * horner!(x, 6., 4., -145., 200., 423., -782., -181., 450., 145.);
    let d1 =
        (x2 + x2) * horner!(x, -30., 10., 575., -1073., -991., 3069., -236., -2262., 494., 452.);
    let d2 =
        (x + x) * horner!(x, -3., 96., 206., -1496., -233., 4850., -1085., -4841., 1371., 1239.);
    let d3 = (x + x) * horner!(x, 21., -39., -847., 162., 4979., -1887., -6105., 2292., 1960.);
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

#[derive(Default, Clone, Copy, Debug, Add, From, PartialEq, Display)]
#[display(fmt = "[ a: {}, b: {} ] ", a, b)]
pub struct Param
{
    pub a: ComplexNum,
    pub b: ComplexNum,
}

impl Summarize for Param {}

impl From<ComplexNum> for Param
{
    fn from(z: ComplexNum) -> Self
    {
        let (mut x, mut y) = weierstrass_p(G2, G3, z, 0.01);

        // F5 = 4*x^3 - y^2 - 11/4*x + 3/8

        y /= 2.;
        x = -x - 0.25;
        // F3 = x^3 + 3/4*x^2 + y^2 - 1/2*x - 1/4

        y += 0.5 * (x + 1.);
        // F2 = x^3 + x^2 - (x+1)*y + y^2

        y /= x;
        // F1 = x*y^2 + x^2 - x*y + x - y

        let z = y - x - 1.;
        y = x - 1.;
        // F0 = 2*x^3 + x^2*y - 3*x*y^2 + y^3 + 2*x^2*z + x*y*z - y^2*z + x*z^2

        x /= z;
        y /= z;
        // E5 = 2*x^3 + x^2*y - 3*x*y^2 + y^3 + 2*x^2 + x*y - y^2 + x

        let t = (x + 1.) * y;
        let b = x / t;
        let a = x * b - 1.;

        // E0 = 2*a^5 + 8*a^4*b + 13*a^3*b^2 + 11*a^2*b^3 + 5*a*b^4 + b^5 + 11*a^4 + 35*a^3*b + 42*a^2*b^2 + 23*a*b^3 + 5*b^4 + 21*a^3 + 53*a^2*b + 44*a*b^2 + 12*b^3 + 18*a^2 + 33*a*b + 15*b^2 + 7*a + 7*b + 1

        Self::from((a, b))
    }
}

impl From<Param> for ComplexNum
{
    fn from(value: Param) -> Self
    {
        -(value.b + value.b) / value.a
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct QuadRatPer5
{
    point_grid: PointGrid,
    max_iter: Period,
}
impl Default for QuadRatPer5
{
    fractal_impl!(-2.5, 2.5, -2.5, 2.5);
}

impl ParameterPlane for QuadRatPer5
{
    type Var = ComplexNum;
    type Param = Param;
    type Deriv = ComplexNum;
    type MetaParam = NoParam;
    type Child = JuliaSet<Self>;

    basic_plane_impl!();
    default_name!();

    #[inline]
    fn map(&self, z: Self::Var, c: Self::Param) -> Self::Var
    {
        let z2 = z * z;
        (z2 + c.a * z + c.b) / z2
    }
    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, c: Self::Param) -> (Self::Var, Self::Deriv)
    {
        let z2 = z * z;
        let az = c.a * z;
        ((z2 + az + c.b) / z2, -(az + c.b + c.b) / (z2 * z))
    }

    #[inline]
    fn dynamical_derivative(&self, z: Self::Var, c: Self::Param) -> Self::Deriv
    {
        let z2 = z * z;
        -(c.a * z + c.b + c.b) / (z2 * z)
    }
    #[inline]
    fn parameter_derivative(&self, _z: Self::Var, _c: Self::Param) -> Self::Deriv
    {
        ONE
    }
    fn encode_escaping_point(
        &self,
        iters: Period,
        z: ComplexNum,
        c: Self::Param,
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
        let q = top_coeff(c.a, c.b).norm_sqr().log2();
        let residual = ((u + q) / (v + q)).log2();
        // let residual = ((v - 1.) / (u + u - 1.)).log2() + 1.;
        // (F - M) / (2L - M)
        let potential = (residual as IterCount).mul_add(5., f64::from(iters));
        PointInfo::Escaping { potential }
    }

    fn escape_radius(&self) -> RealNum
    {
        1e24
    }

    fn critical_points_child(&self, param: Self::Param) -> Vec<Self::Var>
    {
        vec![self.start_point(ONE, param)]
    }

    fn cycles_child(&self, Param { a, b }: Self::Param, period: Period) -> Vec<Self::Var>
    {
        match period
        {
            1 => solve_cubic(-b, -a, -ONE).to_vec(),
            2 => solve_quadratic(b, a - b).to_vec(),
            _ => vec![],
        }
    }

    fn default_julia_bounds(&self, _point: ComplexNum, param: Self::Param) -> Bounds
    {
        Bounds::square(20., self.start_point(ONE, param))
    }

    fn default_selection(&self) -> ComplexNum
    {
        ONE
    }
}
