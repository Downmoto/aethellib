//! loader primitives for parsing and validating aethel source documents.

#[cfg(feature = "person-gen")]
pub mod loader_person;
#[cfg(feature = "weapon-gen")]
pub mod loader_weapon;

use serde::{Deserialize, de::DeserializeOwned};
use std::fmt;
use std::fs;
use std::path::Path;

/// open target identifier used by loaders, mergers, and generators.
pub type Target = String;

/// built-in target id for weapon schemas.
pub const TARGET_WEAPON: &str = "weapon";
/// built-in target id for person schemas.
pub const TARGET_PERSON: &str = "person";

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

pub trait TargetedLoader: Sized + DeserializeOwned {
    /// expected target for this loader implementation.
    const TARGET: &'static str;

    /// load, parse, and target-validate a single toml file.
    fn from_file(path: impl AsRef<Path>) -> Result<AethelDoc<Self>, LoaderError> {
        let path_ref = path.as_ref();
        let raw = fs::read_to_string(path_ref)
            .map_err(|source| LoaderError::read_for_path(path_ref, source))?;
        let parsed: AethelDoc<Self> = toml::from_str(&raw)
            .map_err(|source| LoaderError::parse_for_path(path_ref, source))?;

        if parsed.header.target != Self::TARGET {
            return Err(LoaderError::TargetMismatch {
                expected: Self::TARGET.to_string(),
                found: parsed.header.target.clone(),
            });
        }

        Ok(parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::loader_weapon::WeaponLoader;
    use std::fs;

    const FILE_PATH: &str = "data/weapon_test_data.toml";

    #[test]
    fn test_open_toml_file() {
        let content = fs::read_to_string(FILE_PATH);

        assert!(content.is_ok(), "Unable to read toml file at {FILE_PATH}")
    }

    #[test]
    fn test_read_toml_aethellib_header() {
        let content = fs::read_to_string(FILE_PATH).unwrap();

        let file: AethelDoc<toml::Table> = toml::from_str(&content).unwrap();

        assert_eq!(file.header.target, TARGET_WEAPON);
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

    #[test]
    fn test_loader_error_includes_path_for_read_failures() {
        let missing_path = "data/does_not_exist.toml";
        let err = WeaponLoader::from_file(missing_path).unwrap_err();

        assert!(err.to_string().contains(missing_path));
    }

    #[test]
    fn test_loader_error_includes_path_for_parse_failures() {
        let temp_path = std::env::temp_dir().join("aethellib_invalid_loader_input.toml");
        std::fs::write(&temp_path, "not valid toml = [").unwrap();

        let path_string = temp_path.to_string_lossy().to_string();
        let err = WeaponLoader::from_file(path_string.as_str()).unwrap_err();

        assert!(err.to_string().contains(path_string.as_str()));

        std::fs::remove_file(temp_path).unwrap();
    }
}
