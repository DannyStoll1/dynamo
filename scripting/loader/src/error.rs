use pyo3::PyErr;

#[derive(Debug)]
pub enum ScriptError
{
    MalformedConst,
    MalformedEquation,
    UndefinedSymbol,
    ForbiddenKeyword,
    CompilationFailed,
    PythonError(PyErr),
    ErrorWritingFile(std::io::Error),
    ErrorReadingToml(std::io::Error),
    ErrorParsingToml(toml::de::Error),
    ErrorMovingLibrary(std::io::Error),
    ErrorLoadingLibrary(libloading::Error),
    CargoCommandFailed(std::io::Error),
}
