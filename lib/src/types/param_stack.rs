use super::ComplexNum;
use derive_more::Display;

#[derive(Clone, Copy, Debug, Default, Display)]
#[display(fmt = "")]
pub struct NoParam {}

pub trait ParamList: Clone
{
    type Param: Default + Clone + Copy + std::fmt::Display;
    fn local_param(&self) -> Self::Param;
    fn into_local_param(self) -> Self::Param;
}

impl<M, P> ParamList for (M, P)
where
    M: Clone + Default + std::fmt::Display,
    P: Clone + Copy + Default + std::fmt::Display,
{
    type Param = P;
    fn local_param(&self) -> Self::Param
    {
        self.1
    }
    fn into_local_param(self) -> Self::Param
    {
        self.1
    }
}

impl ParamList for ComplexNum
{
    type Param = Self;
    fn local_param(&self) -> Self::Param
    {
        *self
    }
    fn into_local_param(self) -> Self::Param
    {
        self
    }
}

impl ParamList for NoParam
{
    type Param = Self;
    fn local_param(&self) -> Self::Param
    {
        Self {}
    }
    fn into_local_param(self) -> Self::Param
    {
        Self {}
    }
}

#[derive(Clone, Copy, Display, Default)]
#[display(fmt = "[{}, {}]", meta_params, local_param)]
pub struct ParamStack<T, H>
where
    T: Clone + Default + std::fmt::Display,
    H: Clone + Default + std::fmt::Display,
{
    pub meta_params: T,
    pub local_param: H,
}

impl<T, H> ParamStack<T, H>
where
    T: Clone + Default + std::fmt::Display,
    H: Clone + Default + std::fmt::Display,
{
    pub fn new(meta_params: T, local_param: H) -> Self
    {
        Self {
            meta_params,
            local_param,
        }
    }
}

impl<T, H> ParamList for ParamStack<T, H>
where
    H: Clone + Copy + Default + std::fmt::Display,
    T: Clone + Default + std::fmt::Display,
{
    type Param = H;

    fn local_param(&self) -> Self::Param
    {
        self.local_param
    }
    fn into_local_param(self) -> Self::Param
    {
        self.local_param
    }
}
