use crate::macros::*;
use dynamo_common::cache::Cache;
use dynamo_color::prelude::*;
profile_imports!();

type EInt = EisensteinInteger;


#[derive(Clone, Debug)]
pub struct EisensteinMandel<const A: i64, const B: i64>
{
    point_grid: PointGrid,
    max_iter: Period,
    cache: Cache<(EInt, EInt), EscapeResult<EInt, EInt>>,
}

impl<const A: i64, const B: i64> EisensteinMandel<A, B>
{
    const MOD: EInt = EInt::new(A, B);
}

impl<const A: i64, const B: i64> Default for EisensteinMandel<A, B>
{
    fn default() -> Self
    {
        let bounds = Bounds::centered_square(Self::MOD.norm());
        let point_grid = PointGrid::new_by_res_y(1024, bounds);
        Self {
            point_grid,
            max_iter: 1024,
            cache: Cache::new(),
        }
    }
}

impl<const A: i64, const B: i64> ParameterPlane for EisensteinMandel<A, B>
{
    basic_plane_impl!();
    type Var = EInt;
    type Param = EInt;
    type Deriv = EInt;
    type MetaParam = NoParam;
    type Child = JuliaSet<Self>;

    fn default_bounds(&self) -> Bounds
    {
        Bounds::centered_square(Self::MOD.norm())
    }

    fn default_julia_bounds(&self, _point: Cplx, _c: Self::Param) -> Bounds
    {
        self.default_bounds()
    }

    #[inline]
    fn early_bailout(
        &self,
        start: Self::Var,
        c: Self::Param,
    ) -> Option<EscapeResult<Self::Var, Self::Deriv>>
    {
        self.cache.get(&(start, c))
    }

    fn map(&self, z: Self::Var, c: Self::Param) -> Self::Var
    {
        (z * z * z + c) % Self::MOD
    }

    fn map_and_multiplier(&self, z: Self::Var, c: Self::Param) -> (Self::Var, Self::Deriv)
    {
        ((z * z * z + c) % Self::MOD, (3 * z * z) % Self::MOD)
    }

    fn start_point(&self, _point: Cplx, _c: Self::Param) -> Self::Var
    {
        Self::Var::default()
    }

    fn name(&self) -> String
    {
        format!("Eisenstein Integer Mandelbrot mod {}", Self::MOD)
    }

    fn default_coloring(&self) -> Coloring
    {
        let mut coloring = Coloring::default();
        // coloring.get_period_coloring_mut().num_colors = Self::MOD.norm() as f32;
        coloring.get_period_coloring_mut().num_colors = 19.;
        coloring.with_interior_algorithm(IncoloringAlgorithm::Period)
    }

    fn default_coloring_child(&self) -> Coloring
    {
        self.default_coloring()
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
            fill_rate: 8.0 / Self::MOD.norm(),
        }
    }
}

impl<const A: i64, const B: i64> InfinityFirstReturnMap for EisensteinMandel<A, B>
{
    degree_impl!(2);
}

impl<const A: i64, const B: i64> EscapeEncoding for EisensteinMandel<A, B>
{
    fn encode_escape_result(&self, result: EscapeResult<EInt, EInt>, start: EInt, c: EInt) -> PointInfo<EInt>
    {
        self.cache.insert((start, c), result.clone());
        match result
        {
            EscapeResult::Bounded => PointInfo::Bounded,
            EscapeResult::Periodic {
                mut info,
                final_value,
            } =>
            {
                info.multiplier = info.multiplier % Self::MOD;
                self.identify_marked_points(final_value, c, info)
            }
            EscapeResult::KnownPotential(data) => PointInfo::PeriodicKnownPotential(data),
            EscapeResult::Escaped { iters, final_value } =>
            {
                self.encode_escaping_point(iters, final_value, c)
            }
        }
    }

    fn encode_escaping_point(&self, iters: Period, z: EInt, c: EInt) -> PointInfo<EInt>
    {
        if z.is_nan()
        {
            return PointInfo::Escaping {
                potential: Real::from(iters) - 1.,
            };
        }

        let u = self.escape_radius().log2();
        let v = z.norm_sqr().log2();
        let q = self.escape_coeff(c).norm().log2();
        let residual = ((u + q) / (v + q)).log(self.degree_real()) as IterCount;
        let potential = residual.mul_add(self.escaping_period() as IterCount, iters as IterCount);
        PointInfo::Escaping { potential }
    }
}

impl<const A: i64, const B: i64> ExternalRays for EisensteinMandel<A, B> {}
