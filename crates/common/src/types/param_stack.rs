use derive_more::Display;

use super::Cplx;
use crate::prelude::DescriptionConf;
use crate::traits::{Describe, Summarize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Display)]
#[display("")]
pub struct NoParam;
impl From<Cplx> for NoParam
{
    fn from(_: Cplx) -> Self
    {
        Self
    }
}

impl Describe for NoParam
{
    fn describe(&self, _desc_params: &DescriptionConf) -> Option<String>
    {
        None
    }
}
impl Summarize for NoParam {}

pub trait ParamList: Clone
{
    type Param: Default + Clone + PartialEq + Summarize;
    fn local_param(&self) -> &Self::Param;
    fn into_local_param(self) -> Self::Param;
}

impl<M, P> ParamList for (M, P)
where
    M: Clone + Default + Summarize,
    P: Clone + Default + PartialEq + Summarize,
{
    type Param = P;
    fn local_param(&self) -> &Self::Param
    {
        &self.1
    }
    fn into_local_param(self) -> Self::Param
    {
        self.1
    }
}

impl ParamList for Cplx
{
    type Param = Self;
    fn local_param(&self) -> &Self::Param
    {
        self
    }
    fn into_local_param(self) -> Self::Param
    {
        self
    }
}

impl ParamList for i32
{
    type Param = Self;
    fn local_param(&self) -> &Self::Param
    {
        self
    }
    fn into_local_param(self) -> Self::Param
    {
        self
    }
}

impl ParamList for NoParam
{
    type Param = Self;
    fn local_param(&self) -> &Self::Param
    {
        &Self {}
    }
    fn into_local_param(self) -> Self::Param
    {
        Self {}
    }
}

#[derive(Clone, Display, Default)]
#[display("[{meta_params}, {local_param}]")]
pub struct ParamStack<T, H>
where
    T: Clone + Default + Summarize,
    H: Clone + Default + PartialEq + Summarize,
{
    pub meta_params: T,
    pub local_param: H,
}

impl<T, H> Describe for ParamStack<T, H>
where
    T: Clone + Default + Summarize + Describe,
    H: Clone + Default + PartialEq + Summarize + Describe,
{
    fn describe(&self, desc_params: &DescriptionConf) -> Option<String>
    {
        self.meta_params.describe(desc_params).map_or_else(
            || self.local_param.describe(desc_params),
            |meta| {
                if let Some(local) = self.local_param.describe(desc_params) {
                    Some(format!("[{meta}, {local}]"))
                } else {
                    Some(meta)
                }
            },
        )
    }
}

impl<T, H> Summarize for ParamStack<T, H>
where
    T: Clone + Default + Summarize,
    H: Clone + Default + PartialEq + Summarize,
{
    fn summarize(&self) -> Option<String>
    {
        self.meta_params.summarize().map_or_else(
            || self.local_param.summarize(),
            |meta| {
                if let Some(local) = self.local_param.summarize() {
                    Some(format!("[{meta}, {local}]"))
                } else {
                    Some(meta)
                }
            },
        )
    }
}

impl<T, H> ParamStack<T, H>
where
    T: Clone + Default + Summarize,
    H: Clone + Default + PartialEq + Summarize,
{
    #[must_use]
    pub const fn new(meta_params: T, local_param: H) -> Self
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
    H: Clone + Default + PartialEq + Summarize,
{
    type Param = H;

    fn local_param(&self) -> &Self::Param
    {
        &self.local_param
    }
    fn into_local_param(self) -> Self::Param
    {
        self.local_param
    }
}
