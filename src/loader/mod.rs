//! loader primitives for parsing and validating aethel source documents.

pub mod loader_weapon;

use serde::{Deserialize, de::DeserializeOwned};
use std::fmt;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
/// supported target categories for loader dispatch.
pub enum Target {
    /// weapon dataset target.
    Weapon,
    /// person dataset target.
    Person,
    /// unsupported target, used in error handling.
    Unsupported
}

#[derive(Deserialize, Debug, Clone)]
/// common metadata required in each input file header.
pub struct AthelDocHeader {
    /// dataset display name.
    pub name: String,
    /// target category used for loader validation.
    pub target: Target,
    /// optional dataset description.
    pub desc: Option<String>,
    /// optional dataset author.
    pub author: Option<String>,
    /// optional dataset version.
    pub version: Option<String>,
}

#[derive(Deserialize, Debug)]
/// parsed toml payload with header plus target-specific body data.
pub struct AethelDoc<T> {
    /// parsed file header.
    pub header: AthelDocHeader,
    /// target-specific sections flattened from the same root document.
    #[serde(flatten)]
    pub data: T,
}

#[derive(Debug)]
/// errors that can happen while loading and validating a toml file.
pub enum LoaderError {
    /// file system read failure.
    ReadError(std::io::Error),
    /// toml deserialization failure.
    ParseError(toml::de::Error),
    /// file target does not match the loader target.
    TargetMismatch { expected: Target, found: Target },
}

impl fmt::Display for LoaderError {
    /// formats loader errors for user-facing messages.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoaderError::ReadError(err) => write!(f, "unable to read toml file: {err}"),
            LoaderError::ParseError(err) => write!(f, "unable to parse toml file: {err}"),
            LoaderError::TargetMismatch { expected, found } => {
                write!(f, "target mismatch: expected {expected:?}, got {found:?}")
            }
        }
    }
}

impl std::error::Error for LoaderError {}

impl From<std::io::Error> for LoaderError {
    /// converts io errors into loader read errors.
    fn from(value: std::io::Error) -> Self {
        LoaderError::ReadError(value)
    }
}

impl From<toml::de::Error> for LoaderError {
    /// converts toml parsing errors into loader parse errors.
    fn from(value: toml::de::Error) -> Self {
        LoaderError::ParseError(value)
    }
}

pub trait TargetedLoader: Sized + DeserializeOwned {
    /// expected target for this loader implementation.
    const TARGET: Target;

    /// load, parse, and target-validate a single toml file.
    fn from_file(path: impl AsRef<Path>) -> Result<AethelDoc<Self>, LoaderError> {
        let raw = fs::read_to_string(path)?;
        let parsed: AethelDoc<Self> = toml::from_str(&raw)?;

        if parsed.header.target != Self::TARGET {
            return Err(LoaderError::TargetMismatch {
                expected: Self::TARGET,
                found: parsed.header.target,
            });
        }

        Ok(parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    const FILE_PATH: &str = "data/weapon_test_data.toml";

    #[test]
    fn test_open_toml_file() {
        let content = fs::read_to_string(FILE_PATH);

        assert!(!content.is_err(), "Unable to read toml file at {FILE_PATH}")
    }

    #[test]
    fn test_read_toml_aethellib_header() {
        let content = fs::read_to_string(FILE_PATH).unwrap();

        let file: AethelDoc<toml::Table> = toml::from_str(&content).unwrap();

        assert_eq!(file.header.target, Target::Weapon);
        assert_eq!(file.header.name, "weapon test set");
        assert!(file.header.desc.is_some());
        assert!(file.header.author.is_some());
        assert!(file.header.version.is_some());
    }

    #[test]
    fn test_read_toml_missing_required_header_name_fails() {
        let content = r#"
[header]
target = "weapon"
"#;

        let file = toml::from_str::<AethelDoc<toml::Table>>(content);

        assert!(file.is_err());
    }

    #[test]
    fn test_read_toml_missing_required_header_target_fails() {
        let content = r#"
[header]
name = "example"
"#;

        let file = toml::from_str::<AethelDoc<toml::Table>>(content);

        assert!(file.is_err());
    }
}
