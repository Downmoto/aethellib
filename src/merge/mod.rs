//! central target-based merge orchestration for aethel documents.

pub mod merge_weapon;

use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fmt;
use std::fs;

use crate::loader::loader_weapon::WeaponLoader;
use crate::loader::{AethelDoc, AthelDocHeader, LoaderError, Target, TargetedLoader};

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

#[derive(Debug, Clone)]
/// one source document retained in a target corpus.
pub struct SourceAethelDoc<T> {
    /// unique id for this source within a corpus instance.
    pub source_id: String,
    /// deterministic hash derived from canonicalized document content and target.
    pub source_hash: String,
    /// original source path used for loading.
    pub source_path: String,
    /// metadata from the source header.
    pub header: AthelDocHeader,
    /// source data body.
    pub data: T,
}

#[derive(Debug, Clone)]
/// per-target corpus retaining all source documents and metadata.
pub struct AethelCorpus<T> {
    /// target represented by all source documents.
    pub target: Target,
    /// source documents in first-seen order.
    pub documents: Vec<SourceAethelDoc<T>>,
}

#[derive(Debug)]
/// merged document variants keyed by target type.
pub enum MergedAethelDoc {
    /// corpus output for weapon-target files.
    Weapon(AethelCorpus<WeaponLoader>),
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

    let mut weapon_paths: Vec<&str> = Vec::new();

    for path in paths {
        let raw = fs::read_to_string(path).map_err(LoaderError::from)?;
        let parsed: AethelDoc<toml::Table> = toml::from_str(&raw).map_err(LoaderError::from)?;
        match parsed.header.target {
            Target::Weapon => weapon_paths.push(*path),
            Target::Person => return Err(MergeError::UnsupportedTarget(Target::Person)),
            Target::Unsupported => return Err(MergeError::UnsupportedTarget(Target::Unsupported))
        }
    }

    let mut merged_docs: Vec<MergedAethelDoc> = Vec::with_capacity(1);
    if !weapon_paths.is_empty() {
        let merged = merge_weapon::merge_weapon_files(&weapon_paths)?;
        merged_docs.push(MergedAethelDoc::Weapon(merged));
    }

    Ok(merged_docs)
}

pub(crate) fn build_corpus_from_paths<T>(paths: &[&str]) -> Result<AethelCorpus<T>, MergeError>
where
    T: TargetedLoader,
{
    if paths.is_empty() {
        return Err(MergeError::InvalidInput(
            "at least one path is required for merge".to_string(),
        ));
    }

    let mut seen_source_ids: HashMap<String, usize> = HashMap::new();
    let mut documents: Vec<SourceAethelDoc<T>> = Vec::with_capacity(paths.len());

    for path in paths {
        let raw = fs::read_to_string(path).map_err(LoaderError::from)?;
        let parsed: AethelDoc<T> = toml::from_str(&raw).map_err(LoaderError::from)?;

        if parsed.header.target != T::TARGET {
            return Err(MergeError::Loader(LoaderError::TargetMismatch {
                expected: T::TARGET,
                found: parsed.header.target,
            }));
        }

        let source_hash = hash_source_content(parsed.header.target, &raw);
        let source_id = make_unique_source_id(&source_hash, &mut seen_source_ids);

        documents.push(SourceAethelDoc {
            source_id,
            source_hash,
            source_path: (*path).to_string(),
            header: parsed.header,
            data: parsed.data,
        });
    }

    Ok(AethelCorpus {
        target: T::TARGET,
        documents,
    })
}

fn make_unique_source_id(base_hash: &str, seen: &mut HashMap<String, usize>) -> String {
    let count = seen.entry(base_hash.to_string()).or_insert(0);
    *count += 1;

    if *count == 1 {
        base_hash.to_string()
    } else {
        format!("{base_hash}:{count}")
    }
}

fn hash_source_content(target: Target, raw: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("{target:?}\n"));
    hasher.update(canonicalize_raw(raw));
    format!("{:x}", hasher.finalize())
}

fn canonicalize_raw(raw: &str) -> String {
    raw.replace("\r\n", "\n")
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
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
                assert_eq!(doc.documents.len(), 4);
                assert!(doc
                    .documents
                    .iter()
                    .all(|source| source.header.target == Target::Weapon));
            }
        }
    }

    #[test]
    fn test_merge_allows_different_versions_between_sources() {
        let temp_a = std::env::temp_dir().join("aethellib_version_a.toml");
        let temp_b = std::env::temp_dir().join("aethellib_version_b.toml");

        std::fs::write(
            &temp_a,
            r#"
[header]
name = "vendor weapon set"
target = "weapon"
version = "1.0"

[name]
prefix = ["Iron"]
"#,
        )
        .unwrap();

        std::fs::write(
            &temp_b,
            r#"
[header]
name = "vendor weapon set"
target = "weapon"
version = "1.1"

[name]
suffix = ["of Dawn"]
"#,
        )
        .unwrap();

        let path_a = temp_a.to_string_lossy().to_string();
        let path_b = temp_b.to_string_lossy().to_string();

        let merged = merge_from_files(&[path_a.as_str(), path_b.as_str()]).unwrap();
        match &merged[0] {
            MergedAethelDoc::Weapon(doc) => {
                assert_eq!(doc.documents.len(), 2);
                assert_eq!(doc.documents[0].header.version.as_deref(), Some("1.0"));
                assert_eq!(doc.documents[1].header.version.as_deref(), Some("1.1"));
            }
        }

        std::fs::remove_file(temp_a).unwrap();
        std::fs::remove_file(temp_b).unwrap();
    }

    #[test]
    fn test_merge_assigns_unique_source_ids_for_identical_content() {
        let temp_a = std::env::temp_dir().join("aethellib_identical_a.toml");
        let temp_b = std::env::temp_dir().join("aethellib_identical_b.toml");

        let content = r#"
[header]
name = "same content"
target = "weapon"

[name]
prefix = ["Iron"]
"#;

        std::fs::write(&temp_a, content).unwrap();
        std::fs::write(&temp_b, content).unwrap();

        let path_a = temp_a.to_string_lossy().to_string();
        let path_b = temp_b.to_string_lossy().to_string();
        let merged = merge_from_files(&[path_a.as_str(), path_b.as_str()]).unwrap();

        match &merged[0] {
            MergedAethelDoc::Weapon(doc) => {
                assert_eq!(doc.documents.len(), 2);
                assert_ne!(doc.documents[0].source_id, doc.documents[1].source_id);
                assert_eq!(doc.documents[0].source_hash, doc.documents[1].source_hash);
            }
        }

        std::fs::remove_file(temp_a).unwrap();
        std::fs::remove_file(temp_b).unwrap();
    }

    #[test]
    fn test_merge_reports_unsupported_target() {
        let temp_path = std::env::temp_dir().join("aethellib_unsupported_target_test.toml");
        let content = r#"
[header]
name = "unsupported set"
target = "unsupported"
"#;

        std::fs::write(&temp_path, content).unwrap();

        let path_string = temp_path.to_string_lossy().to_string();
        let result = merge_from_files(&[path_string.as_str()]);

        assert!(matches!(result, Err(MergeError::UnsupportedTarget(Target::Unsupported))));

        std::fs::remove_file(temp_path).unwrap();
    }
}