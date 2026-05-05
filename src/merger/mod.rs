//! central target-based merge orchestration for aethel documents.

pub mod error;
pub mod merger_options;
pub(self) mod utils;

use std::collections::HashMap;

use crate::loader::{AethelDoc, AthelDocHeader, Target, TargetedLoader, error::LoaderError};
use crate::merger::error::MergerError;
use crate::merger::utils::{
    build_corpus_from_paths, build_raw_corpus_from_sources, cast_aethel_docs_to_sources,
    parse_merge_inputs,
};
use merger_options::MergeOptions;

/// parsed source used by merge dispatch before target-specific ingestion.
struct ParsedMergeInput {
    path: String,
    raw: String,
    target: Target,
}

/// source payload used by target-specific corpus builders.
struct MergeSourceInput<'a> {
    /// original source path used for loading.
    pub path: &'a str,
    /// raw source content used for parsing and hashing.
    pub raw: &'a str,
}

/// untyped merged document body used for mixed-target merge flows.
pub type Mixed = toml::Table;

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

impl<T> SourceAethelDoc<T>
where
    T: TargetedLoader + serde::Serialize,
{
    /// casts one parsed aethel document into one source document.
    pub fn from_aetheldoc(document: AethelDoc<T>) -> Result<Self, MergerError> {
        let mut source_documents = cast_aethel_docs_to_sources::<T>(vec![document])?;
        Ok(source_documents.remove(0))
    }

    /// casts parsed aethel documents into source documents using merge hash/id rules.
    pub fn from_aetheldocs(documents: Vec<AethelDoc<T>>) -> Result<Vec<Self>, MergerError> {
        cast_aethel_docs_to_sources::<T>(documents)
    }
}

#[derive(Debug, Clone)]
/// per-target corpus retaining all source documents and metadata.
pub struct AethelCorpus<T> {
    /// target represented by all source documents.
    pub target: Target,
    /// source documents in first-seen order.
    pub documents: Vec<SourceAethelDoc<T>>,
}

impl<T> AethelCorpus<T> {
    /// returns the target represented by this corpus.
    pub fn target(&self) -> &str {
        self.target.as_str()
    }
}

impl AethelCorpus<Mixed> {
    /// consumes this value and converts source tables into a typed corpus.
    pub fn into_corpus<T>(self) -> Result<AethelCorpus<T>, MergerError>
    where
        T: TargetedLoader,
    {
        if self.target != T::TARGET {
            return Err(LoaderError::TargetMismatch {
                expected: T::TARGET.to_string(),
                found: self.target,
            }
            .into());
        }

        let mut documents: Vec<SourceAethelDoc<T>> = Vec::with_capacity(self.documents.len());

        for source in self.documents {
            let data: T = toml::Value::Table(source.data)
                .try_into()
                .map_err(|err| LoaderError::parse_for_path(source.source_path.as_str(), err))?;

            documents.push(SourceAethelDoc {
                source_id: source.source_id,
                source_hash: source.source_hash,
                source_path: source.source_path,
                header: source.header,
                data,
            });
        }

        Ok(AethelCorpus {
            target: T::TARGET.to_string(),
            documents,
        })
    }

