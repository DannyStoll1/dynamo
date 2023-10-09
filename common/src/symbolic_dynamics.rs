use lazy_static::lazy_static;
use regex::Regex;
use std::{error::Error, num::ParseIntError, str::FromStr};

use crate::prelude::*;
use derive_more::{From, Into};
use num_traits::sign::Signed;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OrbitSchema
{
    pub period: Period,
    pub preperiod: Period,
}

impl From<Period> for OrbitSchema
{
    fn from(period: Period) -> Self
    {
        Self {
            period,
            preperiod: 0,
        }
    }
}

impl FromStr for OrbitSchema
{
    type Err = ParseOrbitSchemaError;

    /// Parse text representing a period and preperiod into an OrbitSchema.
    /// Acceptable input formats: <period> or <period, preperiod>
    #[allow(clippy::unwrap_used)]
    fn from_str(text: &str) -> Result<Self, Self::Err>
    {
        use ParseOrbitSchemaError as Err;
        lazy_static! {
            static ref PREPERIOD: Regex = Regex::new(r"^\s*(\d+)\s*,\s*(\d+)\s*$").unwrap();
        }

        if let Some(captures) = PREPERIOD.captures(text)
        {
            let preperiod = captures
                .get(1)
                .ok_or(Err::UnrecognizedFormat)?
                .as_str()
                .parse()
                .map_err(Err::Preperiodic)?;
            let period = captures
                .get(2)
                .ok_or(Err::UnrecognizedFormat)?
                .as_str()
                .parse()
                .map_err(Err::Preperiodic)?;
            Ok(Self { preperiod, period })
        }
        else
        {
            text.parse::<Period>()
                .map(|p| p.into())
                .map_err(Err::Periodic)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KneadingSymbol
{
    Interior(usize),
    Boundary(usize),
}
impl KneadingSymbol
{
    fn to_string(self, partition_size: usize) -> String
    {
        match self
        {
            Self::Interior(x) => format!("{}", x),
            Self::Boundary(x) => format!("[{}|{}]", x, (x + 1) % partition_size),
        }
    }
    fn to_string_kneading(self) -> String
    {
        match self
        {
            Self::Interior(x) => format!("{}", x),
            Self::Boundary(_) => "*".to_owned(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Itinerary
{
    partition: CirclePartition,
    pub preperiodic_part: Vec<KneadingSymbol>,
    pub periodic_part: Vec<KneadingSymbol>,
}
impl Itinerary
{
    pub fn orbit_schema(&self) -> OrbitSchema
    {
        OrbitSchema {
            preperiod: self.preperiodic_part.len() as Period,
            period: self.periodic_part.len() as Period,
        }
    }
}
impl std::fmt::Display for Itinerary
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        let pre_str = self
            .preperiodic_part
            .iter()
            .map(|x| x.to_string_kneading())
            .collect::<Vec<String>>()
            .join("");
        let per_str = self
            .periodic_part
            .iter()
            .map(|x| x.to_string_kneading())
            .collect::<Vec<String>>()
            .join("");
        write!(f, "{}p{}", pre_str, per_str)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CirclePartition
{
    angles: Vec<RationalAngle>,
}
impl CirclePartition
{
    #[must_use]
    pub fn new(mut angles: Vec<RationalAngle>) -> Self
    {
        angles.sort();
        Self { angles }
    }

    /// Assumes angles are sorted
    #[must_use]
    pub fn new_raw(angles: Vec<RationalAngle>) -> Self
    {
        Self { angles }
    }

    pub fn identify(&self, theta: RationalAngle) -> KneadingSymbol
    {
        use std::cmp::Ordering;
        for (i, x) in self.angles.iter().enumerate()
        {
            match theta.cmp(x)
            {
                Ordering::Less => return KneadingSymbol::Interior(i),
                Ordering::Equal => return KneadingSymbol::Boundary(i),
                _ =>
                {}
            }
        }
        KneadingSymbol::Interior(0)
    }

    pub fn is_circularly_ordered(&self) -> bool
    {
        let Some((imin, xmin)) = self.angles.iter().enumerate().min_by_key(|(_i, x)|*x)
        else {
            return true
        };
        let mut x_curr = *xmin;
        for i in (imin + 1)..self.angles.len()
        {
            if self.angles[i] < x_curr
            {
                return false;
            }
            x_curr = self.angles[i]
        }
        for i in 0..imin
        {
            if self.angles[i] < x_curr
            {
                return false;
            }
            x_curr = self.angles[i]
        }
        true
    }

    pub fn size(&self) -> usize
    {
        self.angles.len()
    }
}

/// Wrapper class for num_rational::Rational that performs arithmetic mod 1
#[derive(Clone, Copy, Debug, Hash, From, Into, PartialEq, Eq, PartialOrd, Ord)]
pub struct RationalAngle(Rational);

impl RationalAngle
{
    const ONE_HALF: Self = Self::new_raw(1, 2);

    #[must_use]
    pub fn new(numer: AngleNum, denom: AngleNum) -> Self
    {
        let rational = Rational::new(numer, denom);
        Self(rational).mod_1()
    }

    /// Creates a RationalAngle without checking for zero division, reducing, or projecting mod 1
    #[must_use]
    pub const fn new_raw(numer: AngleNum, denom: AngleNum) -> Self
    {
        Self(Rational::new_raw(numer, denom))
    }

    /// Preperiod and period under the angle tupling map of a given degree
    pub fn orbit_schema(&self, degree: AngleNum) -> OrbitSchema
    {
        let degree2 = degree * degree;
        let mut slow = *self;
        let mut fast = *self;

        loop
        {
            slow *= degree;
            fast *= degree2;
            if fast == slow
            {
                break;
            }
        }

        let mut period = 1;
        slow *= degree;

        while slow != fast
        {
            slow *= degree;
            period += 1;
        }

        let mut preperiod = 0;
        slow = *self;
        while slow != fast
        {
            slow *= degree;
            fast *= degree;
            preperiod += 1;
        }

        OrbitSchema { preperiod, period }
    }

    pub fn itinerary(&self, degree: AngleNum, partition: CirclePartition) -> Itinerary
    {
        let orbit_schema = self.orbit_schema(degree);

        let mut x = *self;

        let preperiodic_part = (0..orbit_schema.preperiod)
            .map(|_i| {
                let id = partition.identify(x);
                x *= degree;
                id
            })
            .collect();

        let periodic_part = (0..orbit_schema.period)
            .map(|_i| {
                let id = partition.identify(x);
                x *= degree;
                id
            })
            .collect();

        Itinerary {
            partition,
            preperiodic_part,
            periodic_part,
        }
    }

    pub fn kneading_sequence(&self, degree: AngleNum) -> Itinerary
    {
        let theta_over_n = *self / degree;
        let partition_angles = (0..degree)
            .map(|x| theta_over_n + Self::new_raw(x, degree))
            .collect();
        let partition = CirclePartition::new_raw(partition_angles);
        self.itinerary(degree, partition)
    }

    fn mod_1(mut self) -> Self
    {
        self.0 = self.0.fract();
        if self.0.is_negative()
        {
            self.0 += 1;
        }
        self
    }

    #[inline]
    fn reduce_mod_1(&mut self)
    {
        self.0 = self.0.fract();
        if *self.0.numer() < 0
        {
            self.0 += 1;
        }
    }

    fn mod_1_unchecked(mut self) -> Self
    {
        self.0 = self.0.fract();
        self
    }

    fn to_complex(self) -> Cplx
    {
        (TAUI * Real::from(self)).exp()
    }
}

impl std::ops::Add for RationalAngle
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output
    {
        Self(self.0 + rhs.0).mod_1()
    }
}

impl std::ops::Sub for RationalAngle
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output
    {
        Self(self.0 - rhs.0).mod_1()
    }
}

impl std::ops::Neg for RationalAngle
{
    type Output = Self;

    fn neg(self) -> Self::Output
    {
        let rational = Rational::new(self.0.denom() - self.0.numer(), *self.0.denom());
        Self(rational)
    }
}

impl std::ops::Mul for RationalAngle
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output
    {
        Self(self.0 * rhs.0).mod_1()
    }
}

impl std::ops::Mul<AngleNum> for RationalAngle
{
    type Output = Self;

    fn mul(self, rhs: AngleNum) -> Self
    {
        Self::new(*self.0.numer() * rhs, *self.0.denom())
    }
}

impl std::ops::MulAssign<AngleNum> for RationalAngle
{
    fn mul_assign(&mut self, rhs: AngleNum)
    {
        self.0 *= rhs;
        self.reduce_mod_1();
    }
}

impl std::ops::Div for RationalAngle
{
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output
    {
        Self(self.0 / rhs.0).mod_1()
    }
}

impl std::ops::Div<AngleNum> for RationalAngle
{
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn div(self, rhs: AngleNum) -> Self
    {
        Self::new(*self.0.numer(), *self.0.denom() * rhs)
    }
}

impl std::fmt::Display for RationalAngle
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "{}/{}", self.0.numer(), self.0.denom())
    }
}

impl From<RationalAngle> for Real
{
    fn from(angle: RationalAngle) -> Self
    {
        (*angle.0.numer() as Self) / (*angle.0.denom() as Self)
    }
}

#[derive(Debug)]
pub struct DivisionByZeroError;

impl std::fmt::Display for DivisionByZeroError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "Division by zero occurred.")
    }
}

