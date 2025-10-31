pub use num::Integer;
pub use num::integer::gcd;

use crate::types::{Period, SignedPeriod};

#[must_use]
pub const fn div_rem(a: Period, b: Period) -> Option<(Period, Period)>
{
    if b == 0 { None } else { Some((a / b, a % b)) }
}

pub fn divisors(n: Period) -> impl Iterator<Item = Period>
{
    (1..).take_while(move |&x| x * x <= n).flat_map(move |x| {
        if n.is_multiple_of(x) {
            if x * x == n {
                vec![x].into_iter()
            } else {
                vec![x, n / x].into_iter()
            }
        } else {
            vec![].into_iter()
        }
    })
}

#[must_use]
pub fn euler_totient(n: Period) -> Period
{
    (1..=n).filter(|&x| gcd(x, n) == 1).count() as Period
}

#[must_use]
pub const fn moebius(n: Period) -> SignedPeriod
{
    if n == 1 {
        return 1;
    }
    let mut result = 1;
    let mut n = n;
    let mut i = 2;
    while i * i <= n {
        if n.is_multiple_of(i) {
            result = -result;
            n /= i;
            if n.is_multiple_of(i) {
                return 0;
            }
        }
        i += 1;
    }
    if n > 1 {
        result = -result;
    }
    result
}

fn dirichlet_convolution<F, G>(f: F, g: G, n: Period) -> SignedPeriod
where
    F: Fn(Period) -> SignedPeriod,
    G: Fn(Period) -> SignedPeriod,
{
    divisors(n).map(|d| f(d) * g(n / d)).sum()
}

fn filtered_dirichlet_convolution<F, G, H>(f: F, g: G, n: Period, filter_fn: H) -> SignedPeriod
where
    F: Fn(Period) -> SignedPeriod,
    G: Fn(Period) -> SignedPeriod,
    H: FnMut(&Period) -> bool,
{
    divisors(n).filter(filter_fn).map(|d| f(d) * g(n / d)).sum()
}

fn moebius_inversion<F>(f: F, n: Period) -> SignedPeriod
where
    F: Fn(Period) -> SignedPeriod,
{
    dirichlet_convolution(moebius, f, n)
}
