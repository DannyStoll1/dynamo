use fractal_common::types::*;

trait DynamicalMap<Vars, Params>
{
    fn eval(&self, vars: Vars, params: Params) -> Vars;
}

impl<Func, V> DynamicalMap<V, ComplexNum> for Func
where
    Func: Fn(V, ComplexNum) -> V,
{
    fn eval(&self, z: V, c: ComplexNum) -> V
    {
        (self)(z, c)
    }
}

impl<Func, V, P> DynamicalMap<V, [P; 2]> for Func
where
    V: Copy,
    P: Copy,
    Func: Fn(V, P, P) -> V,
{
    fn eval(&self, z: V, params: [P; 2]) -> V
    {
        (self)(z, params[0], params[1])
    }
}

trait DynamicalMapGrad<Vars, Params>
{
    fn eval(&self, vars: Vars, params: Params) -> (Vars, Vars);
}

impl<Func, V> DynamicalMapGrad<V, ComplexNum> for Func
where
    Func: Fn(V, ComplexNum) -> (V, V),
{
    fn eval(&self, z: V, c: ComplexNum) -> (V, V)
    {
        (self)(z, c)
    }
}

impl<Func, V, P> DynamicalMapGrad<V, [P; 2]> for Func
where
    V: Copy,
    P: Copy,
    Func: Fn(V, P, P) -> (V, V),
{
    fn eval(&self, z: V, params: [P; 2]) -> (V, V)
    {
        (self)(z, params[0], params[1])
    }
}
