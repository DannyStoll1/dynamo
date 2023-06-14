use crate::{
    macros::*,
    math_utils::{roots_of_unity, solve_quadratic},
};
profile_imports!();

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Unicritical<const D: i32>
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl<const D: i32> Unicritical<D>
{
    const D_FLOAT: Real = D as Real;
    const CRIT: Cplx = Cplx::new(-Self::D_FLOAT, 0.0);
    const DEFAULT_BOUNDS: Bounds =
        Bounds::square(-Self::D_FLOAT * 1.2, Cplx::new(-Self::D_FLOAT + 1.0, 0.0));
}

impl<const D: i32> Default for Unicritical<D>
{
    fractal_impl!();
}

impl<const D: i32> ParameterPlane for Unicritical<D>
{
    parameter_plane_impl!();
    basic_escape_encoding!(Self::D_FLOAT, 1.);

    fn map(&self, z: Self::Var, c: Self::Param) -> Self::Var
    {
        c * (1. + z / Self::D_FLOAT).powi(D)
    }

    fn dynamical_derivative(&self, z: Self::Var, c: Self::Param) -> Self::Deriv
    {
        c * (1. + z / Self::D_FLOAT).powi(D - 1)
    }

    fn parameter_derivative(&self, _z: Self::Var, _c: Self::Param) -> Self::Deriv
    {
        ONE
    }

    fn start_point(&self, _point: Cplx, _c: Self::Param) -> Self::Var
    {
        ZERO
    }

    fn critical_points_child(&self, _c: Self::Param) -> Vec<Self::Var>
    {
        vec![Self::CRIT]
    }

    fn default_julia_bounds(&self, _point: Cplx, _c: Self::Param) -> Bounds
    {
        Bounds::square(Self::D_FLOAT * 1.618, Self::CRIT)
    }

    fn default_selection(&self) -> Cplx
    {
        let zeta = (TAUI / Self::D_FLOAT).exp();
        (zeta - 1.) * Self::D_FLOAT
    }

    fn name(&self) -> String
    {
        format!("Unicritical({})", D)
    }
}

const U3_MC_3_DEN_0_0: Cplx = Cplx::new(
    15.019639247721374096483891512291213301,
    48.282356214136125505791181322357303256,
);
const U3_MC_3_DEN_0_1: Cplx = Cplx::new(
    11.411649536823681903898210262013797027,
    8.4252528735805598634775380564467266474,
);
const U3_MC_3_DEN_1_0: Cplx = Cplx::new(
    14.056957561484392537067372953583571855,
    50.196352118588645802062628004790145640,
);
const U3_MC_3_DEN_1_1: Cplx = Cplx::new(
    11.541242409948602031084332383773389961,
    8.7081257821104883767887704456728548666,
);
const U3_MC_3_NUM_0: Cplx = Cplx::new(
    -1744.4085890137323522786251794517118838,
    1473.7406022924868333263830519688068733,
);
const U3_MC_3_NUM_1: Cplx = Cplx::new(
    -343.84995190007797545558612484001381645,
    1273.1006905106409712731472362996230616,
);
const U3_MC_3_NUM_2: Cplx = Cplx::new(
    96.708321954359615099287578681161821316,
    269.23872202836427878674659230859595175,
);
const U3_MC_3_NUM_3: Cplx = Cplx::new(
    22.564029832221341871706338236042215261,
    15.916865188953581664230580625630756331,
);

const U3_MC_3_CONST: Cplx = Cplx::new(
    -2.2165312633441736636979371216505676846,
    0.38892825772801723043367655111187170378,
);

const U3_MC_3_POLE_0: Cplx = Cplx::new(
    -5.9140152052732332793003934321513703506,
    -3.7098663410743973546667039624491877621,
);

const U3_MC_3_POLE_1: Cplx = Cplx::new(
    -5.4976343315504486245978168298624266761,
    -4.7153865325061625088108340939975388853,
);

const U3_MC_3_ANGLE: Cplx = Cplx::new(0.5 * SQRT_3, -0.5);

// const U3_MC_3_POLE_0: Cplx = Cplx::new(
//     -6.3071559227053154449928460559449771172,
//     -4.4052647736416225259941095453003318476,
// );
//
// const U3_MC_3_POLE_1: Cplx = Cplx::new(
//     -5.2340864872432865860914863278284128442,
//     -4.3028610084688658507946609003725230190,
// );

impl HasDynamicalCovers for Unicritical<3>
{
    fn marked_cycle_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> Cplx;
        let bounds: Bounds;

        match period
        {
            1 =>
            {
                param_map = |t| 3. * (t + 1.) * t * t;
                bounds = Bounds {
                    min_x: -1.8,
                    max_x: 0.9,
                    min_y: -1.2,
                    max_y: 1.2,
                };
            }
            2 =>
            {
                param_map = |t| 3. * (t - 2.) * t * t;
                bounds = Bounds {
                    min_x: -1.,
                    max_x: 2.5,
                    min_y: -1.,
                    max_y: 1.,
                };
            }
            3 =>
            {
                param_map = |t| {
                    let t = t * U3_MC_3_ANGLE;
                    let t = (U3_MC_3_POLE_1 * t + U3_MC_3_POLE_0) / (t + 1.);
                    let num0 = horner_monic!(
                        t,
                        U3_MC_3_NUM_0,
                        U3_MC_3_NUM_1,
                        U3_MC_3_NUM_2,
                        U3_MC_3_NUM_3
                    );
                    let den0 = horner_monic!(t, U3_MC_3_DEN_0_0, U3_MC_3_DEN_0_1);
                    let den1 = horner_monic!(t, U3_MC_3_DEN_1_0, U3_MC_3_DEN_1_1);
                    U3_MC_3_CONST * num0 * num0 / (den0 * den0 * den0 * den1)
                };
                bounds = Bounds {
                    min_x: -2.,
                    max_x: 5.8,
                    min_y: -2.,
                    max_y: 3.5,
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

    fn dynatomic_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> Cplx;
        let bounds: Bounds;

        match period
        {
            2 =>
            {
                param_map = |t| {
                    let t2 = t * t;
                    let num0 = t - 0.5;
                    let num1 = t2 + t + 1.25;
                    let den0 = t2 + 0.75;
                    let num01 = num0 * num1;
                    -3. * num01 * num01 / (den0 * den0 * den0)
                };
                bounds = Bounds {
                    min_x: -3.5,
                    max_x: 5.5,
                    min_y: -4.5,
                    max_y: 4.5,
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
}
