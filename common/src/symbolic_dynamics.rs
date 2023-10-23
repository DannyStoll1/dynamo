use lazy_static::lazy_static;
use regex::Regex;
use std::{cmp::Ordering, collections::VecDeque, error::Error, num::ParseIntError, str::FromStr};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::prelude::*;

/// Information to display about a rational angle
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AngleInfo
{
    pub angle: RationalAngle,
    pub orbit_schema: OrbitSchema,
    pub kneading_sequence: Itinerary,
}

/// Period and preperiod of a point with finite orbit in a dynamical system.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct OrbitSchema
{
    pub period: Period,
    pub preperiod: Period,
}

impl OrbitSchema
{
    #[must_use]
    pub const fn with_degree(self, degree: AngleNum) -> OrbitSchemaWithDegree
    {
        OrbitSchemaWithDegree {
            preperiod: self.preperiod,
            period: self.period,
            degree,
        }
    }
}

impl PartialOrd for OrbitSchema
{
    fn lt(&self, other: &Self) -> bool
    {
        self.preperiod < other.preperiod && self.period == other.period
    }
    fn le(&self, other: &Self) -> bool
    {
        self.preperiod <= other.preperiod && self.period == other.period
    }
    fn partial_cmp(&self, other: &Self) -> Option<Ordering>
    {
        if self.period != other.period
        {
            return None;
        }
        Some(self.preperiod.cmp(&other.preperiod))
    }
}

impl Default for OrbitSchema
{
    fn default() -> Self
    {
        Self {
            period: 1,
            preperiod: 0,
        }
    }
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct OrbitSchemaWithDegree
{
    pub preperiod: Period,
    pub period: Period,
    pub degree: AngleNum,
}

impl OrbitSchemaWithDegree
{
    /// Value m for which all angles with this orbit schema can be expressed in the form $k/m$.
    pub const fn natural_denom(&self) -> AngleNum
    {
        (self.degree.pow(self.preperiod) * (self.degree.pow(self.period) - 1)).abs()
    }

    pub fn active_angles(&self, include_suffixes: bool) -> VecDeque<RationalAngle>
    {
        if include_suffixes
        {
            self.child_angles()
        }
        else
        {
            self.exact_angles()
        }
    }

    /// All angles of the same period and preperiod
    pub fn exact_angles(&self) -> VecDeque<RationalAngle>
    {
        let denom = self.natural_denom();
        (1..denom)
            .map(|numer| RationalAngle::new(numer, denom))
            .filter(|theta| theta.with_degree(self.degree).orbit_schema() == self.forget())
            .collect()
    }

    /// All angles of the same period and the same or smaller preperiod
    pub fn child_angles(&self) -> VecDeque<RationalAngle>
    {
        let denom = self.natural_denom();
        (1..denom)
            .map(|numer| RationalAngle::new(numer, denom))
            .filter(|theta| theta.with_degree(self.degree).orbit_schema() <= self.forget())
            .collect()
    }

