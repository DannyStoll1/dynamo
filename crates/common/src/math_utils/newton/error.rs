#[derive(Clone, Copy, Debug)]
pub enum Error<T>
{
    FailedToConverge(T),
    NanEncountered,
}

impl<T> Error<T>
{
    pub fn map<U, F>(self, f: F) -> Error<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Self::FailedToConverge(val) => Error::FailedToConverge(f(val)),
            Self::NanEncountered => Error::NanEncountered,
        }
    }
}

pub type NewtonResult<T> = Result<T, Error<T>>;
