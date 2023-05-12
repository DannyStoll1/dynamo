use crate::types::ComplexNum;
use fractal_derive::FractalProfile;

#[derive(Clone, Debug, FractalProfile)]
#[fractal_params(
    min_x = -4.1,
    max_x = 4.55,
    min_y = -4.25,
    max_y = 4.25,
    df_dz = {
        let x = z.re;
        let _y = z.im;
        let v = x.mul_add(x, 1.);
        ComplexNum::new(
            -2.*c.re*x/(v*v),
            -c.im
        )
    },
    df_dc = ONE_COMPLEX,
    map = {
        let x = z.re;
        let y = z.im;
        ComplexNum::new(
            c.re/x.mul_add(x, 1.) + y,
            c.im.mul_add(-x - 1., y)
        )
    },
    plane_methods =
        #[inline]
        fn start_point(&self, param: ComplexNum) -> ComplexNum
        {
            let mut z = ComplexNum::new(0.5, 1.5);
            for _ in 0..10000 {
                z = self.map(z, param);
            }
            z
        }
)]
pub struct Rulkov
{
    point_grid: PointGrid,
    max_iter: Period,
}
