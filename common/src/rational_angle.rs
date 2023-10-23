use crate::symbolic_dynamics::AngleWithDegree;
use crate::types::{AngleNum, Rational};
use derive_more::{From, Into};
use num_traits::sign::Signed;

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};


#[cfg(feature = "serde")]
#[derive(Serialize, Deserialize)]
#[serde(remote = "Rational")]
struct RatioDef {
    #[serde(getter = "Rational::numer")]
    numer: AngleNum,
    #[serde(getter = "Rational::denom")]
    denom: AngleNum,
}

#[cfg(feature = "serde")]
impl From<RatioDef> for Rational {
    fn from(r: RatioDef) -> Self {
        Rational::new(r.numer, r.denom)
    }
}

// #[cfg(feature = "serde")]
// impl From<Rational> for RatioDef {
//     fn from(r: Rational) -> Self {
//         RatioDef {
//             numer: *r.numer(),
//             denom: *r.denom(),
//         }
//     }
// }

/// Wrapper class for num_rational::Rational that performs arithmetic mod 1
#[derive(Clone, Copy, Debug, Hash, From, Into, PartialEq, Eq, PartialOrd, Ord, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RationalAngle(
    #[cfg_attr(feature = "serde", serde(with = "RatioDef"))]
    Rational
);

impl RationalAngle
{
    pub const ZERO: Self = Self::new_raw(0, 1);
    pub const ONE_HALF: Self = Self::new_raw(1, 2);

    #[must_use]
    pub fn new(numer: AngleNum, denom: AngleNum) -> Self
    {
        let rational = Rational::new(numer, denom);
        Self(rational)
    }

    /// Creates a RationalAngle without checking for zero division, reducing, or projecting mod 1
    #[must_use]
    pub const fn new_raw(numer: AngleNum, denom: AngleNum) -> Self
    {
        Self(Rational::new_raw(numer, denom))
    }

    #[must_use]
    pub const fn with_degree(self, degree: AngleNum) -> AngleWithDegree
    {
        AngleWithDegree {
            angle: self,
            degree,
        }
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
}

// #[cfg(feature = "serde")]
// impl<'de> Deserialize<'de> for Duration
// {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         #[derive(Deserialize)]
//         enum Field
//         {
//             Secs,
//             Nanos,
//         }
//
//         // This part could also be generated independently by:
//         //
//         //    #[derive(Deserialize)]
//         //    #[serde(field_identifier, rename_all = "lowercase")]
//         //    enum Field { Secs, Nanos }
//         impl<'de> Deserialize<'de> for Field
//         {
//             fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
//             where
//                 D: Deserializer<'de>,
//             {
//                 struct FieldVisitor;
//
//                 impl<'de> Visitor<'de> for FieldVisitor
//                 {
//                     type Value = Field;
//
//                     fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result
//                     {
//                         formatter.write_str("`secs` or `nanos`")
//                     }
//
//                     fn visit_str<E>(self, value: &str) -> Result<Field, E>
//                     where
//                         E: de::Error,
//                     {
//                         match value
//                         {
//                             "secs" => Ok(Field::Secs),
//                             "nanos" => Ok(Field::Nanos),
//                             _ => Err(de::Error::unknown_field(value, FIELDS)),
//                         }
//                     }
//                 }
//
//                 deserializer.deserialize_identifier(FieldVisitor)
//             }
//         }
//
//         struct DurationVisitor;
//
//         impl<'de> Visitor<'de> for DurationVisitor
//         {
//             type Value = Duration;
//
//             fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result
//             {
//                 formatter.write_str("struct Duration")
//             }
//
//             fn visit_seq<V>(self, mut seq: V) -> Result<Duration, V::Error>
//             where
//                 V: SeqAccess<'de>,
//             {
//                 let secs = seq
//                     .next_element()?
//                     .ok_or_else(|| de::Error::invalid_length(0, &self))?;
//                 let nanos = seq
//                     .next_element()?
//                     .ok_or_else(|| de::Error::invalid_length(1, &self))?;
//                 Ok(Duration::new(secs, nanos))
//             }
//
//             fn visit_map<V>(self, mut map: V) -> Result<Duration, V::Error>
//             where
//                 V: MapAccess<'de>,
//             {
//                 let mut secs = None;
//                 let mut nanos = None;
//                 while let Some(key) = map.next_key()?
//                 {
//                     match key
//                     {
//                         Field::Secs =>
//                         {
//                             if secs.is_some()
//                             {
//                                 return Err(de::Error::duplicate_field("secs"));
//                             }
//                             secs = Some(map.next_value()?);
//                         }
//                         Field::Nanos =>
//                         {
//                             if nanos.is_some()
//                             {
//                                 return Err(de::Error::duplicate_field("nanos"));
//                             }
//                             nanos = Some(map.next_value()?);
//                         }
//                     }
//                 }
//                 let secs = secs.ok_or_else(|| de::Error::missing_field("secs"))?;
//                 let nanos = nanos.ok_or_else(|| de::Error::missing_field("nanos"))?;
//                 Ok(Duration::new(secs, nanos))
//             }
//         }
//
//         const FIELDS: &'static [&'static str] = &["secs", "nanos"];
//         deserializer.deserialize_struct("Duration", FIELDS, DurationVisitor)
//     }
// }

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

macro_rules! mul_div_int_impl {
    ($other: ty) => {
        impl std::ops::Mul<$other> for RationalAngle
        {
            type Output = Self;

            fn mul(self, rhs: $other) -> Self::Output
            {
                Self(self.0 * (rhs as AngleNum)).mod_1()
            }
        }
        impl std::ops::Mul<RationalAngle> for $other
        {
            type Output = RationalAngle;

            fn mul(self, rhs: RationalAngle) -> Self::Output
            {
                rhs * (self as AngleNum)
            }
        }
        impl std::ops::MulAssign<$other> for RationalAngle
        {
            fn mul_assign(&mut self, rhs: $other)
            {
                self.0 *= rhs as AngleNum;
                self.reduce_mod_1();
            }
        }
        impl std::ops::Div<$other> for RationalAngle
        {
            type Output = Self;

            fn div(self, rhs: $other) -> Self::Output
            {
                Self(self.0 / (rhs as AngleNum)).mod_1()
            }
        }
        impl std::ops::DivAssign<$other> for RationalAngle
        {
            fn div_assign(&mut self, rhs: $other)
            {
                self.0 /= rhs as AngleNum;
                self.reduce_mod_1();
            }
        }
    };
}

impl std::ops::Mul<AngleNum> for RationalAngle
{
    type Output = Self;

    fn mul(self, rhs: AngleNum) -> Self::Output
    {
        Self(self.0 * rhs).mod_1()
    }
}

impl std::ops::Mul<Rational> for RationalAngle
{
    type Output = Self;

    fn mul(self, rhs: Rational) -> Self::Output
    {
        Self(self.0 * rhs).mod_1()
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

mul_div_int_impl!(u32);
mul_div_int_impl!(i32);
mul_div_int_impl!(u64);

impl std::fmt::Display for RationalAngle
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl std::fmt::Binary for RationalAngle
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        let degree = f.precision().and_then(|x| x.try_into().ok()).unwrap_or(2);
        let itinerary = self.with_degree(degree).canonical_itinerary(Self::ZERO);
        std::fmt::Display::fmt(&itinerary, f)
    }
}

impl std::ops::Deref for RationalAngle
{
    type Target = Rational;

    fn deref(&self) -> &Self::Target
    {
        &self.0
    }
}
