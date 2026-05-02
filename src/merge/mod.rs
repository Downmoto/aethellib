//! central target-based merge orchestration for aethel documents.

pub mod merge_weapon;

use std::fmt;
use std::fs;

use crate::loader::loader_weapon::WeaponLoader;
use crate::loader::{AethelDoc, LoaderError, Target};

#[derive(Debug)]
/// merge errors returned by the module-level merge entrypoint.
pub enum MergeError {
    /// wraps loader-level parse/read/target errors.
    Loader(LoaderError),
    /// reports invalid merge arguments, such as an empty file list.
    InvalidInput(String),
    /// reports a target found in input files that has no registered merger yet.
    UnsupportedTarget(Target),
}

impl fmt::Display for MergeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MergeError::Loader(err) => write!(f, "{err}"),
            MergeError::InvalidInput(msg) => write!(f, "{msg}"),
            MergeError::UnsupportedTarget(target) => {
                write!(f, "unsupported merge target: {target:?}")
            }
        }
    }
}

impl std::error::Error for MergeError {}

impl From<LoaderError> for MergeError {
    fn from(value: LoaderError) -> Self {
        MergeError::Loader(value)
    }
}

#[derive(Debug)]
/// merged document variants keyed by target type.
pub enum MergedAethelDoc {
    /// merged output for weapon-target files.
    Weapon(AethelDoc<WeaponLoader>),
}

impl MergedAethelDoc {
    /// returns the target represented by this merged document variant.
    pub fn target(&self) -> Target {
        match self {
            MergedAethelDoc::Weapon(_) => Target::Weapon,
        }
    }
}

/// merges a mixed list of toml files into one merged document per discovered target.
///
/// files are first grouped by `header.target` while preserving first-seen target order,
/// then dispatched to each target-specific merger.
pub fn merge_from_files(paths: &[&str]) -> Result<Vec<MergedAethelDoc>, MergeError> {
    if paths.is_empty() {
        return Err(MergeError::InvalidInput(
            "at least one path is required for merge".to_string(),
        ));
    }

    let mut grouped_paths: Vec<(Target, Vec<&str>)> = Vec::new();

    for path in paths {
        let raw = fs::read_to_string(path).map_err(LoaderError::from)?;
        let parsed: AethelDoc<toml::Table> = toml::from_str(&raw).map_err(LoaderError::from)?;

        if let Some((_, matching_group_paths)) =
            grouped_paths.iter_mut().find(|(target, _)| *target == parsed.header.target)
        {
            matching_group_paths.push(*path);
        } else {
            grouped_paths.push((parsed.header.target, vec![*path]));
        }
    }

    let mut merged_docs = Vec::with_capacity(grouped_paths.len());

    for (target, target_paths) in grouped_paths {
        match target {
            Target::Weapon => {
                let merged = merge_weapon::merge_weapon_files(&target_paths)?;
                merged_docs.push(MergedAethelDoc::Weapon(merged));
            }
            Target::Person => {
                return Err(MergeError::UnsupportedTarget(Target::Person));
            }
        }
    }

    Ok(merged_docs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_requires_at_least_one_file() {
        let err = merge_from_files(&[]).unwrap_err();
        assert!(matches!(err, MergeError::InvalidInput(_)));
    }

    #[test]
    fn test_merge_groups_files_by_target() {
        let paths = [
            "data/weapon_merge_part_1.toml",
            "data/weapon_merge_part_2.toml",
            "data/weapon_merge_part_3.toml",
            "data/weapon_merge_part_4.toml",
        ];

        let merged = merge_from_files(&paths).unwrap();

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].target(), Target::Weapon);

        match &merged[0] {
            MergedAethelDoc::Weapon(doc) => {
                let name = doc.data.name.as_ref().unwrap();
                assert_eq!(name.prefix.as_ref().unwrap(), &vec!["Iron", "Steel"]);
                assert_eq!(name.suffix.as_ref().unwrap(), &vec!["of the Dawn", "of the Dusk"]);
                assert_eq!(name.primitives.as_ref().unwrap(), &vec!["ka", "li"]);
            }
        }
    }

    #[test]
    fn test_merge_reports_unsupported_target() {
        let temp_path = std::env::temp_dir().join("aethellib_person_target_test.toml");
        let content = r#"
[header]
name = "person set"
target = "person"
"#;

        std::fs::write(&temp_path, content).unwrap();

        let path_string = temp_path.to_string_lossy().to_string();
        let result = merge_from_files(&[path_string.as_str()]);

        assert!(matches!(result, Err(MergeError::UnsupportedTarget(Target::Person))));

        std::fs::remove_file(temp_path).unwrap();
    }
}