use crate::macros::{horner, horner_monic, profile_imports};

profile_imports!();

fn f(z: Cplx, c: Cplx) -> Cplx
{
    z * z + c
}
fn df_dz(z: Cplx, _c: Cplx) -> Cplx
{
    z + z
}

#[derive(Clone, Debug)]
pub struct Mandelbrot
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl Default for Mandelbrot
{
    fractal_impl!(-2.1, 0.55, -1.25, 1.25);
}

impl ParameterPlane for Mandelbrot
{
    parameter_plane_impl!();
    default_name!();

    fn param_map(&self, point: Cplx) -> Self::Param
    {
        point
    }

    fn escape_radius(&self) -> Real
    {
        1e26
    }

    fn start_point(&self, _point: Cplx, _c: Self::Param) -> Self::Var
    {
        ZERO
    }

    fn map(&self, z: Self::Var, c: Self::Param) -> Self::Var
    {
        f(z, c)
    }

    fn dynamical_derivative(&self, z: Self::Var, c: Self::Param) -> Self::Deriv
    {
        df_dz(z, c)
    }

    fn parameter_derivative(&self, _z: Self::Var, _c: Self::Param) -> Self::Deriv
    {
        ONE
    }

    fn early_bailout(&self, _start: Cplx, c: Self::Param) -> EscapeState<Cplx, Cplx>
    {
        // Main cardioid
        let four_c = 4. * c;
        let y2 = four_c.im * four_c.im;
        let temp = four_c.re - 1.;
        let mu_norm2 = temp.mul_add(temp, y2);
        let a = mu_norm2 * mu_norm2.mul_add(0.25, temp);

        if a < y2
        {
            let multiplier = 1. - (1. - four_c).sqrt();
            let decay_rate = multiplier.norm();
            let fixed_point = 0.5 * multiplier;
            let init_dist = (c - fixed_point).norm_sqr();
            let potential = init_dist.log(decay_rate);
            let preperiod = potential as Period;
            return EscapeState::Periodic {
                period: 1,
                preperiod,
                multiplier,
                final_error: (1e-6),
            };
        }

        // Basilica bulb
        let mu2 = four_c + 4.;
        if mu2.norm_sqr() < 1.
        {
            let decay_rate = mu2.norm();
            let fixed_point = -0.5 - 0.5 * (-four_c - 3.).sqrt();
            let init_dist = (c - fixed_point).norm_sqr();
            let potential = 2. * init_dist.log(decay_rate);
            let preperiod = potential as Period;
            return EscapeState::Periodic {
                period: 2,
                preperiod,
                multiplier: mu2,
                final_error: (1e-6),
            };
        }

        EscapeState::NotYetEscaped
    }

    #[inline]
    fn critical_points_child(&self, _param: Cplx) -> ComplexVec
    {
        vec![Cplx::new(0., 0.)]
    }

    #[inline]
    fn default_julia_bounds(&self, _point: Cplx, _param: Cplx) -> Bounds
    {
        Bounds::centered_square(2.2)
    }