    /// clones and converts source tables into a typed corpus.
    pub fn to_corpus<T>(&self) -> Result<AethelCorpus<T>, MergerError>
    where
        T: TargetedLoader,
    {
        self.clone().into_corpus::<T>()
    }
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
) -> Result<Vec<AethelCorpus<Mixed>>, MergerError> {
    if paths.is_empty() {
        return Err(MergerError::InvalidInput(
            "at least one path is required for merge".to_string(),
        ));
    }

    let options = opts.unwrap_or_default();

    let parsed_inputs = parse_merge_inputs(paths)?;
    let mut grouped_sources: HashMap<Target, Vec<MergeSourceInput<'_>>> = HashMap::new();
    let mut target_order: Vec<Target> = Vec::new();

    for input in &parsed_inputs {
        let sources = grouped_sources
            .entry(input.target.clone())
            .or_insert_with(|| {
                target_order.push(input.target.clone());
                Vec::new()
            });

        sources.push(MergeSourceInput {
            path: &input.path,
            raw: &input.raw,
        });
    }

    let mut merged_docs: Vec<AethelCorpus<Mixed>> = Vec::with_capacity(target_order.len());
    for target in target_order {
        if let Some(sources) = grouped_sources.remove(&target) {
            let corpus = build_raw_corpus_from_sources(&sources, Some(options))?;
            merged_docs.push(corpus);
        }
    }

    Ok(merged_docs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::{TARGET_PERSON, TARGET_WEAPON};
    use crate::merger::utils::build_corpus_from_sources;
    use crate::test_support::{TempTomlFile, person_document, weapon_document};
    use serde::Deserialize;

    #[derive(Deserialize, Debug, Clone)]
    struct TestWeaponLoader {
        name: Option<TestWeaponNameSection>,
    }

    #[derive(Deserialize, Debug, Clone)]
    struct TestWeaponNameSection {
        prefix: Option<Vec<String>>,
    }

    #[derive(Deserialize, Debug, Clone)]
    struct TestPersonLoader {
        name: Option<TestPersonNameSection>,
    }

    #[derive(Deserialize, Debug, Clone)]
    struct TestPersonNameSection {
        first: Option<Vec<String>>,
    }

    impl TargetedLoader for TestWeaponLoader {
        const TARGET: &'static str = TARGET_WEAPON;
    }

    impl TargetedLoader for TestPersonLoader {
        const TARGET: &'static str = TARGET_PERSON;
    }

    #[test]
    fn test_merge_requires_at_least_one_file() {
        let err = merge_from_files(&[], None).unwrap_err();
        assert!(matches!(err, MergerError::InvalidInput(_)));
    }

    #[test]
    fn test_merge_groups_files_by_target() {
        let one = TempTomlFile::new(&weapon_document(
            "part 1",
            r#"
[name]
prefix = ["iron"]
"#,
        ));
        let two = TempTomlFile::new(&weapon_document(
            "part 2",
            r#"
[name]
suffix = ["of dawn"]
"#,
        ));
        let paths = [one.path_str(), two.path_str()];

        let merged = merge_from_files(&paths, None).unwrap();

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].target(), TARGET_WEAPON);

        assert_eq!(merged[0].documents.len(), 2);
        assert!(
            merged[0]
                .documents
                .iter()
                .all(|source| source.header.target == TARGET_WEAPON)
        );
    }

    #[test]
    fn test_merge_groups_mixed_targets_in_first_seen_order() {
        let person = TempTomlFile::new(&person_document(
            "person set",
            r#"
[name]
first = ["al"]
"#,
        ));
        let weapon = TempTomlFile::new(&weapon_document(
            "weapon set",
            r#"
[name]
prefix = ["iron"]
"#,
        ));
        let paths = [person.path_str(), weapon.path_str()];

        let merged = merge_from_files(&paths, None).unwrap();

        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].target(), TARGET_PERSON);
        assert_eq!(merged[1].target(), TARGET_WEAPON);

        let person = merged[0].to_corpus::<TestPersonLoader>().unwrap();
        let weapon = merged[1].to_corpus::<TestWeaponLoader>().unwrap();

        assert_eq!(person.documents.len(), 1);
        assert_eq!(weapon.documents.len(), 1);

        assert!(
            weapon.documents[0]
                .data
                .name
                .as_ref()
                .and_then(|name| name.prefix.as_ref())
                .is_some()
        );
        assert!(
            person.documents[0]
                .data
                .name
                .as_ref()
                .and_then(|name| name.first.as_ref())
                .is_some()
        );
    }

    #[test]
    fn test_merge_allows_different_versions_between_sources() {
        let temp_a = TempTomlFile::new(
            r#"
[header]
name = "vendor weapon set"
target = "weapon"
version = "1.0"

[name]
prefix = ["Iron"]
"#,
        );

        let temp_b = TempTomlFile::new(
            r#"
[header]
name = "vendor weapon set"
target = "weapon"
version = "1.1"

[name]
suffix = ["of Dawn"]
"#,
        );

        let merged = merge_from_files(&[temp_a.path_str(), temp_b.path_str()], None).unwrap();
        assert_eq!(merged[0].documents.len(), 2);
        assert_eq!(
            merged[0].documents[0].header.version.as_deref(),
            Some("1.0")
        );
        assert_eq!(
            merged[0].documents[1].header.version.as_deref(),
            Some("1.1")
        );
    }

    #[test]
    fn test_merge_assigns_unique_source_ids_for_identical_content() {
        let content = r#"
[header]
name = "same content"
target = "weapon"

[name]
prefix = ["Iron"]
"#;

        let temp_a = TempTomlFile::new(content);
        let temp_b = TempTomlFile::new(content);
        let merged = merge_from_files(&[temp_a.path_str(), temp_b.path_str()], None).unwrap();

        assert_eq!(merged[0].documents.len(), 2);
        assert_ne!(
            merged[0].documents[0].source_id,
            merged[0].documents[1].source_id
        );
        assert_eq!(
            merged[0].documents[0].source_hash,
            merged[0].documents[1].source_hash
        );
    }

    #[test]
    fn test_merge_weapon_files_rejects_person_target_path() {
        let weapon = TempTomlFile::new(&weapon_document("weapon set", ""));
        let person = TempTomlFile::new(&person_document("person set", ""));
        let paths = [weapon.path_str(), person.path_str()];

        let result = merge_target_files::<TestWeaponLoader>(&paths, None);

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
    fn test_merged_aethel_doc_into_corpus_returns_weapon_variant() {
        let one = TempTomlFile::new(&weapon_document(
            "part 1",
            r#"
[name]
prefix = ["iron"]
"#,
        ));
        let paths = [one.path_str()];

        let merged_docs = merge_from_files(&paths, None).unwrap();
        let owned = merged_docs
            .into_iter()
            .next()
            .unwrap()
            .into_corpus::<TestWeaponLoader>()
            .unwrap();
        assert_eq!(owned.target, TARGET_WEAPON);
    }

    #[test]
    fn test_merged_aethel_doc_into_corpus_returns_person_variant() {
        let person = TempTomlFile::new(&person_document(
            "person set",
            r#"
[name]
first = ["al"]
"#,
        ));
        let paths = [person.path_str()];

        let merged_docs = merge_from_files(&paths, None).unwrap();
        let owned = merged_docs
            .into_iter()
            .next()
            .unwrap()
            .into_corpus::<TestPersonLoader>()
            .unwrap();
        assert_eq!(owned.target, TARGET_PERSON);
    }

    #[test]
    fn test_merged_aethel_doc_into_corpus_rejects_target_mismatch() {
        let person = TempTomlFile::new(&person_document(
            "person set",
            r#"
[name]
first = ["al"]
"#,
        ));
        let paths = [person.path_str()];

        let result = merge_from_files(&paths, None)
            .unwrap()
            .into_iter()
            .next()
            .unwrap()
            .into_corpus::<TestWeaponLoader>();

        assert!(matches!(
            result,
            Err(MergerError::Loader(LoaderError::TargetMismatch { expected, found }))
            if expected == TARGET_WEAPON && found == TARGET_PERSON
        ));
    }

    #[test]
    fn test_build_person_corpus_from_sources_matches_build_corpus_from_paths() {
        let person = TempTomlFile::new(&person_document(
            "person set",
            r#"
[name]
first = ["al"]
"#,
        ));
        let paths = [person.path_str()];

        let from_paths = build_corpus_from_paths::<TestPersonLoader>(&paths, None).unwrap();

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

        let from_sources =
            build_corpus_from_sources::<TestPersonLoader>(&source_inputs, None).unwrap();

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

        assert!(
            from_paths.documents[0]
                .data
                .name
                .as_ref()
                .and_then(|name| name.first.as_ref())
                .is_some()
        );
    }
}
