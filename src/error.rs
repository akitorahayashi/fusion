use std::error::Error;
use std::fmt::{self, Display};
use std::io;

/// Library-wide error type capturing domain-neutral and underlying I/O failures.
#[derive(Debug)]
pub enum AppError {
    Io(io::Error),
    /// Configuration or environment issue that prevents command execution.
    ConfigError(String),
    /// Process lifecycle failure tied to a specific managed service.
    ProcessError {
        service: String,
        message: String,
    },
}

impl Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io(err) => write!(f, "{}", err),
            AppError::ConfigError(message) => write!(f, "{message}"),
            AppError::ProcessError { service, message } => {
                write!(f, "Service '{service}' error: {message}")
            }
        }
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AppError::Io(err) => Some(err),
            AppError::ConfigError(_) | AppError::ProcessError { .. } => None,
        }
    }
}

impl From<io::Error> for AppError {
    fn from(value: io::Error) -> Self {
        AppError::Io(value)
    }
}

impl AppError {
    pub(crate) fn config_error<S: Into<String>>(message: S) -> Self {
        AppError::ConfigError(message.into())
    }

    pub(crate) fn process_error<S: Into<String>, M: Into<String>>(service: S, message: M) -> Self {
        AppError::ProcessError { service: service.into(), message: message.into() }
    }

    /// Provide an `io::ErrorKind`-like view for callers expecting legacy behavior.
    pub fn kind(&self) -> io::ErrorKind {
        match self {
            AppError::Io(err) => err.kind(),
            AppError::ConfigError(_) => io::ErrorKind::InvalidInput,
            AppError::ProcessError { .. } => io::ErrorKind::Other,
        }
    }
}
