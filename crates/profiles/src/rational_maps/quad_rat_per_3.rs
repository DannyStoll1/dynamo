use crate::macros::{
    degree_impl, ext_ray_impl_nonmonic, ext_ray_impl_rk, has_child_impl, horner, horner_monic,
    profile_imports,
};
use dynamo_common::math_utils::weierstrass_p;
profile_imports!();

// Quadratic rational maps with a critical 3-cycle: -c => ∞ -> 1 -> -c
#[derive(Clone, Debug)]
pub struct QuadRatPer3
{
    point_grid: PointGrid,
    compute_mode: ComputeMode,
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
}
impl Default for QuadRatPer3
{
    fractal_impl!();
}

type Prm = param::Param;

impl DynamicalFamily for QuadRatPer3
{
    type Var = Cplx;
    type Param = Prm;
    type Deriv = Cplx;
    type MetaParam = NoParam;
    basic_plane_impl!();
    default_name!();

    fn description(&self) -> String
    {
        "The moduli space of quadratic rational maps with a critical 3-cycle, \
            parameterized as $f_c(z) = (z^2 + c^3 - c - 1)/(z^2 - c^2)$. \
            In these coordinates, ∞ -> 1 -> -c is the critical 3-cycle. \
            The plane is colored according to the \
            activity of the free critical point 0."
            .to_owned()
    }

    fn start_point(&self, _point: Cplx, _c: &Prm) -> Cplx
    {
        0.0.into()
    }

    fn start_point_d(&self, _point: Cplx, _c: &Self::Param)
        -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        (ZERO, ZERO, ZERO)
    }

    #[inline]
    fn map_and_multiplier(&self, z: Cplx, Prm { a, b, c: _ }: &Self::Param) -> (Cplx, Cplx)
    {
        let z2 = z.powi(2);
        let v = z2 + b;
        ((z2 + a) / v, 2.0 * (b - a) * z / v.powi(2))
    }

    #[inline]
    fn map(&self, z: Cplx, Prm { a, b, c: _ }: &Self::Param) -> Cplx
    {
        let z2 = z.powi(2);
        (z2 + a) / (z2 + b)
    }

    #[inline]
    fn gradient(
        &self,
        z: Self::Var,
        Prm { a: _, b: _, c }: &Self::Param,
    ) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        let z2 = z.powi(2);
        let c2 = c.powi(2);
        let r = c2 - z2;
        let u = r.inv();
        let u2 = u.powi(2);
        let v = c + 1.;

        let f = u * (v - z2 - c2 * c);
        let df_dz = 2. * (1. - c) * v.powi(2) * z * u2;
        let df_dc = v * u2 * (r - c * (r + 2. * (ONE - z2)));
        (f, df_dz, df_dc)
    }

    #[inline]
    fn param_map(&self, point: Cplx) -> Self::Param
    {
        point.into()
    }
}

has_child_impl!(QuadRatPer3, 4.0);
default_bounds_impl!(QuadRatPer3);

impl MarkedPoints for QuadRatPer3
{
    #[inline]
    fn critical_points_child(&self, _param: &Prm) -> ComplexVec
    {
        vec![(0.).into()]
    }

    fn cycles_child(&self, Prm { a: _, b, c }: &Prm, period: Period) -> ComplexVec
    {
        match period {
            1 => {
                let x0 = -b;
                let x1 = c * x0;
                let x2 = 3. * x0 + 1.;
                let u = 27. * (c - x1) - 9. * x0 + 25.;
                let x3 = (0.5 * (u + (-4. * x2.powi(3) + u.powi(2)).sqrt())).powf(ONE_THIRD);
                let x4 = x3 / 3.;
                let x5 = x2 / (3. * x3);
                let r1 = -x4 * OMEGA_BAR - x5 * OMEGA + ONE_THIRD;
                let r2 = -x4 * OMEGA - x5 * OMEGA_BAR + ONE_THIRD;
                vec![-x4 - x5 + ONE_THIRD, r1, r2]
            }
            2 => {
                let disc = (c * (5. * c + 6.) + 5.).sqrt();
                vec![-0.5 * (c - disc + 1.), -0.5 * (c + disc + 1.)]
            }
            3 => {
                let c2 = -b;
                let u = (c - 1.).inv();
                let a0 = u * (1. + c + c2 + c2.powi(2));
                let a1 = u * (1. + c * (1. - 2. * c2));
                let a2 = -u * (2. + c + c2);
                let [r0, r1, r2] = solve_cubic(a0, a1, a2);
                vec![ONE, -c, r0, r1, r2]
            }
            4 => {
                let c2 = -b;
                let coeffs = [
                    horner_monic!(c, -1., -4., -7., -2., 13., 29., 29., 15., -3., -8., -6., 0., 0.),
                    horner_monic!(c, -1., -4., -7., -2., 12., 22., 14., -2., -9., -6., -2., 0.),
                    horner!(c, 5., 14., 11., -34., -93., -115., -61., -3., 30., 14., 6., -6.),
                    horner!(c, 5., 14., 13., -20., -60., -60., -12., 28., 26., 6., -4.),
                    horner!(c, -7., -3., 37., 125., 157., 109., -1., -31., -27., 9.),
                    horner!(c, -8., -10., 16., 66., 72., 18., -30., -26., -2.),
                    -horner_monic!(c, 1., 33., 85., 121., 63., 1., -33.),
                    horner!(c, 2., -12., -38., -40., -4., 20., 8.),
                    horner!(c, 9., 35., 44., 21., -14., -7.),
                    horner!(c, 4., 12., 9., -4., -5.),
                    c * (5. * c2 - 7.) - 6.,
                    c2 - 1.,
                    1. - c,
                ];
                solve_polynomial(coeffs)
            }
            _ => vec![],
        }
    }
}
impl HasDynamicalCovers for QuadRatPer3
{
    fn marked_cycle_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> (Prm, Cplx);
        let bounds: Bounds;

