use std::fmt::{Display, Formatter};

use pyo3::PyErr;

#[derive(Debug)]
pub enum ScriptError
{
    MalformedConst,
    MalformedEquation,
    UndefinedSymbol,
    ForbiddenKeyword,
    CompilationFailed,
    MissingDirectory,
    PythonError(PyErr),
    ErrorWritingFile(std::io::Error),
    ErrorReadingToml(std::io::Error),
    ErrorParsingToml(toml::de::Error),
    ErrorMovingLibrary(std::io::Error),
    ErrorLoadingLibrary(libloading::Error),
    CargoCommandFailed(std::io::Error),
}

impl From<PyErr> for ScriptError
{
    fn from(err: PyErr) -> Self
    {
        Self::PythonError(err)
    }
}
impl From<std::convert::Infallible> for ScriptError
{
    fn from(_err: std::convert::Infallible) -> Self
    {
        unreachable!()
    }
}

impl Display for ScriptError
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
    {
        match self {
            Self::MalformedConst => write!(f, "Malformed complex constant"),
            Self::MalformedEquation => write!(f, "Malformed symbolic equation"),
            Self::UndefinedSymbol => write!(f, "Script references an undefined symbol"),
            Self::ForbiddenKeyword => write!(f, "Script uses a forbidden keyword"),
            Self::CompilationFailed => write!(f, "Script compilation failed"),
            Self::MissingDirectory => write!(f, "Script directory is missing"),
            Self::PythonError(err) => write!(f, "Python error: {err}"),
            Self::ErrorWritingFile(err) => write!(f, "Failed to write script file: {err}"),
            Self::ErrorReadingToml(err) => write!(f, "Failed to read script file: {err}"),
            Self::ErrorParsingToml(err) => write!(f, "Failed to parse script TOML: {err}"),
            Self::ErrorMovingLibrary(err) => {
                write!(f, "Failed to move compiled script library: {err}")
            }
            Self::ErrorLoadingLibrary(err) => {
                write!(f, "Failed to load compiled script library: {err}")
            }
            Self::CargoCommandFailed(err) => {
                write!(f, "Failed to execute cargo while building script: {err}")
            }
        }
    }
}

impl std::error::Error for ScriptError {}