impl std::error::Error for DivisionByZeroError {}

#[derive(Debug)]
pub enum ParseAngleError
{
    UnrecognizedFormat,
    Fraction(Box<dyn Error>),
    Dyadic(Box<dyn Error>),
    Preperiodic(Box<dyn Error>),
}

impl std::fmt::Display for ParseAngleError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self
        {
            Self::UnrecognizedFormat => write!(f, "Unrecognized angle format."),
            Self::Fraction(cause) =>
            {
                write!(f, "Error parsing fraction string: {}", cause)
            }
            Self::Dyadic(cause) => write!(f, "Error parsing dyadic string: {}", cause),
            Self::Preperiodic(cause) =>
            {
                write!(f, "Error parsing preperiodic string: {}", cause)
            }
        }
    }
}

impl std::error::Error for ParseAngleError
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)>
    {
        match self
        {
            Self::UnrecognizedFormat => None,
            Self::Fraction(cause) => Some(&**cause),
            Self::Dyadic(cause) => Some(&**cause),
            Self::Preperiodic(cause) => Some(&**cause),
        }
    }
}

pub fn parse_angle(text: &str) -> Result<RationalAngle, ParseAngleError>
{
    parse_fraction(text)
        .or_else(|| parse_dyadic(text))
        .or_else(|| parse_preperiodic(text))
        .unwrap_or(Err(ParseAngleError::UnrecognizedFormat))
}

