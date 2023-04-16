use crate::primitive_types::*;
use super::ParameterPlane;

pub struct Orbit<F, S>
where
    F: Fn(ComplexNum, ComplexNum) -> ComplexNum,
    S: Fn(Period, ComplexNum) -> EscapeState,
{
    f: F,
    stop_condition: S,
    param: ComplexNum,
    pub z: ComplexNum,
    pub iter: Period,
    pub state: EscapeState,
}

impl<F, S> Orbit<F, S>
where
    F: Fn(ComplexNum, ComplexNum) -> ComplexNum,
    S: Fn(Period, ComplexNum) -> EscapeState,
{
    pub fn new(f: F, stop_condition: S, z: ComplexNum, param: ComplexNum) -> Self
    {
        Self {
            f,
            stop_condition,
            z,
            param,
            iter: 0,
            state: EscapeState::NotYetEscaped,
        }
    }

    fn apply_map(&self, z: ComplexNum) -> ComplexNum
    {
        (self.f)(z, self.param)
    }

    // pub fn from_plane(plane: Box<dyn ParameterPlane>, param: ComplexNum) -> Self
    // {
    //     let start = plane.start_point(param);
    //     Self::new(
    //         |z, c| plane.map(z, c),
    //         |i, z| plane.stop_condition(i, z),
    //         start,
    //         param,
    //     )
    // }
}

impl<F, S> Iterator for Orbit<F, S>
where
    F: Fn(ComplexNum, ComplexNum) -> ComplexNum,
    S: Fn(Period, ComplexNum) -> EscapeState,
{
    type Item = (ComplexNum, EscapeState);

    fn next(&mut self) -> Option<Self::Item>
    {
        if self.iter == 0 {
            self.iter = 1;
            self.state = (self.stop_condition)(self.iter, self.z);
            return Some((self.z, self.state));
        }

        if let EscapeState::NotYetEscaped = self.state
        {
            self.z = self.apply_map(self.z);
            self.iter += 1;
            self.state = (self.stop_condition)(self.iter, self.z);
            Some((self.z, self.state))
        }
        else
        {
            None
        }
    }
}

pub struct CycleDetectedOrbit<F, S, C>
where
    F: Fn(ComplexNum, ComplexNum) -> ComplexNum,
    S: Fn(Period, ComplexNum) -> EscapeState,
    C: Fn(Period, ComplexNum, ComplexNum, ComplexNum) -> EscapeState,
{
    f: F,
    stop_condition: S,
    check_periodicity: C,
    param: ComplexNum,
    pub z_slow: ComplexNum,
    pub z_fast: ComplexNum,
    pub iter: Period,
    pub state: EscapeState,
}

impl<F, S, C> CycleDetectedOrbit<F, S, C>
where
    F: Fn(ComplexNum, ComplexNum) -> ComplexNum,
    S: Fn(Period, ComplexNum) -> EscapeState,
    C: Fn(Period, ComplexNum, ComplexNum, ComplexNum) -> EscapeState,
{
    pub fn new(
        f: F,
        stop_condition: S,
        check_periodicity: C,
        z: ComplexNum,
        param: ComplexNum,
    ) -> Self
    {
        Self {
            f,
            stop_condition,
            check_periodicity,
            param,
            z_slow: z,
            z_fast: z,
            iter: 0,
            state: EscapeState::NotYetEscaped,
        }
    }

    fn apply_map(&self, z: ComplexNum) -> ComplexNum
    {
        (self.f)(z, self.param)
    }

    // pub fn from_plane(plane: impl ParameterPlane, param: ComplexNum) -> Self
    // {
    //     let start = plane.start_point(param);
    //     Self::new(
    //         |z, c| plane.map(z, c),
    //         |i, z| plane.stop_condition(i, z),
    //         |i, z0, z1, c| plane.check_periodicity(i, z0, z1, c),
    //         start,
    //         param,
    //     )
    // }
}

impl<F, S, C> Iterator for CycleDetectedOrbit<F, S, C>
where
    F: Fn(ComplexNum, ComplexNum) -> ComplexNum,
    S: Fn(Period, ComplexNum) -> EscapeState,
    C: Fn(Period, ComplexNum, ComplexNum, ComplexNum) -> EscapeState,
{
    type Item = (ComplexNum, EscapeState);

    fn next(&mut self) -> Option<Self::Item>
    {
        if let EscapeState::NotYetEscaped = self.state
        {
            self.iter += 1;
            if self.iter % 2 == 1
            {
                self.z_slow = self.apply_map(self.z_slow);
                self.z_fast = self.apply_map(self.z_fast);
                self.state = (self.stop_condition)(self.iter, self.z_fast);
            }
            else
            {
                self.z_fast = self.apply_map(self.z_fast);
                self.state =
                    (self.check_periodicity)(self.iter, self.z_slow, self.z_fast, self.param);
            }
            Some((self.z_fast, self.state))
        }
        else
        {
            None
        }
    }
}
