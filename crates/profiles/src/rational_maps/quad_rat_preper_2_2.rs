use crate::macros::{degree_impl, has_child_impl, profile_imports};
use crate::macros::{horner, horner_monic};
profile_imports!();

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct QuadRatPreper22
{
    point_grid: PointGrid,
    compute_mode: ComputeMode,
    max_iter: IterCount,
}

impl QuadRatPreper22
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -2.0,
        max_x: 3.0,
        min_y: -2.5,
        max_y: 2.5,
    };
}
impl Default for QuadRatPreper22
{
    fractal_impl!();
}

impl DynamicalFamily for QuadRatPreper22
{
    type Var = Cplx;
    type Param = CplxPair;
    type Deriv = Cplx;
    type MetaParam = NoParam;
    basic_plane_impl!();
    default_name!();

    fn map(&self, z: Self::Var, CplxPair { a, b }: &Self::Param) -> Self::Var
    {
        a / z * (1. - z) / (b + z)
    }

    fn map_and_multiplier(
        &self,
        z: Self::Var,
        CplxPair { a, b }: &Self::Param,
    ) -> (Self::Var, Self::Deriv)
    {
        let u = z * (b + z);
        (a * (1. - z) / u, -a * (b + z * (2. - z)) / (u * u))
    }

    fn param_map(&self, t: Cplx) -> Self::Param
    {
        let t = t.inv();
        let t2 = t * t;
        CplxPair {
            a: -t2,
            b: t2 + t + t,
        }
    }

    fn start_point(&self, _t: Cplx, CplxPair { a, b }: &Self::Param) -> Self::Var
    {
        1. + (b + 1.).sqrt() * (b + a + 2.).re.signum()
    }
}

default_bounds_impl!(QuadRatPreper22);
has_child_impl!(QuadRatPreper22);

impl MarkedPoints for QuadRatPreper22
{
    fn critical_points_child(&self, CplxPair { a: _, b }: &Self::Param) -> Vec<Self::Var>
    {
        let disc = (b + 1.).sqrt();
        vec![1. + disc, 1. - disc]
    }

    #[inline]
    fn cycles_child(&self, CplxPair { a, b }: &Self::Param, period: Period) -> Vec<Self::Var>
    {
        match period {
            1 => solve_cubic(-a, *a, *b).to_vec(),
            2 => vec![ZERO],
            3 => {
                let a2 = a * a;
                let b2 = b * b;
                let b3 = b * b2;
                let b4 = b2 * b2;
                let coeffs = [
                    -a2,
                    a2 * (3. - b) - 2. * a * b2,
                    horner!(a, -b4, b2 * (4. - b) - 3. * b, 2. * b - 4.),
                    horner!(a, b3 * (b - 3.), horner_monic!(b, -1., 7., -3.), 3. - b),
                    -horner_monic!(a, 2. * b2 * (2. - b), 4. * b - b2 - 3.),
                    b * (b - 3.) - 2. * a,
                    -ONE,
                ];
                solve_polynomial(coeffs)
            }
            _ => vec![],
        }
    }
}

impl InfinityFirstReturnMap for QuadRatPreper22
{
    degree_impl!(2, 2);
    // TODO: angle_map_large_param, escaping_phase
}

impl EscapeEncoding for QuadRatPreper22
{
    fn encode_escaping_point(
        &self,
        iters: IterCount,
        z: Cplx,
        CplxPair { a: _, b }: &Self::Param,
    ) -> PointInfo<Self::Deriv>
    {
        let phase = Some((iters % 2) as Period);
        if z.is_nan() {
            return PointInfo::Escaping {
                potential: (iters as f64) - 2.,
                phase,
            };
        }

        let expansion_rate = b.norm_sqr();
        let u = self.escape_radius().log(expansion_rate);
        let v = z.norm_sqr().log(expansion_rate);
        let residual = u - v;
        let potential = 2.0f64.mul_add(residual as IterCountSmooth, iters as IterCountSmooth);
        PointInfo::Escaping { potential, phase }
    }
}

impl ExternalRays for QuadRatPreper22 {}
