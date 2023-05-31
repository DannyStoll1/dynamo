use super::ComplexNum;
use derive_more::Display;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Display)]
#[display(fmt = "")]
pub struct NoParam {}

pub trait Summarize : std::fmt::Display
{
    fn summarize(&self) -> Option<String> {
        Some(format!("{}", self))
    }
}

impl Summarize for NoParam
{
    fn summarize(&self) -> Option<String>
    {
        None
    }
}

impl Summarize for ComplexNum
{
    fn summarize(&self) -> Option<String>
    {
        Some(format!("c = {self}"))
    }
}

pub trait ParamList: Clone
{
    type Param: Default + Clone + Copy + PartialEq + Summarize;
    fn local_param(&self) -> Self::Param;
    fn into_local_param(self) -> Self::Param;
}

impl<M, P> ParamList for (M, P)
where
    M: Clone + Default + Summarize,
    P: Clone + Copy + Default + PartialEq + Summarize,
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
    T: Clone + Default + Summarize,
    H: Clone + Default + PartialEq + Summarize,
{
    pub meta_params: T,
    pub local_param: H,
}

impl<T, H> Summarize for ParamStack<T, H>
where
    T: Clone + Default + Summarize,
    H: Clone + Default + PartialEq + Summarize,
{
    fn summarize(&self) -> Option<String>
    {
        if let Some(meta) = self.meta_params.summarize()
        {
            if let Some(local) = self.local_param.summarize()
            {
                Some(format!("[{}, {}]", meta, local))
            }
            else
            {
                Some(meta)
            }
        }
        else
        {
            self.local_param.summarize()
        }
    }
}

impl<T, H> ParamStack<T, H>
where
    T: Clone + Default + Summarize,
    H: Clone + Default + PartialEq + Summarize,
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
    T: Clone + Default + Summarize,
    H: Clone + Copy + Default + PartialEq + Summarize,
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
