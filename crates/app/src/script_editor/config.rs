use std::path::Path;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref SCRIPT_PROJ_DIR: &'static Path = Path::new("user_scripts");
    pub static ref DEFAULT_TEXT: String =
        std::fs::read_to_string(SCRIPT_PROJ_DIR.join(".default.toml")).unwrap_or_default();
}
