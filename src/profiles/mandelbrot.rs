use crate::dynamics::covering_maps::{CoveringMap, HasDynamicalCovers};
use fractal_derive::FractalProfile;

#[derive(Clone, Debug, FractalProfile)]
#[fractal_params(
    map = z * z + c,
    min_x = -2.1,
    max_x = 0.55,
    min_y = -1.25,
    max_y = 1.25,
    df_dz = z+z,
    df_dc = ONE_COMPLEX,
    plane_methods =
        fn early_bailout(&self, _start: ComplexNum, param: ComplexNum) -> EscapeState
        {
            // Main cardioid
            let four_c = 4. * param;
            let y2 = four_c.im * four_c.im;
            let temp = four_c.re - 1.;
            let mu_norm2 = temp * temp + y2;
            let a = mu_norm2 * (mu_norm2 * 0.25 + temp);

            if a < y2
            {
                let multiplier = 1. - (1. - four_c).sqrt();
                let decay_rate = multiplier.norm();
                let fixed_point = 0.5 * multiplier;
                let init_dist = (param - fixed_point).norm_sqr();
                let potential = init_dist.log(decay_rate);
                let preperiod = potential as Period;
                return EscapeState::Periodic {
                    period: 1,
                    preperiod,
                    multiplier,
                    final_error: (1e-6).into(),
                };
            }

            // Basilica bulb
            let mu2 = four_c + 4.;
            if mu2.norm_sqr() < 1.
            {
                let decay_rate = mu2.norm();
                let fixed_point = -0.5 - 0.5 * (-four_c - 3.).sqrt();
                let init_dist = (param - fixed_point).norm_sqr();
                let potential = 2. * init_dist.log(decay_rate);
                let preperiod = potential as Period;
                return EscapeState::Periodic {
                    period: 2,
                    preperiod,
                    multiplier: mu2,
                    final_error: (1e-6).into(),
                };
            }

            EscapeState::NotYetEscaped
        }

        #[inline]
        fn default_julia_bounds(&self, _param: ComplexNum) -> Bounds
        {
            Bounds::centered_square(2.2)
        }
)]
pub struct Mandelbrot
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl HasDynamicalCovers for Mandelbrot
{
    fn marked_cycle_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(ComplexNum) -> ComplexNum;
        let bounds: Bounds;

        match period
        {
            1 =>
            {
                param_map = |c| 0.25 - c * c;
                bounds = Bounds {
                    min_x: -1.8,
                    max_x: 1.8,
                    min_y: -1.0,
                    max_y: 1.0,
                };
            }
            3 =>
            {
                param_map = |c| -1.75 * (1. + 7. * c * c);
                bounds = Bounds {
                    min_x: -0.3,
                    max_x: 0.3,
                    min_y: -0.5,
                    max_y: 0.5,
                };
            }
            4 =>
            {
                param_map = |c| {
                    let u = c * c;
                    -0.25 * u - 0.75 - 1. / c
                };
                bounds = Bounds {
                    min_x: -2.9,
                    max_x: 2.1,
                    min_y: -3.1,
                    max_y: 3.1,
                };
                // bounds = Bounds {
                //     min_x: -1.029809,
                //     max_x: -1.029387,
                //     min_y: -0.682203,
                //     max_y: -0.681675,
                // };
            }
            _ =>
            {
                param_map = |c| c;
                bounds = self.point_grid.bounds.clone();
            }
        };
        let grid = self.point_grid.with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }

    fn dynatomic_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(ComplexNum) -> ComplexNum;
        let bounds: Bounds;

        match period
        {
            1 =>
            {
                param_map = |c| 0.25 - c * c;
                bounds = Bounds {
                    min_x: -1.8,
                    max_x: 1.8,
                    min_y: -1.0,
                    max_y: 1.0,
                };
            }
            3 =>
            {
                param_map = |c| {
                    let c2 = c * c;
                    let v = c2 * (c2 - 3. * c + 6.) - c - c + 2.;
                    let u = v + 1. / (c2 - c);
                    -0.25 * u / (c2 - c)
                };
                bounds = Bounds {
                    min_x: -2.5,
                    max_x: 3.5,
                    min_y: -3.,
                    max_y: 3.,
                };
            }
            _ =>
            {
                param_map = |c| c;
                bounds = self.point_grid.bounds.clone();
            }
        };
        let grid = self.point_grid.with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }
    fn misiurewicz_curve(self, preperiod: Period, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(ComplexNum) -> ComplexNum;
        let bounds: Bounds;

        match (preperiod, period)
        {
            (2, 1) =>
            {
                param_map = |c| {
                    let c2 = c * c;
                    -2. * (c2 + 1.) / ((c2 - 1.) * (c2 - 1.))
                };
                bounds = Bounds {
                    min_x: -3.5,
                    max_x: 3.5,
                    min_y: -3.0,
                    max_y: 3.0,
                };
            }
            (2, 2) =>
            {
                param_map = |c| {
                    let c2 = c * c;
                    -(c2 * (c2 + c + c + 2.) - c - c + 1.) / (4. * c2)
                };
                bounds = Bounds {
                    min_x: -4.,
                    max_x: 2.4,
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
        let grid = self.point_grid.with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }
}