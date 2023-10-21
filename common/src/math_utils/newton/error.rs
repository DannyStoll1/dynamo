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
        match self
        {
            Error::FailedToConverge(val) => Error::FailedToConverge(f(val)),
            Error::NanEncountered => Error::NanEncountered,
        }
    }
}

pub type NewtonResult<T> = Result<T, Error<T>>;
