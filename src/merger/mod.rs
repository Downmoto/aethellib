//! central target-based merge orchestration for aethel documents.

pub mod merger_options;

use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs;

use crate::loader::loader_person::PersonLoader;
use crate::loader::loader_weapon::WeaponLoader;
use crate::loader::{AethelDoc, AthelDocHeader, LoaderError, TARGET_PERSON, TARGET_WEAPON, Target, TargetedLoader};
use crate::merger::merger_options::MergerOptionError;
use merger_options::MergeOptions;

/// parsed source used by merge dispatch before target-specific ingestion.
struct ParsedMergeInput {
    path: String,
    raw: String,
    target: Target,
}

/// source payload used by target-specific corpus builders.
pub(crate) struct MergeSourceInput<'a> {
    /// original source path used for loading.
    pub path: &'a str,
    /// raw source content used for parsing and hashing.
    pub raw: &'a str,
}

#[derive(Debug)]
/// merge errors returned by the module-level merge entrypoint.
pub enum MergerError {
    /// wraps loader-level parse/read/target errors.
    Loader(LoaderError),
    /// wraps merger option errors.
    MergerOption(MergerOptionError),
    /// reports invalid merge arguments, such as an empty file list.
    InvalidInput(String),
    /// reports a target found in input files that has no registered merger yet.
    UnsupportedTarget(Target),
}

impl fmt::Display for MergerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MergerError::Loader(err) => write!(f, "{err}"),
            MergerError::MergerOption(err) => write!(f, "{err}"),
            MergerError::InvalidInput(msg) => write!(f, "{msg}"),
            MergerError::UnsupportedTarget(target) => {
                write!(f, "unsupported merge target: {target:?}")
            }
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
    /// corpus output for person-target files.
    Person(AethelCorpus<PersonLoader>),
    /// corpus output for weapon-target files.
    Weapon(AethelCorpus<WeaponLoader>),
}

impl MergedAethelDoc {
    /// returns the target represented by this merged document variant.
    pub fn target(&self) -> &'static str {
        match self {
            MergedAethelDoc::Person(_) => TARGET_PERSON,
            MergedAethelDoc::Weapon(_) => TARGET_WEAPON,
        }
    }

    /// returns a shared reference to a person corpus when this variant is person.
    pub fn as_person(&self) -> Option<&AethelCorpus<PersonLoader>> {
        match self {
            MergedAethelDoc::Person(corpus) => Some(corpus),
            MergedAethelDoc::Weapon(_) => None,
        }
    }

    /// consumes this value and returns a person corpus when this variant is person.
    pub fn into_person(self) -> Option<AethelCorpus<PersonLoader>> {
        match self {
            MergedAethelDoc::Person(corpus) => Some(corpus),
            MergedAethelDoc::Weapon(_) => None,
        }
    }

    /// returns a shared reference to a weapon corpus when this variant is weapon.
    pub fn as_weapon(&self) -> Option<&AethelCorpus<WeaponLoader>> {
        match self {
            MergedAethelDoc::Person(_) => None,
            MergedAethelDoc::Weapon(corpus) => Some(corpus),
        }
    }

    /// consumes this value and returns a weapon corpus when this variant is weapon.
    pub fn into_weapon(self) -> Option<AethelCorpus<WeaponLoader>> {
        match self {
            MergedAethelDoc::Person(_) => None,
            MergedAethelDoc::Weapon(corpus) => Some(corpus),
        }
    }
}

/// assembles a weapon corpus while preserving file-specific metadata.
pub fn merge_weapon_files(
    paths: &[&str],
    opts: Option<MergeOptions>,
) -> Result<AethelCorpus<WeaponLoader>, MergerError> {
    build_corpus_from_paths::<WeaponLoader>(paths, opts)
}

/// assembles a person corpus while preserving file-specific metadata.
pub fn merge_person_files(
    paths: &[&str],
    opts: Option<MergeOptions>,
) -> Result<AethelCorpus<PersonLoader>, MergerError> {
    build_corpus_from_paths::<PersonLoader>(paths, opts)
}

/// assembles a corpus for any loader that implements `TargetedLoader`.
pub fn merge_target_files<T>(
    paths: &[&str],
    opts: Option<MergeOptions>,
) -> Result<AethelCorpus<T>, MergerError>
where
    T: TargetedLoader,
{
    build_corpus_from_paths::<T>(paths, opts)
}