        match period {
            1 => {
                param_map = |t| {
                    let pole = 1.324_717_957_244_75;
                    let u = 1. / t + pole;
                    let u2 = u.powi(2);
                    let u3 = u2 * u;

                    let du = -u2.inv();

                    let num = u3 - u + 1.;
                    let dnum = 3. * u2 - 1.;
                    let den = u3 - u2 - u2 + 3. * u - 1.;
                    let dden = dnum - 4. * u + 4.;

                    (
                        (num / den).into(),
                        du * (den * dnum - num * dden) / den.powi(2),
                    )
                };
                bounds = Bounds {
                    min_x: -5.75,
                    max_x: 5.08,
                    min_y: -5.32,
                    max_y: 5.32,
                };
            }
            4 => {
                param_map = |c| {
                    let t = (13.0 as Real).sqrt();
                    let g2 = Cplx::new(-8.0 / 3.0, 0.);
                    let g3 = Cplx::new(1.0 / 27.0, 0.);

                    let (p, dp) = weierstrass_p(g2, g3, c, 0.01);
                    let x = p - 1. / 3.;
                    let y = (dp + 1.) / x - t - 1.;

                    let u = x / 2.;
                    let v = y / 4.;
                    let xx = -(t + 1.) * u + (t + 3.) * v + (t + 4.);
                    let yy = u - v - (t + 1.) / 4.;
                    let zz = -x + 2. * v + (t + 3.) / 2.;

                    let s0 = xx / zz;
                    let s1 = zz / yy;

                    // TODO: derivative
                    ((s0 * s1 + s1 + (t + 4.)).into(), ONE)
                    // let l = s0^2*s1 + s0*s1 + (2*t)*s0 + (t - 1);
                };
                bounds = Bounds {
                    min_x: -3.9,
                    max_x: 3.9,
                    min_y: -2.6,
                    max_y: 2.6,
                };
            }
            _ => {
                param_map = |t| (t.into(), ONE);
                bounds = self.point_grid.bounds.clone();
            }
        };
        CoveringMap::new(self, param_map).with_orig_bounds(bounds)
    }
}

impl InfinityFirstReturnMap for QuadRatPer3
{
    degree_impl!(2, 3);

    fn escaping_phase(&self) -> Period
    {
        2
    }

    fn escape_coeff(&self, prm: &Self::Param) -> Cplx
    {
        0.25 * (1. - prm.c.inv())
    }

    fn escape_coeff_d(&self, prm: &Self::Param) -> (Cplx, Cplx)
    {
        let u = prm.c.inv();
        (0.25 * (1. - u), 0.25 * u.powi(2))
    }

    #[inline]
    fn angle_map_large_param(&self, angle: RationalAngle) -> RationalAngle
    {
        angle + RationalAngle::ONE_HALF
    }
}

impl EscapeEncoding for QuadRatPer3
{
    basic_escape_encoding!(None, 3);
}
impl ExternalRays for QuadRatPer3
{
    ext_ray_impl_rk!(0.01, 1e6);
}

mod param
{
    use dynamo_common::prelude::*;

    #[derive(Clone, Copy, PartialEq, Debug)]
    pub struct Param
    {
        pub a: Cplx, // c^3 - c - 1
        pub b: Cplx, // -c^2
        pub c: Cplx,
    }

    impl Default for Param
    {
        fn default() -> Self
        {
            Self {
                a: -ONE,
                b: ZERO,
                c: ZERO,
            }
        }
    }

    impl From<Cplx> for Param
    {
        #[inline]
        fn from(c: Cplx) -> Self
        {
            let c2 = c.powi(2);
            let a = (c2 - 1.) * c - 1.;
            Self { a, b: -c2, c }
        }
    }

    impl From<Param> for Cplx
    {
        #[inline]
        fn from(param: Param) -> Self
        {
            param.c
        }
    }

    impl std::fmt::Display for Param
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
        {
            self.c.fmt(f)
        }
    }

    impl Describe for Param {}
    impl Named for Param
    {
        fn name(&self) -> &str
        {
            "c"
        }
    }
}
