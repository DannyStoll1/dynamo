use crate::primitive_types::{ComplexNum, EscapeState};

#[derive(Clone, Copy, Debug)]
pub struct OrbitInfo {
    pub point: ComplexNum,
    pub param: ComplexNum,
    pub result: EscapeState,
}