/// merges a mixed list of toml files into one merged document per discovered target.
///
/// files are first grouped by `header.target` while preserving first-seen target order,
/// then dispatched to each target-specific merger.
pub fn merge_from_files(
    paths: &[&str],
    opts: Option<MergeOptions>,
) -> Result<Vec<MergedAethelDoc>, MergerError> {
    if paths.is_empty() {
        return Err(MergerError::InvalidInput(
            "at least one path is required for merge".to_string(),
        ));
    }

    let options = opts.unwrap_or_default();

    let parsed_inputs = parse_merge_inputs(paths)?;
    let mut person_sources: Vec<MergeSourceInput<'_>> = Vec::new();
    let mut weapon_sources: Vec<MergeSourceInput<'_>> = Vec::new();
    let mut target_order: Vec<&'static str> = Vec::with_capacity(2);
    let mut seen_person = false;
    let mut seen_weapon = false;

    for input in &parsed_inputs {
        match input.target.as_str() {
            TARGET_PERSON => person_sources.push(MergeSourceInput {
                path: &input.path,
                raw: &input.raw,
            }),
            TARGET_WEAPON => weapon_sources.push(MergeSourceInput {
                path: &input.path,
                raw: &input.raw,
            }),
            _ => return Err(MergerError::UnsupportedTarget(input.target.clone())),
        }

        match input.target.as_str() {
            TARGET_PERSON if !seen_person => {
                seen_person = true;
                target_order.push(TARGET_PERSON);
            }
            TARGET_WEAPON if !seen_weapon => {
                seen_weapon = true;
                target_order.push(TARGET_WEAPON);
            }
            _ => {}
        }
    }

    let mut merged_docs: Vec<MergedAethelDoc> = Vec::with_capacity(target_order.len());
    for target in target_order {
        match target {
            TARGET_PERSON => {
                let merged =
                    build_corpus_from_sources::<PersonLoader>(&person_sources, Some(options))?;
                merged_docs.push(MergedAethelDoc::Person(merged));
            }
            TARGET_WEAPON => {
                let merged =
                    build_corpus_from_sources::<WeaponLoader>(&weapon_sources, Some(options))?;
                merged_docs.push(MergedAethelDoc::Weapon(merged));
            }
            _ => {
                return Err(MergerError::UnsupportedTarget(target.to_string()));
            }
        }
    }

    Ok(merged_docs)
}

/// parses source files once so dispatch and target ingestion share the same payload.
fn parse_merge_inputs(paths: &[&str]) -> Result<Vec<ParsedMergeInput>, MergerError> {
    let mut parsed_inputs = Vec::with_capacity(paths.len());

    for path in paths {
        let raw =
            fs::read_to_string(path).map_err(|source| LoaderError::read_for_path(path, source))?;
        let parsed: AethelDoc<toml::Table> =
            toml::from_str(&raw).map_err(|source| LoaderError::parse_for_path(path, source))?;

        parsed_inputs.push(ParsedMergeInput {
            path: (*path).to_string(),
            raw,
            target: parsed.header.target,
        });
    }

    Ok(parsed_inputs)
}

/// loads and validates source files for one target, then assembles a corpus.
pub(crate) fn build_corpus_from_paths<T>(
    paths: &[&str],
    opts: Option<MergeOptions>,
) -> Result<AethelCorpus<T>, MergerError>
where
    T: TargetedLoader,
{
    if paths.is_empty() {
        return Err(MergerError::InvalidInput(
            "at least one path is required for merge".to_string(),
        ));
    }

    let mut sources: Vec<(String, String)> = Vec::with_capacity(paths.len());

    for path in paths {
        let raw =
            fs::read_to_string(path).map_err(|source| LoaderError::read_for_path(path, source))?;
        sources.push(((*path).to_string(), raw));
    }

    let source_refs: Vec<MergeSourceInput<'_>> = sources
        .iter()
        .map(|(path, raw)| MergeSourceInput {
            path: path.as_str(),
            raw: raw.as_str(),
        })
        .collect();

    build_corpus_from_sources::<T>(&source_refs, opts)
}

