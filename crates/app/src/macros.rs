macro_rules! fractal_menu_button {
    ($self: ident, $ui: ident, $name: expr, $fractal: ty) => {
        if $ui.button($name).clicked()
        {
            $self.change_fractal(
                || <$fractal>::default(),
                <$fractal as DynamicalFamily>::Child::from,
            );
            $self.interface.consume_click();
            $ui.close_menu();
            return;
        }
    };
    ($self: ident, $ui: ident, $name: expr, $fractal: ty, $covering: ident, $($periods: expr),+) => {
        if $ui.button($name).clicked()
        {
            $self.change_fractal(|| <$fractal>::default().$covering($($periods),+), JuliaSet::from);
            $self.interface.consume_click();
            $ui.close_menu();
            return;
        }
    };
    ($self: ident, $ui: ident, $name: expr, $fractal: ident, $child: ident) => {
        if $ui.button($name).clicked()
        {
            $self.change_fractal(|| $fractal::default(), $child::from);
            $self.interface.consume_click();
            $ui.close_menu();
            return;
        }
    };
}

macro_rules! fractal_menu_button_mc {
    ($self: ident, $ui: ident, $fractal: ty, $period: expr) => {
        fractal_menu_button!(
            $self,
            $ui,
            format!("Period {}", $period),
            $fractal,
            marked_cycle_curve,
            $period
        );
    };
}

macro_rules! fractal_menu_button_dyn {
    ($self: ident, $ui: ident, $fractal: ty, $period: expr) => {
        fractal_menu_button!(
            $self,
            $ui,
            format!("Period {}", $period),
            $fractal,
            dynatomic_curve,
            $period
        );
    };
}

macro_rules! fractal_menu_button_mis {
    ($self: ident, $ui: ident, $fractal: ty, $preperiod: expr, $period: expr) => {
        fractal_menu_button!(
            $self,
            $ui,
            format!("Preperiod {}, Period {}", $preperiod, $period),
            $fractal,
            misiurewicz_curve,
            $preperiod,
            $period
        );
    };
}

pub(crate) use {
    fractal_menu_button, fractal_menu_button_dyn, fractal_menu_button_mc, fractal_menu_button_mis,
};
