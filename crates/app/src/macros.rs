macro_rules! interface {
    ($parent: ty) => {
        || create_interface(|| <$parent>::default(), JuliaSet::from)
    };
    ($parent: ty, $child: ident) => {
        || create_interface(|| <$parent>::default(), $child::from)
    };
    ($parent: ty, $covering: ident, $($periods: expr),+) => {
        || create_interface(|| <$parent>::default().$covering($($periods),+), JuliaSet::from)
    };
}

macro_rules! interface_mc {
    ($parent: ty, $period: expr) => {
        || {
            create_interface(
                || <$parent>::default().marked_cycle_curve($period),
                JuliaSet::from,
            )
        }
    };
}

macro_rules! interface_dyn {
    ($parent: ty, $period: expr) => {
        || {
            create_interface(
                || <$parent>::default().dynatomic_curve($period),
                JuliaSet::from,
            )
        }
    };
}

macro_rules! interface_mis {
    ($parent: ty, $preperiod: expr, $period: expr) => {
        || {
            create_interface(
                || <$parent>::default().misiurewicz_curve($preperiod, $period),
                JuliaSet::from,
            )
        }
    };
}

pub(crate) use {
    interface, interface_dyn, interface_mc, interface_mis,
};