/// assembles a target corpus from already-loaded source payloads.
pub(crate) fn build_corpus_from_sources<T>(
    sources: &[MergeSourceInput<'_>],
    opts: Option<MergeOptions>,
) -> Result<AethelCorpus<T>, MergerError>
where
    T: TargetedLoader,
{
    if sources.is_empty() {
        return Err(MergerError::InvalidInput(
            "at least one path is required for merge".to_string(),
        ));
    }

    let options = opts.unwrap_or_default();

    let mut seen_source_ids: HashMap<String, usize> = HashMap::new();
    let mut seen_header_names: HashSet<String> = HashSet::new();
    let mut documents: Vec<SourceAethelDoc<T>> = Vec::with_capacity(sources.len());

    for source in sources {
        let parsed: AethelDoc<T> = toml::from_str(source.raw)
            .map_err(|err| LoaderError::parse_for_path(source.path, err))?;

        if parsed.header.target != T::TARGET {
            return Err(LoaderError::TargetMismatch {
                expected: T::TARGET.to_string(),
                found: parsed.header.target.clone(),
            }
            .into());
        }

        if !options.identical_names_allowed && !seen_header_names.insert(parsed.header.name.clone())
        {
            return Err(MergerOptionError::IdenticalNameAllowed {
                header: parsed.header.name,
            }
            .into());
        }

        let source_hash = hash_source_content(parsed.header.target.as_str(), source.raw);
        let source_id = make_unique_source_id(&source_hash, &mut seen_source_ids);

        documents.push(SourceAethelDoc {
            source_id,
            source_hash,
            source_path: source.path.to_string(),
            header: parsed.header,
            data: parsed.data,
        });
    }

    Ok(AethelCorpus {
        target: T::TARGET.to_string(),
        documents,
    })
}

/// creates a unique source id from a base hash within one corpus build.
fn make_unique_source_id(base_hash: &str, seen: &mut HashMap<String, usize>) -> String {
    let count = seen.entry(base_hash.to_string()).or_insert(0);
    *count += 1;

    if *count == 1 {
        base_hash.to_string()
    } else {
        format!("{base_hash}:{count}")
    }
}

/// hashes canonicalized source content with target context for stable identity.
fn hash_source_content(target: &str, raw: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("{target}\n"));
    hasher.update(canonicalize_raw(raw));
    format!("{:x}", hasher.finalize())
}

