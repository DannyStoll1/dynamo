use crate::primitive_types::*;

#[derive(Clone, Copy, Debug)]
pub struct OrbitInfo {
    pub point: ComplexNum,
    pub param: ComplexNum,
    pub result: EscapeState,
}

