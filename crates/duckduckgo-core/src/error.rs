use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Usage error. ({0}) Invalid input. → Check the command syntax.")]
    Usage(String),
    #[error(
        "Cannot reach DuckDuckGo. ({0}) Network request failed. → Check connectivity, set --proxy, or increase --timeout."
    )]
    Network(String),
    #[error(
        "Unexpected DuckDuckGo response. ({0}) Remote response was not usable. → Retry later or file an issue if this persists."
    )]
    Remote(String),
    #[error(
        "Unexpected DuckDuckGo HTML structure. ({0}) Selectors may have changed. → Update duckduckgo-cli or file an issue with -vv output and the query used."
    )]
    Parse(String),
    #[error(
        "Search blocked by DuckDuckGo. ({0}) Anti-bot rate limit exceeded. → Wait 60s, route through --proxy, or reduce concurrency."
    )]
    Blocked(String),
    #[error(
        "Local I/O error. ({0}) Could not read or write local files. → Check permissions and available disk space."
    )]
    Io(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Usage(_) => 2,
            Self::Network(_) | Self::Remote(_) => 3,
            Self::Parse(_) => 4,
            Self::Blocked(_) => 5,
            Self::Io(_) => 6,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}