/// normalizes source text before hashing to reduce platform-specific diffs.
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
    use crate::loader::loader_weapon::WeaponLoader;

    #[test]
    fn test_merge_requires_at_least_one_file() {
        let err = merge_from_files(&[], None).unwrap_err();
        assert!(matches!(err, MergerError::InvalidInput(_)));
    }

    #[test]
    fn test_merge_groups_files_by_target() {
        let paths = [
            "data/weapon_merge_part_1.toml",
            "data/weapon_merge_part_2.toml",
            "data/weapon_merge_part_3.toml",
            "data/weapon_merge_part_4.toml",
        ];

        let merged = merge_from_files(&paths, None).unwrap();

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].target(), TARGET_WEAPON);

        match &merged[0] {
            MergedAethelDoc::Weapon(doc) => {
                assert_eq!(doc.documents.len(), 4);
                assert!(
                    doc.documents
                        .iter()
                        .all(|source| source.header.target == TARGET_WEAPON)
                );
            }
            _ => panic!("expected weapon corpus"),
        }
    }

    #[test]
    fn test_merge_groups_mixed_targets_in_first_seen_order() {
        let paths = ["data/person_test_data.toml", "data/weapon_test_data.toml"];

        let merged = merge_from_files(&paths, None).unwrap();

        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].target(), TARGET_PERSON);
        assert_eq!(merged[1].target(), TARGET_WEAPON);

        let person = merged[0].as_person().unwrap();
        let weapon = merged[1].as_weapon().unwrap();

        assert_eq!(person.documents.len(), 1);
        assert_eq!(weapon.documents.len(), 1);
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

        let merged = merge_from_files(&[path_a.as_str(), path_b.as_str()], None).unwrap();
        match &merged[0] {
            MergedAethelDoc::Weapon(doc) => {
                assert_eq!(doc.documents.len(), 2);
                assert_eq!(doc.documents[0].header.version.as_deref(), Some("1.0"));
                assert_eq!(doc.documents[1].header.version.as_deref(), Some("1.1"));
            }
            _ => panic!("expected weapon corpus"),
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
        let merged = merge_from_files(&[path_a.as_str(), path_b.as_str()], None).unwrap();

        match &merged[0] {
            MergedAethelDoc::Weapon(doc) => {
                assert_eq!(doc.documents.len(), 2);
                assert_ne!(doc.documents[0].source_id, doc.documents[1].source_id);
                assert_eq!(doc.documents[0].source_hash, doc.documents[1].source_hash);
            }
            _ => panic!("expected weapon corpus"),
        }

        std::fs::remove_file(temp_a).unwrap();
        std::fs::remove_file(temp_b).unwrap();
    }

    #[test]
    fn test_merge_weapon_files_rejects_person_target_path() {
        let paths = ["data/weapon_test_data.toml", "data/person_test_data.toml"];

        let result = merge_weapon_files(&paths, None);

        assert!(matches!(
            result,
            Err(MergerError::Loader(LoaderError::TargetMismatch {
                expected,
                found,
            }))
            if expected == TARGET_WEAPON && found == TARGET_PERSON
        ));
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
        let result = merge_from_files(&[path_string.as_str()], None);

        assert!(matches!(
            result,
            Err(MergerError::UnsupportedTarget(target)) if target == "unsupported"
        ));

        std::fs::remove_file(temp_path).unwrap();
    }

    #[test]
    fn test_build_corpus_from_sources_matches_build_corpus_from_paths() {
        let paths = [
            "data/weapon_merge_part_1.toml",
            "data/weapon_merge_part_2.toml",
            "data/weapon_merge_part_3.toml",
            "data/weapon_merge_part_4.toml",
        ];

        let from_paths = build_corpus_from_paths::<WeaponLoader>(&paths, None).unwrap();

        let loaded_sources: Vec<(String, String)> = paths
            .iter()
            .map(|path| ((*path).to_string(), std::fs::read_to_string(path).unwrap()))
            .collect();

        let source_inputs: Vec<MergeSourceInput<'_>> = loaded_sources
            .iter()
            .map(|(path, raw)| MergeSourceInput {
                path: path.as_str(),
                raw: raw.as_str(),
            })
            .collect();

        let from_sources = build_corpus_from_sources::<WeaponLoader>(&source_inputs, None).unwrap();

        assert_eq!(from_paths.target, from_sources.target);
        assert_eq!(from_paths.documents.len(), from_sources.documents.len());

        for (left, right) in from_paths
            .documents
            .iter()
            .zip(from_sources.documents.iter())
        {
            assert_eq!(left.source_id, right.source_id);
            assert_eq!(left.source_hash, right.source_hash);
            assert_eq!(left.source_path, right.source_path);
            assert_eq!(left.header.name, right.header.name);
            assert_eq!(left.header.target, right.header.target);
            assert_eq!(left.header.version, right.header.version);
        }
    }

    #[test]
    fn test_merged_aethel_doc_accessors_return_weapon_variant() {
        let paths = [
            "data/weapon_merge_part_1.toml",
            "data/weapon_merge_part_2.toml",
            "data/weapon_merge_part_3.toml",
            "data/weapon_merge_part_4.toml",
        ];

        let merged_docs = merge_from_files(&paths, None).unwrap();
        let weapon_ref = merged_docs[0].as_weapon();
        assert!(weapon_ref.is_some());
        assert_eq!(weapon_ref.unwrap().target, TARGET_WEAPON);

        let owned = merged_docs.into_iter().next().unwrap().into_weapon();
        assert!(owned.is_some());
        assert_eq!(owned.unwrap().target, TARGET_WEAPON);
    }

    #[test]
    fn test_merged_aethel_doc_accessors_return_person_variant() {
        let paths = ["data/person_test_data.toml"];

        let merged_docs = merge_from_files(&paths, None).unwrap();
        let person_ref = merged_docs[0].as_person();
        assert!(person_ref.is_some());
        assert_eq!(person_ref.unwrap().target, TARGET_PERSON);

        let owned = merged_docs.into_iter().next().unwrap().into_person();
        assert!(owned.is_some());
        assert_eq!(owned.unwrap().target, TARGET_PERSON);
    }

    #[test]
    fn test_build_person_corpus_from_sources_matches_build_corpus_from_paths() {
        let paths = ["data/person_test_data.toml"];

        let from_paths = build_corpus_from_paths::<PersonLoader>(&paths, None).unwrap();

        let loaded_sources: Vec<(String, String)> = paths
            .iter()
            .map(|path| ((*path).to_string(), std::fs::read_to_string(path).unwrap()))
            .collect();

        let source_inputs: Vec<MergeSourceInput<'_>> = loaded_sources
            .iter()
            .map(|(path, raw)| MergeSourceInput {
                path: path.as_str(),
                raw: raw.as_str(),
            })
            .collect();

        let from_sources = build_corpus_from_sources::<PersonLoader>(&source_inputs, None).unwrap();

        assert_eq!(from_paths.target, from_sources.target);
        assert_eq!(from_paths.documents.len(), from_sources.documents.len());
        assert_eq!(
            from_paths.documents[0].source_id,
            from_sources.documents[0].source_id
        );
        assert_eq!(
            from_paths.documents[0].source_hash,
            from_sources.documents[0].source_hash
        );
        assert_eq!(
            from_paths.documents[0].source_path,
            from_sources.documents[0].source_path
        );
        assert_eq!(
            from_paths.documents[0].header.name,
            from_sources.documents[0].header.name
        );
        assert_eq!(
            from_paths.documents[0].header.target,
            from_sources.documents[0].header.target
        );
    }
}
