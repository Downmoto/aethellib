//! merge options and option-specific validation errors.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// configurable merge behaviour used by merge entrypoints.
pub struct MergeOptions {
    /// allows repeated `header.name` values across source files when enabled.
    pub identical_names_allowed: bool,
}

impl Default for MergeOptions {
    fn default() -> Self {
        Self {
            identical_names_allowed: true,
        }
    }
}

#[derive(Debug)]
pub enum MergerOptionError {
    IdenticalNameAllowed { header: String },
}

impl fmt::Display for MergerOptionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IdenticalNameAllowed { header } => write!(
                f,
                "duplicate header.name '{}' is not allowed when identical_names_allowed is false",
                header
            ),
        }
    }
}

impl std::error::Error for MergerOptionError {}
