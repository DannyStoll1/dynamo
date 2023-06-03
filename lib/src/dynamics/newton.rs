use super::ParameterPlane;
use crate::types::*;

trait NewtonPlane: ParameterPlane<Var = ComplexNum, Deriv = ComplexNum>
{
    fn second_dynamical_derivative(&self, z: Self::Var, c: Self::Param) -> Self::Deriv;

    #[inline]
    fn map_dmap_d2map(&self, z: Self::Var, c: Self::Param)
        -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        let (f, df) = self.map_and_multiplier(z, c);
        let d2f = self.second_dynamical_derivative(z, c);
        (f, df, d2f)
    }

    #[inline]
    fn newton_map(&self, z: Self::Var, c: Self::Param) -> Self::Var
    {
        let (f, df) = self.map_and_multiplier(z, c);
        z - f / df
    }

    #[inline]
    fn newton_map_and_multiplier(&self, z: Self::Var, c: Self::Param) -> (Self::Var, Self::Deriv)
    {
        let (f, df, d2f) = self.map_dmap_d2map(z, c);
        let correction = f / df;
        (z - correction, d2f * correction / df)
    }
}
