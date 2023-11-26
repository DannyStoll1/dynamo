use crate::dynamics::{DynamicalFamily, FamilyDefaults};
use dynamo_common::prelude::*;

pub struct WithSelection<T: DynamicalFamily>
{
    pub family: T,
    point: Cplx,
    param: T::Param,
}

impl<T: DynamicalFamily> WithSelection<T>
{
    #[inline]
    pub fn selected_param(&self) -> &T::Param
    {
        &self.param
    }

    #[inline]
    pub fn selected_point(&self) -> &Cplx
    {
        &self.point
    }

    #[inline]
    pub fn select(&mut self, point: Cplx)
    {
        self.param = self.family.param_map(point);
        self.point = point;
    }

    #[inline]
    pub(crate) fn select_param(&mut self, param: T::Param)
    {
        self.param = param;
    }
}

impl<T: FamilyDefaults> From<T> for WithSelection<T>
{
    fn from(family: T) -> Self
    {
        let point = family.default_selection();
        let param = family.param_map(point);
        Self {
            family,
            point,
            param,
        }
    }
}

impl<T: DynamicalFamily> std::ops::Deref for WithSelection<T>
{
    type Target = T;
    fn deref(&self) -> &Self::Target
    {
        &self.family
    }
}

impl<T: DynamicalFamily> std::ops::DerefMut for WithSelection<T>
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.family
    }
}
