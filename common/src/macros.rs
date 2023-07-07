macro_rules! max {
    ($x:expr) => ( $x );
    ($x:expr, $($xs:expr),+) => {
        std::cmp::max($x, max!( $($xs),+ ))
    };
}

macro_rules! min {
    ($x:expr) => ( $x );
    ($x:expr, $($xs:expr),+) => {
        std::cmp::min($x, min!( $($xs),+ ))
    };
}

#[macro_export]
macro_rules! horner {
    ($c: expr) => ( $c );
    ($var: expr, $c: expr ) => ( $c );
    ($var: expr, $c: expr, $($cs:expr),+) => {
        $c + $var * horner!($var, $($cs),+)
    };
}

#[macro_export]
macro_rules! horner_monic {
    () => ( 1. );
    ($c: expr) => ( $c );
    ($var: expr, $c: expr ) => ( $c + $var );
    ($var: expr, $c: expr, $($cs:expr),+) => {
        $c + $var * horner_monic!($var, $($cs),+)
    };
}

pub use {horner, horner_monic};
pub(crate) use {max, min};
