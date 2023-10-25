#![allow(dead_code)]
#![allow(unused_imports)]
#![feature(const_fn_floating_point_arithmetic)]

pub mod dynamics;
pub mod error;
pub mod macros;
pub mod prelude;

#[cfg(test)]
mod tests
{
    use dynamo_common::prelude::{OrbitSchema, RationalAngle};

    #[test]
    fn angle_period()
    {
        let angle = RationalAngle::new(3, 15);
        let period_schema = angle.with_degree(2).orbit_schema();
        assert_eq!(
            period_schema,
            OrbitSchema {
                period: 4,
                preperiod: 0
            }
        );

        let angle = RationalAngle::new(1, 10);
        let period_schema = angle.with_degree(2).orbit_schema();
        assert_eq!(
            period_schema,
            OrbitSchema {
                period: 4,
                preperiod: 1
            }
        );

        let angle = RationalAngle::new(17, 168);
        let period_schema = angle.with_degree(2).orbit_schema();
        assert_eq!(
            period_schema,
            OrbitSchema {
                period: 6,
                preperiod: 3
            }
        );
    }

    #[test]
    fn kneading_sequence()
    {
        let angle = RationalAngle::new(3, 7).with_degree(2);
        let kneading_sequence = angle.kneading_sequence();
        assert_eq!(kneading_sequence.to_string(), "p10*".to_owned());
    }

    #[test]
    fn parsing()
    {
        let str0 = "1/17";
        let val0 = RationalAngle::new(1, 17);

        let str1 = "0110";
        let val1 = RationalAngle::new(3, 8);

        let str2 = "p101";
        let val2 = RationalAngle::new(5, 7);

        let str3 = "011p10";
        let val3 = RationalAngle::new(11, 24);

        let Ok(out0) = str0.parse::<RationalAngle>()
        else
        {
            panic!("parse_angle returned None on input {str0}")
        };
        let Ok(out1) = str1.parse::<RationalAngle>()
        else
        {
            panic!("parse_angle returned None on input {str1}")
        };
        let Ok(out2) = str2.parse::<RationalAngle>()
        else
        {
            panic!("parse_angle returned None on input {str2}")
        };
        let Ok(out3) = str3.parse::<RationalAngle>()
        else
        {
            panic!("parse_angle returned None on input {str3}")
        };

        assert_eq!(out0, val0);
        assert_eq!(out1, val1);
        assert_eq!(out2, val2);
        assert_eq!(out3, val3);
    }
}
