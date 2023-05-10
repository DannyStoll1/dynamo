use crate::types::{ComplexNum, EscapeState, Period};

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
        if self.iter == 0
        {
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

pub struct CycleDetectedOrbit<F, G, S, C, B>
where
    F: Fn(ComplexNum, ComplexNum) -> ComplexNum,
    G: Fn(ComplexNum, ComplexNum) -> ComplexNum,
    S: Fn(Period, ComplexNum) -> EscapeState,
    C: Fn(Period, ComplexNum, ComplexNum, ComplexNum) -> EscapeState,
    B: Fn(ComplexNum, ComplexNum) -> EscapeState,
{
    f: F,
    df_dz: G,
    stop_condition: S,
    check_periodicity: C,
    early_bailout: B,
    param: ComplexNum,
    pub z_slow: ComplexNum,
    pub z_fast: ComplexNum,
    pub multiplier: ComplexNum,
    pub iter: Period,
    pub state: EscapeState,
}

impl<F, G, S, C, B> CycleDetectedOrbit<F, G, S, C, B>
where
    F: Fn(ComplexNum, ComplexNum) -> ComplexNum,
    G: Fn(ComplexNum, ComplexNum) -> ComplexNum,
    S: Fn(Period, ComplexNum) -> EscapeState,
    C: Fn(Period, ComplexNum, ComplexNum, ComplexNum) -> EscapeState,
    B: Fn(ComplexNum, ComplexNum) -> EscapeState,
{
    pub fn new(
        f: F,
        df_dz: G,
        stop_condition: S,
        check_periodicity: C,
        early_bailout: B,
        z: ComplexNum,
        param: ComplexNum,
    ) -> Self
    {
        Self {
            f,
            df_dz,
            stop_condition,
            check_periodicity,
            early_bailout,
            param,
            z_slow: z,
            z_fast: z,
            multiplier: (1.).into(),
            iter: 0,
            state: EscapeState::NotYetEscaped,
        }
    }

    fn apply_map(&self, z: ComplexNum) -> ComplexNum
    {
        (self.f)(z, self.param)
    }

    fn derivative(&self, z: ComplexNum) -> ComplexNum
    {
        (self.df_dz)(z, self.param)
    }

    fn check_early_bailout(&mut self)
    {
        self.state = (self.early_bailout)(self.z_slow, self.param);
    }

    pub fn reset(&mut self, param: ComplexNum, start_point: ComplexNum)
    {
        self.param = param;
        self.z_slow = start_point;
        self.z_fast = start_point;
        self.multiplier = (1.).into();
        self.iter = 0;
        self.state = EscapeState::NotYetEscaped;
    }

    pub fn run_until_complete(&mut self) -> EscapeState
    {
        self.check_early_bailout();

        while let EscapeState::NotYetEscaped = self.state
        {
            self.iter += 1;
            if self.iter % 2 == 1
            {
                self.z_slow = self.apply_map(self.z_slow);
                self.z_fast = self.apply_map(self.z_fast);
                self.multiplier *= self.derivative(self.z_fast);
                self.state = (self.stop_condition)(self.iter, self.z_fast);
            }
            else
            {
                self.z_fast = self.apply_map(self.z_fast);
                self.multiplier *= self.derivative(self.z_fast);
                self.state =
                    (self.check_periodicity)(self.iter, self.z_slow, self.z_fast, self.param);
            }
        }
        self.state
    }

    // pub fn from_plane(plane: impl ParameterPlane, param: ComplexNum) -> Self
    // {
    //     let start = plane.start_point(param);
    //     Self::new(
    //         |z, c| plane.map(z, c),
    //         |z, c| plane.dynamical_derivative(z, c),
    //         |i, z| plane.stop_condition(i, z),
    //         |i, z0, z1, c| plane.check_periodicity(i, z0, z1, c),
    //         start,
    //         param,
    //     )
    // }
}

impl<F, G, S, C, B> Iterator for CycleDetectedOrbit<F, G, S, C, B>
where
    F: Fn(ComplexNum, ComplexNum) -> ComplexNum,
    G: Fn(ComplexNum, ComplexNum) -> ComplexNum,
    S: Fn(Period, ComplexNum) -> EscapeState,
    C: Fn(Period, ComplexNum, ComplexNum, ComplexNum) -> EscapeState,
    B: Fn(ComplexNum, ComplexNum) -> EscapeState,
{
    type Item = (ComplexNum, EscapeState);

    fn next(&mut self) -> Option<Self::Item>
    {
        if let EscapeState::NotYetEscaped = self.state
        {
            let retval = self.z_fast;
            self.iter += 1;
            if self.iter % 2 == 1
            {
                self.z_slow = self.apply_map(self.z_slow);
                self.z_fast = self.apply_map(self.z_fast);
                self.multiplier *= self.derivative(self.z_fast);
                self.state = (self.stop_condition)(self.iter, self.z_fast);
            }
            else
            {
                self.z_fast = self.apply_map(self.z_fast);
                self.multiplier *= self.derivative(self.z_fast);
                self.state =
                    (self.check_periodicity)(self.iter, self.z_slow, self.z_fast, self.param);
            }
            Some((retval, self.state))
        }
        else
        {
            None
        }
    }
}

pub struct CycleDetectedOrbitBrent<F, G, C, B>
where
    F: Fn(ComplexNum, ComplexNum) -> ComplexNum,
    G: Fn(ComplexNum, ComplexNum) -> ComplexNum,
    C: Fn(Period, ComplexNum, ComplexNum, ComplexNum) -> EscapeState,
    B: Fn(ComplexNum, ComplexNum) -> EscapeState,
{
    f: F,
    df_dz: G,
    check_periodicity: C,
    early_bailout: B,
    param: ComplexNum,
    pub z_slow: ComplexNum,
    pub z_fast: ComplexNum,
    pub multiplier: ComplexNum,
    pub iter: Period,
    pub state: EscapeState,
    period_limit: Period,
    period: Period,
}

impl<F, G, C, B> CycleDetectedOrbitBrent<F, G, C, B>
where
    F: Fn(ComplexNum, ComplexNum) -> ComplexNum,
    G: Fn(ComplexNum, ComplexNum) -> ComplexNum,
    C: Fn(Period, ComplexNum, ComplexNum, ComplexNum) -> EscapeState,
    B: Fn(ComplexNum, ComplexNum) -> EscapeState,
{
    pub fn new(
        f: F,
        df_dz: G,
        check_periodicity: C,
        early_bailout: B,
        z: ComplexNum,
        param: ComplexNum,
    ) -> Self
    {
        let z_fast = f(z, param);
        let multiplier = df_dz(z_fast, param);
        Self {
            f,
            df_dz,
            check_periodicity,
            early_bailout,
            param,
            z_slow: z,
            z_fast,
            multiplier,
            iter: 0,
            state: EscapeState::NotYetEscaped,
            period_limit: 1,
            period: 1,
        }
    }

    fn apply_map(&self, z: ComplexNum) -> ComplexNum
    {
        (self.f)(z, self.param)
    }

    fn check_early_bailout(&mut self)
    {
        self.state = (self.early_bailout)(self.z_slow, self.param);
    }

    fn derivative(&self, z: ComplexNum) -> ComplexNum
    {
        (self.df_dz)(z, self.param)
    }

    pub fn reset(&mut self, param: ComplexNum, start_point: ComplexNum)
    {
        self.param = param;
        self.z_slow = start_point;
        self.z_fast = start_point;
        self.multiplier = (1.).into();
        self.iter = 0;
        self.state = EscapeState::NotYetEscaped;
        self.period = 1;
        self.period_limit = 1;
    }

    pub fn run_until_complete(&mut self) -> EscapeState
    {
        self.check_early_bailout();

        while let EscapeState::NotYetEscaped = self.state
        {
            if self.period_limit == self.period
            {
                self.z_slow = self.z_fast;
                self.period_limit *= 2;
                self.period = 0;
            }
            self.z_fast = self.apply_map(self.z_fast);
            self.multiplier *= self.derivative(self.z_fast);

            self.period += 1;
            self.iter += 1;

            self.state = (self.check_periodicity)(self.iter, self.z_slow, self.z_fast, self.param);
        }
        self.state
    }
}

impl<F, G, C, B> Iterator for CycleDetectedOrbitBrent<F, G, C, B>
where
    F: Fn(ComplexNum, ComplexNum) -> ComplexNum,
    G: Fn(ComplexNum, ComplexNum) -> ComplexNum,
    C: Fn(Period, ComplexNum, ComplexNum, ComplexNum) -> EscapeState,
    B: Fn(ComplexNum, ComplexNum) -> EscapeState,
{
    type Item = (ComplexNum, EscapeState);

    fn next(&mut self) -> Option<Self::Item>
    {
        if let EscapeState::NotYetEscaped = self.state
        {
            let retval = self.z_fast;
            if self.period_limit == self.period
            {
                self.z_slow = self.z_fast;
                self.period_limit *= 2;
                self.period = 0;
            }
            self.z_fast = self.apply_map(self.z_fast);
            self.multiplier *= self.derivative(self.z_fast);

            self.period += 1;
            self.iter += 1;

            self.state = (self.check_periodicity)(self.iter, self.z_slow, self.z_fast, self.param);
            Some((retval, self.state))
        }
        else
        {
            None
        }
    }
}
