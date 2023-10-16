#[derive(Debug)]
pub enum UserScriptError
{
    MalformedConst,
    MalformedEquation,
    UndefinedSymbol,
    ForbiddenKeyword,
    CompilationFailed,
    ErrorWritingFile(std::io::Error),
    ErrorReadingToml(std::io::Error),
    ErrorParsingToml(toml::de::Error),
    ErrorLoadingLibrary(libloading::Error),
}