    const fn forget(self) -> OrbitSchema
    {
        OrbitSchema {
            preperiod: self.preperiod,
            period: self.period,
        }
    }
}

impl From<OrbitSchemaWithDegree> for OrbitSchema
{
    fn from(od: OrbitSchemaWithDegree) -> Self
    {
        od.forget()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
        let formatted = format!("{}p{}", pre_str, per_str);
        f.pad(&formatted)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CirclePartition
{
    angles: VecDeque<RationalAngle>,
}
impl CirclePartition
{
    #[must_use]
    pub fn new(mut angles: VecDeque<RationalAngle>) -> Self
    {
        angles.make_contiguous().sort();
        Self { angles }
    }

    /// Assumes input is in ascending circular order.
    #[must_use]
    pub fn from_circularly_ordered(mut angles: VecDeque<RationalAngle>) -> Self
    {
        sort_circularly_ordered(&mut angles);
        Self { angles }
    }

    /// Assumes angles are sorted
    #[must_use]
    pub fn new_raw(angles: VecDeque<RationalAngle>) -> Self
    {
        Self { angles }
    }

    /// Partition of the form $\{ theta/n, (theta+1)/n, ..., (theta+n-1)/n \}$.
    /// These are of particular importance dynamically. For instance, if $c$
    /// has external angle $\theta$ in the complement of the degree $n$ multibrot set,
    /// and $z\in J(f_c)$ is a landing point of a dynamical ray at an angle $\alpha$,
    /// then the topological dynamics of $z$ relative to the critical point is described by
    /// the itinerary of $\alpha$ relative to the canonical partition of $\theta$.
    #[must_use]
    pub fn canonical(angle: RationalAngle, degree: AngleNum) -> Self
    {
        let theta_over_n = angle / degree;
        //Guaranteed to be circularly ordered
        let partition_angles = (0..degree.abs())
            .map(|x| theta_over_n + RationalAngle::new_raw(x, degree))
            .collect();
        Self::from_circularly_ordered(partition_angles)
    }

    #[must_use]
    pub fn identify(&self, theta: RationalAngle) -> KneadingSymbol
    {
        for (i, x) in self.angles.iter().enumerate().rev()
        {
            match theta.cmp(x)
            {
                Ordering::Greater => return KneadingSymbol::Interior(i),
                Ordering::Equal => return KneadingSymbol::Boundary(i),
                Ordering::Less => {}
            }
        }
        KneadingSymbol::Interior(self.angles.len().saturating_sub(1))
    }

    #[must_use]
    pub fn is_circularly_ordered(&self) -> bool
    {
        let Some((imin, xmin)) = self.angles.iter().enumerate().min_by_key(|(_i, x)| *x)
        else
        {
            return true;
        };
        let mut x_curr = *xmin;
        for i in (imin + 1)..self.angles.len()
        {
            if self.angles[i] < x_curr
            {
                return false;
            }
            x_curr = self.angles[i];
        }
        for i in 0..imin
        {
            if self.angles[i] < x_curr
            {
                return false;
            }
            x_curr = self.angles[i];
        }
        true
    }

    #[must_use]
    pub fn size(&self) -> usize
    {
        self.angles.len()
    }
}

impl From<RationalAngle> for Real
{
    fn from(angle: RationalAngle) -> Self
    {
        (*angle.numer() as Self) / (*angle.denom() as Self)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AngleWithDegree
{
    pub angle: RationalAngle,
    pub degree: AngleNum,
}

impl AngleWithDegree
{
    /// Preperiod and period under the angle tupling map
    /// Guaranteed to be finite, e.g. by pigeon hole principle.
    pub fn orbit_schema(&self) -> OrbitSchema
    {
        let degree2 = self.degree * self.degree;
        let mut slow = self.angle;
        let mut fast = self.angle;

        loop
        {
            slow *= self.degree;
            fast *= degree2;
            if fast == slow
            {
                break;
            }
        }

        let mut period = 1;
        slow *= self.degree;

        while slow != fast
        {
            slow *= self.degree;
            period += 1;
        }

        let mut preperiod = 0;
        slow = self.angle;
        while slow != fast
        {
            slow *= self.degree;
            fast *= self.degree;
            preperiod += 1;
        }

        OrbitSchema { preperiod, period }
    }

    pub fn itinerary(&self, partition: CirclePartition) -> Itinerary
    {
        let orbit_schema = self.orbit_schema();
        self.itinerary_given_orbit_schema(orbit_schema, partition)
    }

    fn itinerary_given_orbit_schema(
        &self,
        orbit_schema: OrbitSchema,
        partition: CirclePartition,
    ) -> Itinerary
    {
        let mut x = self.angle;

        let preperiodic_part = (0..orbit_schema.preperiod)
            .map(|_i| {
                let id = partition.identify(x);
                x *= self.degree;
                id
            })
            .collect();

        let periodic_part = (0..orbit_schema.period)
            .map(|_i| {
                let id = partition.identify(x);
                x *= self.degree;
                id
            })
            .collect();

        Itinerary {
            partition,
            preperiodic_part,
            periodic_part,
        }
    }

    pub fn kneading_sequence(&self) -> Itinerary
    {
        let orbit_schema = self.orbit_schema();
        self.kneading_sequence_given_orbit_schema(orbit_schema)
    }

    fn kneading_sequence_given_orbit_schema(&self, orbit_schema: OrbitSchema) -> Itinerary
    {
        let theta_over_n = self.angle / self.degree;
        let partition_angles = (0..self.degree)
            .map(|x| theta_over_n + RationalAngle::new_raw(x, self.degree))
            .collect();
        let partition = CirclePartition::new_raw(partition_angles);
        self.itinerary_given_orbit_schema(orbit_schema, partition)
    }

    /// Canonical itinerary of self relative to `rel_angle`
    pub fn canonical_itinerary(&self, rel_angle: RationalAngle) -> Itinerary
    {
        let orbit_schema = self.orbit_schema();
        self.canonical_itinerary_given_orbit_schema(orbit_schema, rel_angle)
    }

    /// Canonical itinerary of self relative to `rel_angle`
    pub fn canonical_itinerary_given_orbit_schema(
        &self,
        orbit_schema: OrbitSchema,
        rel_angle: RationalAngle,
    ) -> Itinerary
    {
        let partition = CirclePartition::canonical(rel_angle, self.degree);
        self.itinerary_given_orbit_schema(orbit_schema, partition)
    }

    #[must_use]
    pub fn to_angle_info(self) -> AngleInfo
    {
        let orbit_schema = self.orbit_schema();
        let kneading_sequence = self.kneading_sequence_given_orbit_schema(orbit_schema);
        AngleInfo {
            angle: self.angle,
            orbit_schema,
            kneading_sequence,
        }
    }
}

impl std::fmt::Display for AngleWithDegree
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        let itinerary = self.canonical_itinerary(RationalAngle::ZERO);
        itinerary.fmt(f)
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

    let Some(captures) = BIN_ANGLE.captures(text)
    else
    {
        return None;
    };
    let Some(bin_str) = captures.get(1)
    else
    {
        return None;
    };

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

    let Some(captures) = BIN_PREPER.captures(text)
    else
    {
        return None;
    };
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

/// Sort a VecDeque assuming it is circularly ordered.
pub(crate) fn sort_circularly_ordered<T: PartialOrd>(angles: &mut VecDeque<T>)
{
    let Some(mut prev) = angles.front()
    else
    {
        return;
    };
    for (i, x) in angles.iter().enumerate()
    {
        if x < prev
        {
            angles.rotate_left(i);
            return;
        }
        prev = x;
    }
}
