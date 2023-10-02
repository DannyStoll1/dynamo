use fractal_common::types::Rational;
use lazy_static::lazy_static;
use regex::Regex;

/// Parse text representing an angle as done in Wolf Jung's Mandel.
/// Supports fraction strings, e.g. "17/168",
/// binary strings for dyadic angles, e.g. "011" -> 3/8,
/// and binary representations of (pre)periodic angles,
/// e.g. "011p10" -> 3/2^3 + 2/(2^3*(2^2-1)) = 11/24
pub fn parse_angle(text: &str) -> Option<Rational>
{
    parse_fraction(text)
        .or_else(|| parse_dyadic(text))
        .or_else(|| parse_preperiodic(text))
}

fn parse_fraction(text: &str) -> Option<Rational>
{
    lazy_static! {
        static ref FRACTION: Regex = Regex::new(r"^(-?\d+)/(\d+)$").unwrap();
    }

    if let Some(captures) = FRACTION.captures(&text)
    {
        if let (Some(numerator), Some(denominator)) = (captures.get(1), captures.get(2))
        {
            if let (Ok(numerator), Ok(denominator)) = (
                numerator.as_str().parse::<i32>(),
                denominator.as_str().parse::<i32>(),
            )
            {
                if denominator != 0
                {
                    return Some(Rational::new(numerator, denominator));
                }
                else
                {
                    return None;
                }
            }
        }
    }
    None
}

fn parse_dyadic(text: &str) -> Option<Rational>
{
    lazy_static! {
        static ref BIN_ANGLE: Regex = Regex::new(r"^(\d+)$").unwrap();
    }

    let Some(captures) = BIN_ANGLE.captures(&text) else {return None};
    if let Some(bin_str) = captures.get(1)
    {
        if let Ok(numerator) = i32::from_str_radix(bin_str.as_str(), 2)
        {
            let denominator = 1 << bin_str.len();
            return Some(Rational::new(numerator, denominator));
        }
    }
    None
}

fn parse_preperiodic(text: &str) -> Option<Rational>
{
    lazy_static! {
        static ref BIN_PREPER: Regex = Regex::new(r"^(\d*)p(\d+)$").unwrap();
    }

    let Some(captures) = BIN_PREPER.captures(&text) else {return None};
    if let (Some(pre_str), Some(per_str)) = (captures.get(1), captures.get(2))
    {
        if let (Ok(num_pre), Ok(num_per)) = (
            i32::from_str_radix(pre_str.as_str(), 2),
            i32::from_str_radix(per_str.as_str(), 2),
        )
        {
            let den_pre = 1 << pre_str.len();
            let den_per = den_pre * ((1 << per_str.len()) - 1);

            let pre = Rational::new(num_pre, den_pre);
            let per = Rational::new(num_per, den_per);
            return Some(pre + per);
        }
    }
    None
}
