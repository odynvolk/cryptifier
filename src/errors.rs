//! Centralized error handling for the Cryptifier application.
use std::fmt;

/// Main application error type.
#[derive(Debug)]
pub enum CryptifierError {
    NetworkError {
        source: String,
        message: String,
    },
    ParseError {
        context: String,
        message: String,
    },
    ConfigError {
        key: Option<String>,
        message: String,
    },
}

impl fmt::Display for CryptifierError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CryptifierError::NetworkError { source, message } => {
                write!(f, "Network error from {}: {}", source, message)
            }
            CryptifierError::ParseError { context, message } => {
                write!(f, "Parse error in {}: {}", context, message)
            }
            CryptifierError::ConfigError { key, message } => {
                if let Some(k) = key {
                    write!(f, "Config error for '{}': {}", k, message)
                } else {
                    write!(f, "Config error: {}", message)
                }
            }
        }
    }
}

impl std::error::Error for CryptifierError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl From<reqwest::Error> for CryptifierError {
    fn from(err: reqwest::Error) -> Self {
        CryptifierError::NetworkError {
            source: "reqwest".to_string(),
            message: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for CryptifierError {
    fn from(err: serde_json::Error) -> Self {
        CryptifierError::ParseError {
            context: "JSON".to_string(),
            message: err.to_string(),
        }
    }
}

impl From<std::env::VarError> for CryptifierError {
    fn from(err: std::env::VarError) -> Self {
        CryptifierError::ConfigError {
            key: None,
            message: err.to_string(),
        }
    }
}

/// Result type alias for functions that may return CryptifierError.
pub type Result<T> = std::result::Result<T, CryptifierError>;
