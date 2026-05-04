//! error wrapper types used across merge entrypoints.

use std::fmt;

use crate::{loader::error::LoaderError, merger::merger_options::MergerOptionError};

#[derive(Debug)]
/// merge errors returned by the module-level merge entrypoint.
pub enum MergerError {
    /// wraps loader-level parse/read/target errors.
    Loader(LoaderError),
    /// wraps merger option errors.
    MergerOption(MergerOptionError),
    /// reports invalid merge arguments, such as an empty file list.
    InvalidInput(String),
}

impl fmt::Display for MergerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MergerError::Loader(err) => write!(f, "{err}"),
            MergerError::MergerOption(err) => write!(f, "{err}"),
            MergerError::InvalidInput(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for MergerError {}

impl From<LoaderError> for MergerError {
    fn from(value: LoaderError) -> Self {
        MergerError::Loader(value)
    }
}

impl From<MergerOptionError> for MergerError {
    fn from(value: MergerOptionError) -> Self {
        MergerError::MergerOption(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_for_invalid_input() {
        let error = MergerError::InvalidInput("bad input".to_string());
        assert_eq!(error.to_string(), "bad input");
    }

    #[test]
    fn test_loader_conversion_wraps_loader_error() {
        let loader_error = LoaderError::TargetMismatch {
            expected: "weapon".to_string(),
            found: "person".to_string(),
        };
        let wrapped: MergerError = loader_error.into();

        assert!(matches!(wrapped, MergerError::Loader(_)));
    }

    #[test]
    fn test_option_conversion_wraps_option_error() {
        let option_error = MergerOptionError::IdenticalNameAllowed {
            header: "same set".to_string(),
        };
        let wrapped: MergerError = option_error.into();

        assert!(matches!(wrapped, MergerError::MergerOption(_)));
    }
}