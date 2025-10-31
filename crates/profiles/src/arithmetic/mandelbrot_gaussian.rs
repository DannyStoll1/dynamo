use dynamo_color::prelude::*;
use dynamo_common::cache::Cache;

use crate::macros::{basic_plane_impl, degree_impl, profile_imports};
profile_imports!();

type GInt = GaussianInteger;

#[derive(Clone, Debug)]
pub struct GaussianMandel<const A: i64, const B: i64>
{
    point_grid:   PointGrid,
    compute_mode: ComputeMode,
    max_iter:     IterCount,
    cache:        Cache<(GInt, GInt), PointInfo<GInt>>,
}

impl<const A: i64, const B: i64> GaussianMandel<A, B>
{
    const MOD: GInt = GInt::new(A, B);
}

impl<const A: i64, const B: i64> Default for GaussianMandel<A, B>
{
    fn default() -> Self
    {
        let bounds = Bounds::square(Self::MOD.norm() / 2.0, Cplx::from(Self::MOD) / 2.0);
        let point_grid = PointGrid::new_by_res_y(1024, bounds);
        Self {
            point_grid,
            compute_mode: ComputeMode::default(),
            max_iter: 1024,
            cache: Cache::new(),
        }
    }
}

impl<const A: i64, const B: i64> DynamicalFamily for GaussianMandel<A, B>
{
    basic_plane_impl!();
    type Var = GInt;
    type Param = GInt;
    type Deriv = GInt;
    type MetaParam = NoParam;

    #[inline]
    fn early_bailout(&self, start: Self::Var, c: &Self::Param) -> Option<PointInfo<GInt>>
    {
        self.cache.get(&(start, *c))
    }

    fn map(&self, z: Self::Var, c: &Self::Param) -> Self::Var
    {
        (z * z + *c) % Self::MOD
    }

    fn map_and_multiplier(&self, z: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv)
    {
        ((z * z + *c) % Self::MOD, (2 * z) % Self::MOD)
    }

    fn start_point(&self, _point: Cplx, _c: &Self::Param) -> Self::Var
    {
        Self::Var::default()
    }

    fn name(&self) -> String
    {
        format!("Gaussian Integer Mandelbrot mod {}", Self::MOD)
    }

    fn preperiod_coloring(&self) -> IncoloringAlgorithm
    {
        IncoloringAlgorithm::PreperiodPeriod {
            fill_rate: 8.0 / Self::MOD.norm(),
        }
    }

    fn potential_and_period_coloring(&self) -> IncoloringAlgorithm
    {
        IncoloringAlgorithm::PotentialAndPeriod {
            periodicity_tolerance: self.periodicity_tolerance(),
            crit_degree: 2.0,
            fill_rate: 8.0 / Self::MOD.norm(),
        }
    }

    #[inline]
    fn param_map(&self, point: Cplx) -> Self::Param
    {
        point.into()
    }
}

impl<const A: i64, const B: i64> FamilyDefaults for GaussianMandel<A, B>
{
    fn default_bounds(&self) -> Bounds
    {
        Bounds::square(Self::MOD.norm() / 2.0, Cplx::from(Self::MOD) / 2.0)
    }

    fn default_coloring(&self) -> Coloring
    {
        let mut coloring = Coloring::default();
        coloring.get_period_coloring_mut().num_colors = Self::MOD.norm() as f32;
        coloring.with_interior_algorithm(IncoloringAlgorithm::Period)
    }
}

impl<const A: i64, const B: i64> HasJulia for GaussianMandel<A, B>
{
    fn default_bounds_child(&self, _point: Cplx, _c: &Self::Param) -> Bounds
    {
        self.default_bounds()
    }

    fn default_coloring_child(&self) -> Coloring
    {
        self.default_coloring()
    }
}

impl<const A: i64, const B: i64> InfinityFirstReturnMap for GaussianMandel<A, B>
{
    degree_impl!(2);
}

impl<const A: i64, const B: i64> MarkedPoints for GaussianMandel<A, B> {}

impl<const A: i64, const B: i64> EscapeEncoding for GaussianMandel<A, B>
{
    fn encode_escape_result(
        &self,
        result: EscapeResult<GInt, GInt>,
        start: GInt,
        c: &GInt,
    ) -> PointInfo<GInt>
    {
        let info = match result {
            EscapeResult::Periodic {
                mut info,
                final_value,
            } => {
                info.multiplier = info.multiplier % Self::MOD;
                self.identify_marked_points(final_value, c, info)
            }
            EscapeResult::Bounded(_) => PointInfo::Bounded,
            EscapeResult::Escaped { iters, final_value } => {
                self.encode_escaping_point(iters, final_value, c)
            }
            EscapeResult::Unknown => PointInfo::Unknown,
        };
        self.cache.insert((start, *c), info.clone());
        info
    }

    fn encode_escaping_point(&self, iters: IterCount, z: GInt, c: &GInt) -> PointInfo<GInt>
    {
        if z.is_nan() {
            return PointInfo::Escaping {
                potential: (iters - 1) as IterCountSmooth,
                phase:     None,
            };
        }

        let u = self.escape_radius().log2();
        let v = z.norm_sqr().log2();
        let q = self.escape_coeff(c).norm().log2();
        let residual = ((u + q) / (v + q)).log(self.degree_real()) as IterCountSmooth;
        let potential = residual.mul_add(
            IterCountSmooth::from(self.escaping_period()),
            iters as IterCountSmooth,
        );
        PointInfo::Escaping {
            potential,
            phase: None,
        }
    }
}

impl<const A: i64, const B: i64> ExternalRays for GaussianMandel<A, B> {}
