use dynamo_common::math_utils::newton::error::Error as NewtonError;

#[derive(Clone, Copy, Debug)]
pub enum FindPointError<T>
{
    PeriodIsZero,
    NewtonError(NewtonError<T>),
}

pub type FindPointResult<T> = Result<T, FindPointError<T>>;
