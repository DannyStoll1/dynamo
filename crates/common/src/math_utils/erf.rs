use crate::consts::{ERF_CHEB_POLY, ISQRT_PI};
use crate::macros::horner;
use crate::types::*;

fn erfcx_y100(y100: Real) -> Real
{
    // Steven G. Johnson, October 2012.

    // Given y100=100*y, where y = 4/(4+x) for x >= 0, compute erfc(x).

    // Uses a look-up table of 100 different Chebyshev polynomials
    // for y intervals [0,0.01], [0.01,0.02], ...., [0.99,1], generated
    // with the help of Maple and a little shell script.  This allows
    // the Chebyshev polynomials to be of significantly lower degree (about 1/4)
    // compared to fitting the whole [0,1] interval with a single polynomial.

    let iy = y100 as i32;

    if (iy >= 0) && (iy < 100)
    {
        let t = y100 + y100 - (1 + 2 * iy) as Real;
        let lut = ERF_CHEB_POLY[iy as usize];
        return horner!(t, lut[0], lut[1], lut[2], lut[3], lut[4], lut[5], lut[6]);
    }

    // fall through if |x| < 4*eps, hence y = 1
    return 1.0; // correct within 1e-15
}

pub fn erf_faddeeva(x: Real) -> Real
{
    // Steven G. Johnson, October 2012.

    // This function combines a few different ideas.

    // First, for x > 50, it uses a continued-fraction expansion (same as
    // for the Faddeeva function, but with algebraic simplifications for z=i*x).

    // Second, for 0 <= x <= 50, it uses Chebyshev polynomial approximations,
    // but with two twists:
    //
    // a) It maps x to y = 4 / (4+x) in [0,1].  This simple transformation,
    // inspired by a similar transformation in the octave-forge/specfun
    // erfcx by Soren Hauberg, results in much faster Chebyshev convergence
    // than other simple transformations I have examined.
    //
    // b) Instead of using a single Chebyshev polynomial for the entire
    // [0,1] y interval, we break the interval up into 100 equal
    // subintervals, with a switch/lookup table, and use much lower
    // degree Chebyshev polynomials in each subinterval. This greatly
    // improves performance in my tests.
    //
    // For x < 0, we use the relationship erfcx(-x) = 2 exp(x^2) - erfc(x),
    // with the usual checks for overflow etcetera.

    // Performance-wise, it seems to be substantially faster than either
    // the SLATEC DERFC function [or an erfcx function derived therefrom]
    // or Cody's CALERF function (from netlib.org/specfun), while
    // retaining near machine precision in accuracy.

    if x >= 0.
    {
        if x > 50.
        {
            // continued-fraction expansion is faster
            if x > 5e7
            {
                // 1-term expansion, important to avoid overflow
                return ISQRT_PI / x;
            }
            /* 5-term expansion (rely on compiler for CSE), simplified from:
            ispi / (x+0.5/(x+1/(x+1.5/(x+2/x))))  */
            let x2 = x * x;
            return ISQRT_PI * ((x2) * (x2 + 4.5) + 2.) / (x * ((x2) * (x2 + 5.) + 3.75));
        }
        return erfcx_y100(400. / (4. + x));
    }
    else
    {
        if x < 26.7
        {
            Real::INFINITY
        }
        else
        {
            let u = (x * x).exp();
            if x < -6.1
            {
                u + u
            }
            else
            {
                u + u - erfcx_y100(400. / (4. - x))
            }
        }
    }
}
