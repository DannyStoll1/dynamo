use crate::dynamics::ParameterPlane;
use crate::point_grid::{Bounds, PointGrid};
use crate::primitive_types::{ComplexNum, EscapeState, IterCount, Period};

#[derive(Clone)]
pub struct JuliaSet<T>
where
    T: ParameterPlane + Clone,
{
    pub point_grid: PointGrid,
    pub max_iter: Period,
    pub parent: T,
    // pub map: Box<dyn Fn(ComplexNum, ComplexNum) -> ComplexNum>,
    // pub stop_condition: Box<dyn Fn(Period, ComplexNum) -> EscapeState>,
    // pub escape_encoding: Box<dyn Fn(EscapeState, ComplexNum) -> IterCount>,
    pub param: ComplexNum,
    // pub parent_params: Vec<ComplexNum>,
}

impl<T> From<T> for JuliaSet<T>
where
    T: ParameterPlane + Clone,
{
    fn from(parent: T) -> Self
    {
        let point_grid = parent
            .point_grid()
            .with_same_height(parent.default_julia_bounds());
        Self {
            point_grid,
            parent: parent.clone(),
            max_iter: parent.max_iter(),
            param: (0.).into(),
        }
    }
}

impl<T> ParameterPlane for JuliaSet<T>
where
    T: ParameterPlane + Clone,
{
    #[inline]
    fn map(&self, z: ComplexNum, _c: ComplexNum) -> ComplexNum
    {
        self.parent.map(z, self.param)
    }

    #[inline]
    fn dynamical_derivative(&self, z: ComplexNum, _c: ComplexNum) -> ComplexNum
    {
        self.parent.dynamical_derivative(z, self.param)
    }

    #[inline]
    fn parameter_derivative(&self, z: ComplexNum, _c: ComplexNum) -> ComplexNum
    {
        self.parent.parameter_derivative(z, self.param)
    }

    #[inline]
    fn gradient(&self, z: ComplexNum, _c: ComplexNum) -> (ComplexNum, ComplexNum)
    {
        self.parent.gradient(z, self.param)
    }

    #[inline]
    fn point_grid(&self) -> PointGrid
    {
        self.point_grid
    }

    #[inline]
    fn point_grid_mut(&mut self) -> &mut PointGrid
    {
        &mut self.point_grid
    }

    #[inline]
    fn max_iter(&self) -> Period
    {
        self.max_iter
    }

    #[inline]
    fn max_iter_mut(&mut self) -> &mut Period
    {
        &mut self.max_iter
    }

    #[inline]
    fn param_map(&self, z: ComplexNum) -> ComplexNum
    {
        z
    }

    #[inline]
    fn start_point(&self, z: ComplexNum) -> ComplexNum
    {
        z
    }

    #[inline]
    fn set_max_iter(&mut self, new_max_iter: Period)
    {
        self.max_iter = new_max_iter;
    }

    fn encode_periodic_point(
        &self,
        period: Period,
        preperiod: Period,
        multiplier: ComplexNum,
        final_error: ComplexNum,
    ) -> IterCount
    {
        let scaling_rate = multiplier.norm();
        let coloring_rate: f64;

        if scaling_rate > 1e-50
        {
            coloring_rate = -scaling_rate.log2() / 50.;
        }
        else
        {
            coloring_rate = 10.
        }

        let u = period as IterCount;
        let mut w = -(final_error.norm_sqr() / self.periodicity_tolerance()).log(scaling_rate)
            as IterCount;
        if w.is_infinite() || w.is_nan()
        {
            w = -0.2;
        }
        let v = preperiod as IterCount + u * w;
        // 0.02 is the internal coloring rate. Larger numbers mean faster darkening of the
        //   interiors of hyperbolic components.
        -(u + 0.99 * (v * coloring_rate / u).tanh())
    }

    fn encode_escape_result(&self, state: EscapeState, base_param: ComplexNum) -> IterCount
    {
        match state
        {
            EscapeState::NotYetEscaped => 0.,
            EscapeState::Bounded => 0.,
            EscapeState::Periodic {
                period,
                preperiod,
                multiplier,
                final_error,
            } => self.encode_periodic_point(period, preperiod, multiplier, final_error),
            EscapeState::Escaped { iters, final_value } =>
            {
                self.parent.encode_escaping_point(iters, final_value)
            }
        }
    }

    // fn encode_escape_result(&self, state: EscapeState, _base_param: ComplexNum) -> IterCount
    // {
    //     self.parent.encode_escape_result(state, self.param)
    // }

    fn stop_condition(&self, iter: Period, z: ComplexNum) -> EscapeState
    {
        self.parent.stop_condition(iter, z)
    }

    #[inline]
    fn set_param(&mut self, value: ComplexNum)
    {
        self.param = value;
    }

    #[inline]
    fn get_param(&self) -> ComplexNum
    {
        self.param
    }

    #[inline]
    fn julia_set(&self, _param: ComplexNum) -> Option<JuliaSet<Self>>
    {
        None
    }

    #[inline]
    fn default_julia_bounds(&self) -> Bounds
    {
        self.point_grid.bounds
    }

    #[inline]
    fn name(&self) -> String
    {
        "JuliaSet".to_owned()
    }
}
