use crate::normed::Semimetric;
use crate::poly_traits::{Differentiable, Eval, HasVar};

pub trait Newton: Eval + Differentiable + HasVar
where
    <Self as HasVar>::Var: Semimetric,
{
    const MAX_ITERS: usize = 16;

    fn find_root_newton(
        &self,
        start: Self::Var,
        error: <Self::Var as Semimetric>::Dist,
    ) -> Option<Self::Var>
    {
        let deriv = self.derivative();
        let mut z = start;
        let mut z_next: Self::Var;

        for _ in 0..Self::MAX_ITERS {
            z_next = z.clone() - self.eval(z.clone()) / deriv.eval(z.clone());
            if z.dist_semi(z_next.clone()) < error {
                return Some(z_next);
            }
            z = z_next;
        }
        None
    }
}
