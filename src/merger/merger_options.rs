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

#[cfg(test)]
mod tests {
    use crate::merger::MergerError;
    use crate::merger::merge_from_files;
    use crate::test_support::{TempTomlFile, weapon_document};

    use super::*;

    #[test]
    fn test_merge_options_default_allows_identical_names() {
        let options = MergeOptions::default();
        assert!(options.identical_names_allowed);
    }

    #[test]
    fn test_merge_rejects_identical_names_when_disabled() {
        let temp_a = TempTomlFile::new(&weapon_document(
            "same dataset",
            r#"
[name]
prefix = ["Iron"]
"#,
        ));
        let temp_b = TempTomlFile::new(&weapon_document(
            "same dataset",
            r#"
[name]
suffix = ["of Dawn"]
"#,
        ));

        let result = merge_from_files(
            &[temp_a.path_str(), temp_b.path_str()],
            Some(MergeOptions {
                identical_names_allowed: false,
            }),
        );

        assert!(matches!(
            result,
            Err(MergerError::MergerOption(
                MergerOptionError::IdenticalNameAllowed { .. }
            ))
        ));
    }
}
