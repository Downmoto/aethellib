//! unified loader error types used across all load entrypoints.

use std::{fmt, path::Path};

use crate::corpus::types::Target;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// stable machine-readable loader error categories.
pub enum CorpusLoaderErrorKind {
    Read,
    Parse,
    TargetMismatch,
    OptionViolation,
    InvalidInput,
    RuleParamsMissing,
}

#[derive(Debug)]
/// errors returned by all load entrypoints.
pub enum CorpusLoaderError {
    /// file system read failure.
    ReadError {
        /// optional source file path if available.
        path: Option<String>,
        /// underlying io source error.
        source: std::io::Error,
    },
    /// toml deserialization or structural parse failure.
    ParseError {
        /// optional source file path if available.
        path: Option<String>,
        /// underlying parse source error.
        source: toml::de::Error,
    },
    /// file target does not match the expected target.
    TargetMismatch { expected: Target, found: Target },
    /// a load option constraint was violated.
    OptionViolation(String),
    /// invalid arguments supplied to a load entrypoint.
    InvalidInput(String),
    /// rule has missing params
    RuleParamsMissing(String),
}

impl fmt::Display for CorpusLoaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CorpusLoaderError::ReadError { path, source } => {
                if let Some(path) = path {
                    write!(f, "unable to read toml file '{path}': {source}")
                } else {
                    write!(f, "unable to read toml file: {source}")
                }
            }
            CorpusLoaderError::ParseError { path, source } => {
                if let Some(path) = path {
                    write!(f, "unable to parse toml file '{path}': {source}")
                } else {
                    write!(f, "unable to parse toml file: {source}")
                }
            }
            CorpusLoaderError::TargetMismatch { expected, found } => {
                write!(f, "target mismatch: expected '{expected}', got '{found}'")
            }
            CorpusLoaderError::OptionViolation(msg) => write!(f, "{msg}"),
            CorpusLoaderError::InvalidInput(msg) => write!(f, "{msg}"),
            CorpusLoaderError::RuleParamsMissing(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for CorpusLoaderError {}

impl From<std::io::Error> for CorpusLoaderError {
    fn from(value: std::io::Error) -> Self {
        CorpusLoaderError::ReadError {
            path: None,
            source: value,
        }
    }
}

impl From<toml::de::Error> for CorpusLoaderError {
    fn from(value: toml::de::Error) -> Self {
        CorpusLoaderError::ParseError {
            path: None,
            source: value,
        }
    }
}

impl CorpusLoaderError {
    /// returns the stable machine-readable kind for this error.
    pub fn kind(&self) -> CorpusLoaderErrorKind {
        match self {
            CorpusLoaderError::ReadError { .. } => CorpusLoaderErrorKind::Read,
            CorpusLoaderError::ParseError { .. } => CorpusLoaderErrorKind::Parse,
            CorpusLoaderError::TargetMismatch { .. } => CorpusLoaderErrorKind::TargetMismatch,
            CorpusLoaderError::OptionViolation(_) => CorpusLoaderErrorKind::OptionViolation,
            CorpusLoaderError::InvalidInput(_) => CorpusLoaderErrorKind::InvalidInput,
            CorpusLoaderError::RuleParamsMissing(_) => CorpusLoaderErrorKind::RuleParamsMissing,
        }
    }

    pub(crate) fn read_for_path(path: impl AsRef<Path>, source: std::io::Error) -> Self {
        CorpusLoaderError::ReadError {
            path: Some(path.as_ref().to_string_lossy().to_string()),
            source,
        }
    }

    pub(crate) fn parse_for_path(path: impl AsRef<Path>, source: toml::de::Error) -> Self {
        CorpusLoaderError::ParseError {
            path: Some(path.as_ref().to_string_lossy().to_string()),
            source,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_formats_target_mismatch() {
        let error = CorpusLoaderError::TargetMismatch {
            expected: "weapon".to_string(),
            found: "person".to_string(),
        };

        assert_eq!(
            error.to_string(),
            "target mismatch: expected 'weapon', got 'person'"
        );
        assert_eq!(error.kind(), CorpusLoaderErrorKind::TargetMismatch);
    }

    #[test]
    fn test_read_for_path_includes_path_in_message() {
        let error = CorpusLoaderError::read_for_path(
            "custom-file.toml",
            std::io::Error::new(std::io::ErrorKind::NotFound, "missing"),
        );

        assert!(error.to_string().contains("custom-file.toml"));
        assert!(matches!(
            error,
            CorpusLoaderError::ReadError { path: Some(_), .. }
        ));
        assert_eq!(error.kind(), CorpusLoaderErrorKind::Read);
    }

    #[test]
    fn test_option_violation_kind() {
        let error = CorpusLoaderError::OptionViolation("some constraint".to_string());
        assert_eq!(error.to_string(), "some constraint");
        assert_eq!(error.kind(), CorpusLoaderErrorKind::OptionViolation);
    }

    #[test]
    fn test_invalid_input_kind() {
        let error = CorpusLoaderError::InvalidInput("bad input".to_string());
        assert_eq!(error.to_string(), "bad input");
        assert_eq!(error.kind(), CorpusLoaderErrorKind::InvalidInput);
    }
}