#[allow(clippy::unwrap_used)]
fn parse_fraction(text: &str) -> Option<Result<RationalAngle, ParseAngleError>>
{
    lazy_static! {
        static ref FRACTION: Regex = Regex::new(r"^(-?\d+)/(\d+)$").unwrap();
    }

    if let Some(captures) = FRACTION.captures(text)
    {
        if let (Some(num_str), Some(den_str)) = (captures.get(1), captures.get(2))
        {
            match (
                num_str.as_str().parse::<AngleNum>(),
                den_str.as_str().parse::<AngleNum>(),
            )
            {
                (Ok(numer), Ok(denom)) =>
                {
                    if denom != 0
                    {
                        return Some(Ok(RationalAngle::new(numer, denom)));
                    }
                    else
                    {
                        return Some(Err(ParseAngleError::Fraction(Box::new(
                            DivisionByZeroError,
                        ))));
                    }
                }
                (Err(e), _) | (_, Err(e)) =>
                {
                    return Some(Err(ParseAngleError::Fraction(Box::new(e))));
                }
            }
        }
    }
    None
}

#[allow(clippy::unwrap_used)]
fn parse_dyadic(text: &str) -> Option<Result<RationalAngle, ParseAngleError>>
{
    lazy_static! {
        static ref BIN_ANGLE: Regex = Regex::new(r"^(\d+)$").unwrap();
    }

    let Some(captures) = BIN_ANGLE.captures(text) else {return None};
    let Some(bin_str) = captures.get(1) else {return None};

    Some(
        AngleNum::from_str_radix(bin_str.as_str(), 2)
            .map(|numer| RationalAngle::new(numer, 1 << bin_str.as_str().len()))
            .map_err(|x| ParseAngleError::Dyadic(Box::new(x))),
    )
}

