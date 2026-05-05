//! loader-specific error types with optional source-path context.

use std::{fmt, path::Path};

use crate::loader::Target;

#[derive(Debug)]
/// errors that can happen while loading and validating a toml file.
pub enum LoaderError {
    /// file system read failure.
    ReadError {
        /// optional source file path if available.
        path: Option<String>,
        /// underlying io source error.
        source: std::io::Error,
    },
    /// toml deserialization failure.
    ParseError {
        /// optional source file path if available.
        path: Option<String>,
        /// underlying parse source error.
        source: toml::de::Error,
    },
    /// file target does not match the loader target.
    TargetMismatch { expected: Target, found: Target },
}

impl fmt::Display for LoaderError {
    /// formats loader errors for user-facing messages.
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
        }
    }
}

impl std::error::Error for LoaderError {}

impl From<std::io::Error> for LoaderError {
    /// converts io errors into loader read errors.
    fn from(value: std::io::Error) -> Self {
        LoaderError::ReadError {
            path: None,
            source: value,
        }
    }
}

impl From<toml::de::Error> for LoaderError {
    /// converts toml parsing errors into loader parse errors.
    fn from(value: toml::de::Error) -> Self {
        LoaderError::ParseError {
            path: None,
            source: value,
        }
    }
}

impl LoaderError {
    /// creates a read error with source path context.
    pub(crate) fn read_for_path(path: impl AsRef<Path>, source: std::io::Error) -> Self {
        LoaderError::ReadError {
            path: Some(path.as_ref().to_string_lossy().to_string()),
            source,
        }
    }

    /// creates a parse error with source path context.
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
    }

    #[test]
    fn test_parse_for_path_includes_path_in_message() {
        let parse_source = toml::from_str::<toml::Table>("[").unwrap_err();
        let error = LoaderError::parse_for_path("invalid.toml", parse_source);

        assert!(error.to_string().contains("invalid.toml"));
        assert!(matches!(
            error,
            LoaderError::ParseError { path: Some(_), .. }
        ));
    }
}
