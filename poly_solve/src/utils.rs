use itertools::EitherOrBoth;

pub(crate) trait Collapse
{
    type Item;
    fn collapse<F>(self, f: F) -> Self::Item
    where
        F: Fn(Self::Item, Self::Item) -> Self::Item;
}

impl<T> Collapse for EitherOrBoth<T, T>
{
    type Item = T;
    fn collapse<F>(self, f: F) -> T
    where
        F: Fn(T, T) -> T,
    {
        match self
        {
            Self::Left(x) => x,
            Self::Right(x) => x,
            Self::Both(x, y) => f(x, y),
        }
    }
}
