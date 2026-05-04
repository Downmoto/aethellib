//! loader primitives for parsing and validating aethel source documents.

#[cfg(feature = "person-gen")]
pub mod loader_person;
#[cfg(feature = "weapon-gen")]
pub mod loader_weapon;
pub mod error;


use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::fs;
use std::path::Path;

use crate::loader::error::LoaderError;

/// open target identifier used by loaders, mergers, and generators.
pub type Target = String;

/// built-in target id for weapon schemas.
pub const TARGET_WEAPON: &str = "weapon";
/// built-in target id for person schemas.
pub const TARGET_PERSON: &str = "person";

#[derive(Deserialize, Serialize, Debug, Clone)]
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

#[derive(Deserialize, Serialize, Debug)]
/// parsed toml payload with header plus target-specific body data.
pub struct AethelDoc<T> {
    /// parsed file header.
    pub header: AthelDocHeader,
    /// target-specific sections flattened from the same root document.
    #[serde(flatten)]
    pub data: T,
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
    use crate::test_support::{TempTomlFile, toml_document};
    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    struct TestWeaponLoader;

    impl TargetedLoader for TestWeaponLoader {
        const TARGET: &'static str = TARGET_WEAPON;
    }

    #[test]
    fn test_read_toml_aethellib_header() {
        let content = r#"
[header]
name = "weapon test set"
target = "weapon"
desc = "fixture description"
author = "fixture author"
version = "1.0"

[name]
prefix = ["iron"]
"#;

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
    fn test_targeted_loader_reads_valid_file() {
        let temp = TempTomlFile::new(&toml_document("valid set", TARGET_WEAPON, ""));

        let loaded = TestWeaponLoader::from_file(temp.path_str()).unwrap();

        assert_eq!(loaded.header.target, TARGET_WEAPON);
    }

    #[test]
    fn test_targeted_loader_rejects_target_mismatch() {
        let temp = TempTomlFile::new(&toml_document("wrong target", TARGET_PERSON, ""));

        let err = TestWeaponLoader::from_file(temp.path_str()).unwrap_err();

        assert!(matches!(
            err,
            LoaderError::TargetMismatch { expected, found }
            if expected == TARGET_WEAPON && found == TARGET_PERSON
        ));
    }

    #[test]
    fn test_loader_error_includes_path_for_read_failures() {
        let missing_path = std::env::temp_dir()
            .join("aethellib_missing_loader_input.toml")
            .to_string_lossy()
            .to_string();
        let err = TestWeaponLoader::from_file(missing_path.as_str()).unwrap_err();

        assert!(err.to_string().contains(missing_path.as_str()));
    }

    #[test]
    fn test_loader_error_includes_path_for_parse_failures() {
        let temp = TempTomlFile::new("not valid toml = [");
        let err = TestWeaponLoader::from_file(temp.path_str()).unwrap_err();

        assert!(err.to_string().contains(temp.path_str()));
    }
}
