use lazy_static::lazy_static;
use std::path::Path;

lazy_static! {
    pub static ref SCRIPT_DIR: &'static Path = Path::new("user_scripts");
    pub static ref DEFAULT_TEXT: String =
        std::fs::read_to_string(SCRIPT_DIR.join(".default.toml")).unwrap_or_default();
}
