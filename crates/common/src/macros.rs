#[macro_export]
macro_rules! horner {
    ($c: expr_2021) => ( $c );
    ($var: expr_2021, $c: expr_2021 ) => ( $c );
    ($var: expr_2021, $c: expr_2021, $($cs:expr_2021),+) => {
        $c + $var * horner!($var, $($cs),+)
    };
}

#[macro_export]
macro_rules! horner_monic {
    () => ( 1. );
    ($c: expr_2021) => ( $c );
    ($var: expr_2021, $c: expr_2021 ) => ( $c + $var );
    ($var: expr_2021, $c: expr_2021, $($cs:expr_2021),+) => {
        $c + $var * horner_monic!($var, $($cs),+)
    };
}

#[macro_export]
macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

pub use {horner, horner_monic, regex};
