use std::path::Path;
use std::sync::LazyLock;

pub static SCRIPT_PROJ_DIR: LazyLock<&'static Path> = LazyLock::new(|| Path::new("user_scripts"));
pub static DEFAULT_TEXT: LazyLock<String> =
    LazyLock::new(|| std::fs::read_to_string(SCRIPT_PROJ_DIR.join(".default.toml")).unwrap_or_default());
