use dynamo_common::horner;
use dynamo_common::math_utils::roots_of_unity;

use crate::macros::{degree_impl, ext_ray_impl_nonmonic, horner_monic, profile_imports};
use crate::covering_helpers::{
    degree_3_dynatomic_curve_period_2, degree_3_marked_cycle_curve_period_3,
};
profile_imports!();

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Unicorn<const D: i32>
{
    point_grid:   PointGrid,
    compute_mode: ComputeMode,
    max_iter:     IterCount,
}

impl<const D: i32> Unicorn<D>
{
    const D_FLOAT: Real = D as Real;
    const CRIT: Cplx = Cplx::new(-Self::D_FLOAT, 0.0);
    const DEFAULT_BOUNDS: Bounds =
        Bounds::square(Self::D_FLOAT * 1.2, Cplx::new(-Self::D_FLOAT + 1.0, 0.0));
}

impl<const D: i32> Default for Unicorn<D>
{
    fractal_impl!();
}

#[allow(clippy::suspicious_operation_groupings)]
impl<const D: i32> DynamicalFamily for Unicorn<D>
{
    parameter_plane_impl!();

    #[inline]
    fn map(&self, z: Self::Var, c: &Self::Param) -> Self::Var
    {
        c * (1. + z.conj() / Self::D_FLOAT).powi(D)
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv)
    {
        let u = 1. + z.conj() / Self::D_FLOAT;
        let df = c * u.powi(D - 1);
        (u * df, df)
    }

    fn gradient(&self, z: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        let u = 1. + z.conj() / Self::D_FLOAT;
        let v = u.powi(D - 1);
        let df = c * v;
        (u * df, df, u * v)
    }

    fn start_point(&self, _point: Cplx, _c: &Self::Param) -> Self::Var
    {
        ZERO
    }

    fn periodicity_tolerance(&self) -> Real
    {
        1e-18
    }

    fn name(&self) -> String
    {
        format!("Unicorn({D})")
    }
}

impl<const D: i32> FamilyDefaults for Unicorn<D>
{
    default_bounds!();

    fn default_selection(&self) -> Cplx
    {
        let zeta = (TAUI / Self::D_FLOAT).exp();
        (zeta - 1.) * Self::D_FLOAT
    }
}

impl<const D: i32> HasJulia for Unicorn<D>
{
    fn default_bounds_child(&self, _point: Cplx, _c: &Self::Param) -> Bounds
    {
        Bounds::square(Self::D_FLOAT * 1.618, Self::CRIT)
    }
}

impl<const D: i32> MarkedPoints for Unicorn<D>
{
    #[inline]
    fn critical_points_child(&self, _c: &Self::Param) -> Vec<Self::Var>
    {
        vec![Self::CRIT]
    }

    fn cycles_child(&self, c: &Self::Param, period: Period) -> Vec<Self::Var>
    {
        use dynamo_common::math_utils::binomial;
        match period {
            1 => {
                let mut coeffs: Vec<Cplx> =
                    (0..=D).map(|x| c * Real::from(binomial(D, x))).collect();
                coeffs[1] -= Self::D_FLOAT;
                solve_polynomial(coeffs)
                    .iter()
                    .map(|z| z * Self::D_FLOAT)
                    .collect()
            }
            _ => vec![],
        }
    }
}

impl<const D: i32> InfinityFirstReturnMap for Unicorn<D>
{
    #[inline]
    fn degree(&self) -> AngleNum
    {
        D.into()
    }

    #[inline]
    fn degree_real(&self) -> Real
    {
        Self::D_FLOAT
    }

    #[inline]
    fn escape_coeff(&self, c: &Self::Param) -> Cplx
    {
        c * Self::D_FLOAT.powi(-D)
    }

    #[inline]
    fn escape_coeff_d(&self, c: &Self::Param) -> (Cplx, Cplx)
    {
        let a = Self::D_FLOAT.powi(-D);
        (c * a, a.into())
    }
}

impl<const D: i32> EscapeEncoding for Unicorn<D> {}

impl<const D: i32> ExternalRays for Unicorn<D>
{
    ext_ray_impl_nonmonic!();
}

impl Unicorn<3>
{
    fn marked_cycle_curve_period_3(self) -> CoveringMap<Self>
    {
        degree_3_marked_cycle_curve_period_3(self)
    }
}

impl HasDynamicalCovers for Unicorn<3>
{
    fn marked_cycle_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> (Cplx, Cplx);
        let bounds: Bounds;

        match period {
            1 => {
                param_map = |t| (3. * (t + 1.) * t.powi(2), 3. * t * (3. * t + 2.));
                bounds = Bounds {
                    min_x: -1.8,
                    max_x: 0.9,
                    min_y: -1.2,
                    max_y: 1.2,
                };
            }
            2 => {
                param_map = |t| (3. * (t - 2.) * t.powi(2), 3. * t * (3. * t - 4.));
                bounds = Bounds {
                    min_x: -1.,
                    max_x: 2.5,
                    min_y: -1.,
                    max_y: 1.,
                };
            }
            3 => return self.marked_cycle_curve_period_3(),
            _ => {
                param_map = |t| (t, ONE);
                bounds = self.point_grid.bounds.clone();
            }
        }
        CoveringMap::new(self, param_map).with_orig_bounds(bounds)
    }

    #[allow(clippy::single_match_else, reason = "The period dispatcher stays flatter as a match while more periods remain unsupported")]
    fn dynatomic_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> (Cplx, Cplx);
        let bounds: Bounds;

        match period {
            2 => return degree_3_dynatomic_curve_period_2(self),
            _ => {
                param_map = |t| (t, ONE);
                bounds = self.point_grid.bounds.clone();
            }
        }
        CoveringMap::new(self, param_map).with_orig_bounds(bounds)
    }
}