#[allow(clippy::unwrap_used)]
fn parse_preperiodic(text: &str) -> Option<Result<RationalAngle, ParseAngleError>>
{
    lazy_static! {
        static ref BIN_PREPER: Regex = Regex::new(r"^(\d*)p(\d+)$").unwrap();
    }

    let Some(captures) = BIN_PREPER.captures(text) else {return None};
    if let (Some(pre_match), Some(per_match)) = (captures.get(1), captures.get(2))
    {
        let per_str = per_match.as_str();
        let pre_str = pre_match.as_str();

        if pre_str.is_empty()
        {
            match AngleNum::from_str_radix(per_str, 2)
            {
                Ok(num_per) =>
                {
                    let den_per = (1 << per_str.len()) - 1;
                    let per = RationalAngle::new(num_per, den_per);
                    return Some(Ok(per));
                }
                Err(e) =>
                {
                    return Some(Err(ParseAngleError::Preperiodic(Box::new(e))));
                }
            }
        }

        match (
            AngleNum::from_str_radix(pre_str, 2),
            AngleNum::from_str_radix(per_str, 2),
        )
        {
            (Ok(num_pre), Ok(num_per)) =>
            {
                let den_pre = 1 << pre_str.len();
                let den_per = den_pre * ((1 << per_str.len()) - 1);

                let pre = RationalAngle::new(num_pre, den_pre);
                let per = RationalAngle::new(num_per, den_per);
                return Some(Ok(pre + per));
            }
            (Err(e), _) | (_, Err(e)) =>
            {
                return Some(Err(ParseAngleError::Preperiodic(Box::new(e))));
            }
        }
    }
    None
}

impl FromStr for RationalAngle
{
    type Err = ParseAngleError;

    /// Parse text representing an angle as done in Wolf Jung's Mandel.
    /// Supports fraction strings, e.g. "17/168",
    /// binary strings for dyadic angles, e.g. "011" -> 3/8,
    /// and binary representations of (pre)periodic angles,
    /// e.g. "011p10" -> 3/2^3 + 2/(2^3*(2^2-1)) = 11/24
    fn from_str(text: &str) -> Result<Self, Self::Err>
    {
        if let Some(result) = parse_fraction(text)
            .or_else(|| parse_dyadic(text))
            .or_else(|| parse_preperiodic(text))
        {
            return result;
        }
        Err(ParseAngleError::UnrecognizedFormat)
    }
}

//
#[derive(Debug)]
pub enum ParseOrbitSchemaError
{
    UnrecognizedFormat,
    Preperiodic(ParseIntError),
    Periodic(ParseIntError),
}

impl std::fmt::Display for ParseOrbitSchemaError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self
        {
            Self::UnrecognizedFormat => write!(f, "Unrecognized angle format."),
            Self::Periodic(cause) =>
            {
                write!(f, "Error parsing periodic string: {}", cause)
            }
            Self::Preperiodic(cause) =>
            {
                write!(f, "Error parsing preperiodic string: {}", cause)
            }
        }
    }
}

impl std::error::Error for ParseOrbitSchemaError
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)>
    {
        match self
        {
            Self::UnrecognizedFormat => None,
            Self::Periodic(cause) => Some(cause),
            Self::Preperiodic(cause) => Some(cause),
        }
    }
}
