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
    IdenticalNameAllowed {
        header: String
    }
}

impl fmt::Display for MergerOptionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IdenticalNameAllowed{header} => write!(f, "duplicate header.name '{}' is not allowed when identical_names_allowed is false",
                header)
        }
    }
}

impl std::error::Error for MergerOptionError {}

#[cfg(test)]
mod tests {
    use crate::merger::merge_from_files;
    use crate::merger::MergerError;

    use super::*;

    #[test]
    fn test_merge_options_default_allows_identical_names() {
        let options = MergeOptions::default();
        assert!(options.identical_names_allowed);
    }

    #[test]
    fn test_merge_rejects_identical_names_when_disabled() {
        let temp_a = std::env::temp_dir().join("aethellib_identical_names_a.toml");
        let temp_b = std::env::temp_dir().join("aethellib_identical_names_b.toml");

        std::fs::write(
            &temp_a,
            r#"
[header]
name = "same dataset"
target = "weapon"

[name]
prefix = ["Iron"]
"#,
        )
        .unwrap();

        std::fs::write(
            &temp_b,
            r#"
[header]
name = "same dataset"
target = "weapon"

[name]
suffix = ["of Dawn"]
"#,
        )
        .unwrap();

        let path_a = temp_a.to_string_lossy().to_string();
        let path_b = temp_b.to_string_lossy().to_string();
        let result = merge_from_files(
            &[path_a.as_str(), path_b.as_str()],
            Some(MergeOptions {
                identical_names_allowed: false,
            }),
        );

        assert!(matches!(
            result,
            Err(MergerError::MergerOption(MergerOptionError::IdenticalNameAllowed { .. }))
        ));

        std::fs::remove_file(temp_a).unwrap();
        std::fs::remove_file(temp_b).unwrap();
    }
}