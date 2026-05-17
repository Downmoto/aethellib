//! unified loader error types used across all load entrypoints.

use std::{fmt, path::Path};

use crate::Target;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// stable machine-readable loader error categories.
pub enum LoaderErrorKind {
    Read,
    Parse,
    TargetMismatch,
    OptionViolation,
    InvalidInput,
    RuleParamsMissing
}

#[derive(Debug)]
/// errors returned by all load entrypoints.
pub enum LoaderError {
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
    RuleParamsMissing(String)
}

impl fmt::Display for LoaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoaderError::ReadError { path, source } => {
                if let Some(path) = path {
                    write!(f, "unable to read toml file '{path}': {source}")
                } else {
                    write!(f, "unable to read toml file: {source}")
                }
            }
            LoaderError::ParseError { path, source } => {
                if let Some(path) = path {
                    write!(f, "unable to parse toml file '{path}': {source}")
                } else {
                    write!(f, "unable to parse toml file: {source}")
                }
            }
            LoaderError::TargetMismatch { expected, found } => {
                write!(f, "target mismatch: expected '{expected}', got '{found}'")
            }
            LoaderError::OptionViolation(msg) => write!(f, "{msg}"),
            LoaderError::InvalidInput(msg) => write!(f, "{msg}"),
            LoaderError::RuleParamsMissing(msg) => write!(f, "{msg}")
        }
    }
}

impl std::error::Error for LoaderError {}

impl From<std::io::Error> for LoaderError {
    fn from(value: std::io::Error) -> Self {
        LoaderError::ReadError {
            path: None,
            source: value,
        }
    }
}

impl From<toml::de::Error> for LoaderError {
    fn from(value: toml::de::Error) -> Self {
        LoaderError::ParseError {
            path: None,
            source: value,
        }
    }
}

impl LoaderError {
    /// returns the stable machine-readable kind for this error.
    pub fn kind(&self) -> LoaderErrorKind {
        match self {
            LoaderError::ReadError { .. } => LoaderErrorKind::Read,
            LoaderError::ParseError { .. } => LoaderErrorKind::Parse,
            LoaderError::TargetMismatch { .. } => LoaderErrorKind::TargetMismatch,
            LoaderError::OptionViolation(_) => LoaderErrorKind::OptionViolation,
            LoaderError::InvalidInput(_) => LoaderErrorKind::InvalidInput,
            LoaderError::RuleParamsMissing(_) => LoaderErrorKind::RuleParamsMissing
        }
    }

    pub(crate) fn read_for_path(path: impl AsRef<Path>, source: std::io::Error) -> Self {
        LoaderError::ReadError {
            path: Some(path.as_ref().to_string_lossy().to_string()),
            source,
        }
    }

    pub(crate) fn parse_for_path(path: impl AsRef<Path>, source: toml::de::Error) -> Self {
        LoaderError::ParseError {
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
        let error = LoaderError::TargetMismatch {
            expected: "weapon".to_string(),
            found: "person".to_string(),
        };

        assert_eq!(
            error.to_string(),
            "target mismatch: expected 'weapon', got 'person'"
        );
        assert_eq!(error.kind(), LoaderErrorKind::TargetMismatch);
    }

    #[test]
    fn test_read_for_path_includes_path_in_message() {
        let error = LoaderError::read_for_path(
            "custom-file.toml",
            std::io::Error::new(std::io::ErrorKind::NotFound, "missing"),
        );

        assert!(error.to_string().contains("custom-file.toml"));
        assert!(matches!(
            error,
            LoaderError::ReadError { path: Some(_), .. }
        ));
        assert_eq!(error.kind(), LoaderErrorKind::Read);
    }

    #[test]
    fn test_option_violation_kind() {
        let error = LoaderError::OptionViolation("some constraint".to_string());
        assert_eq!(error.to_string(), "some constraint");
        assert_eq!(error.kind(), LoaderErrorKind::OptionViolation);
    }

    #[test]
    fn test_invalid_input_kind() {
        let error = LoaderError::InvalidInput("bad input".to_string());
        assert_eq!(error.to_string(), "bad input");
        assert_eq!(error.kind(), LoaderErrorKind::InvalidInput);
    }
}