    fn cycles_child(&self, c: Cplx, period: Period) -> ComplexVec
    {
        use fractal_common::math_utils::poly_solve::solve_polynomial;
        match period
        {
            1 =>
            {
                let u = (1. - 4. * c).sqrt();
                vec![0.5 * (1. + u), 0.5 * (1. - u)]
            }
            2 =>
            {
                let u = (-3. - 4. * c).sqrt();
                vec![0.5 * (-1. + u), -0.5 * (1. + u)]
            }
            3 =>
            {
                let c2 = c * c;
                let coeffs = vec![
                    1. + c + (2. + c) * c2,
                    1. + c + c + c2,
                    1. + 3. * (c + c2),
                    1. + c + c,
                    1. + 3. * c,
                    ONE,
                    ONE,
                ];
                solve_polynomial(&coeffs)
            }
            4 =>
            {
                // [[c^6 + 3*c^5 + 3*c^4 + 3*c^3 + 2*c^2 + 1, 0],
                //  [c^4 + 2*c^3 + c^2 + 2*c, 1],
                //  [6*c^5 + 12*c^4 + 6*c^3 + 5*c^2 + c, 2],
                //  [4*c^3 + 4*c^2 + 1, 3],
                //  [15*c^4 + 18*c^3 + 3*c^2 + 4*c, 4],
                //  [6*c^2 + 2*c, 5],
                //  [20*c^3 + 12*c^2 + 1, 6],
                //  [4*c, 7],
                //  [15*c^2 + 3*c, 8],
                //  [1, 9],
                //  [6*c, 10],
                //  [1, 12]]
                let c2 = c * c;
                let coeffs = vec![
                    1. + c2 * horner_monic!(c, 2., 3., 3., 3.),
                    c * horner_monic!(c, 2., 1., 2.),
                    c * horner!(c, 1., 5., 6., 12., 6.),
                    1. + 4. * c2 * (1. + c),
                    c * horner!(c, 4., 3., 18., 15.),
                    c * horner!(c, 2., 6.),
                    1. + c2 * (12. + 20. * c),
                    4. * c,
                    3. * c + 15. * c2,
                    ONE,
                    6. * c,
                    ZERO,
                    ONE,
                ];
                solve_polynomial(&coeffs)
            }
            5 =>
            {
                let v = horner_monic!(
                    c, 1., 2., 5., 14., 26., 44., 69., 94., 114., 116., 94., 60., 28., 8.
                );
                let u = 14. * c + 1.;
                let coeffs = [
                    v * c + 1.,
                    v,
                    horner!(
                        c, 1., 3., 9., 28., 66., 137., 265., 436., 642., 794., 766., 576., 316.,
                        105., 15.
                    ),
                    horner!(
                        c, 1., 4., 14., 40., 93., 196., 342., 528., 678., 672., 516., 288., 97.,
                        14.
                    ),
                    horner!(
                        c, 1., 5., 20., 67., 179., 437., 876., 1572., 2398., 2790., 2496., 1629.,
                        637., 105.
                    ),
                    horner!(
                        c, 1., 6., 27., 86., 241., 534., 1044., 1720., 2118., 1980., 1341., 540.,
                        91.
                    ),
                    horner!(
                        c, 1., 7., 35., 126., 401., 1000., 2196., 4200., 5990., 6445., 5071.,
                        2366., 455.
                    ),
                    horner!(
                        c, 1., 8., 40., 160., 466., 1152., 2480., 3872., 4465., 3730., 1826., 364.
                    ),
                    horner!(
                        c, 1., 9., 50., 221., 712., 1932., 4712., 8415., 11025., 10615., 6006.,
                        1365.
                    ),
                    horner!(c, 1., 10., 61., 246., 780., 2232., 4543., 6560., 6885., 4180., 1001.),
                    horner!(
                        c, 1., 11., 73., 324., 1116., 3527., 8113., 13140., 15741., 11011., 3003.
                    ),
                    horner!(c, 1., 12., 78., 336., 1295., 3570., 6580., 8856., 6831., 2002.),
                    horner!(c, 1., 13., 92., 427., 1779., 5467., 11172., 16962., 15015., 5005.),
                    horner!(c, 1., 14., 91., 484., 1897., 4592., 8106., 8184., 3003.),
                    horner!(c, 1., 15., 105., 598., 2565., 6822., 13398., 15444., 6435.),
                    horner!(c, 1., 14., 114., 668., 2230., 5292., 7260., 3432.),
                    horner!(c, 1., 15., 130., 815., 2970., 7722., 12012., 6435.),
                    horner!(c, 1., 16., 147., 740., 2430., 4752., 3003.),
                    horner!(c, 1., 17., 165., 900., 3190., 7007., 5005.),
                    horner!(c, 1., 18., 160., 760., 2255., 2002.),
                    horner!(c, 1., 19., 180., 913., 3003., 3003.),
                    horner!(c, 1., 20., 153., 748., 1001.),
                    horner!(c, 1., 21., 171., 910., 1365.),
                    horner!(c, 1., 18., 162., 364.),
                    horner!(c, 1., 19., 182., 455.),
                    horner!(c, 1., 20., 91.),
                    horner!(c, 1., 21., 105.),
                    u,
                    u + c,
                    ONE,
                    ONE,
                ];
                solve_polynomial(&coeffs)
            }
            _ => vec![],
        }
    }
}

impl HasDynamicalCovers for Mandelbrot
{
    fn marked_cycle_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> Cplx;
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
        let grid = self.point_grid.new_with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }

    fn dynatomic_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> Cplx;
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
        let grid = self.point_grid.new_with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }
    fn misiurewicz_curve(self, preperiod: Period, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> Cplx;
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
        let grid = self.point_grid.new_with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }
}
