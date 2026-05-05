//! shared merge utilities for parsing, corpus assembly, and source identity.

use std::{
    collections::{HashMap, HashSet},
    fs,
};

use sha2::{Digest, Sha256};

use crate::{
    loader::{AethelDoc, Target, TargetedLoader, error::LoaderError},
    merger::{
        AethelCorpus, MergeSourceInput, Mixed, ParsedMergeInput, SourceAethelDoc,
        error::MergerError,
        merger_options::{MergeOptions, MergerOptionError},
    },
};

/// parses source files once so dispatch and target ingestion share the same payload.
pub fn parse_merge_inputs(paths: &[&str]) -> Result<Vec<ParsedMergeInput>, MergerError> {
    let mut parsed_inputs = Vec::with_capacity(paths.len());

    for path in paths {
        let raw =
            fs::read_to_string(path).map_err(|source| LoaderError::read_for_path(path, source))?;
        let parsed: AethelDoc<Mixed> =
            toml::from_str(&raw).map_err(|source| LoaderError::parse_for_path(path, source))?;

        parsed_inputs.push(ParsedMergeInput {
            path: (*path).to_string(),
            raw,
            target: parsed.header.target,
        });
    }

    Ok(parsed_inputs)
}

/// assembles an untyped target corpus from already-loaded source payloads.
pub fn build_raw_corpus_from_sources(
    sources: &[MergeSourceInput<'_>],
    opts: Option<MergeOptions>,
) -> Result<AethelCorpus<Mixed>, MergerError> {
    if sources.is_empty() {
        return Err(MergerError::InvalidInput(
            "at least one path is required for merge".to_string(),
        ));
    }

    let options = opts.unwrap_or_default();

    let mut seen_source_ids: HashMap<String, usize> = HashMap::new();
    let mut seen_header_names: HashSet<String> = HashSet::new();
    let mut documents: Vec<SourceAethelDoc<Mixed>> = Vec::with_capacity(sources.len());
    let mut target: Option<Target> = None;

    for source in sources {
        let parsed: AethelDoc<Mixed> = toml::from_str(source.raw)
            .map_err(|err| LoaderError::parse_for_path(source.path, err))?;

        if let Some(expected_target) = &target {
            if parsed.header.target != *expected_target {
                return Err(LoaderError::TargetMismatch {
                    expected: expected_target.clone(),
                    found: parsed.header.target.clone(),
                }
                .into());
            }
        } else {
            target = Some(parsed.header.target.clone());
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
        target: target.unwrap_or_default(),
        documents,
    })
}

/// loads and validates source files for one target, then assembles a corpus.
pub fn build_corpus_from_paths<T>(
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
pub fn build_corpus_from_sources<T>(
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

/// casts parsed aethel documents into source documents using merge hash/id rules.
pub fn cast_aethel_docs_to_sources<T>(
    documents: Vec<AethelDoc<T>>,
) -> Result<Vec<SourceAethelDoc<T>>, MergerError>
where
    T: TargetedLoader + serde::Serialize,
{
    let mut seen_source_ids: HashMap<String, usize> = HashMap::new();
    let mut source_documents: Vec<SourceAethelDoc<T>> = Vec::with_capacity(documents.len());

    for (index, document) in documents.into_iter().enumerate() {
        if document.header.target != T::TARGET {
            return Err(LoaderError::TargetMismatch {
                expected: T::TARGET.to_string(),
                found: document.header.target,
            }
            .into());
        }

        let source_path = format!("<aetheldoc:{index}>");
        let raw = toml::to_string(&document).map_err(|err| {
            MergerError::InvalidInput(format!(
                "unable to serialise aethel document for source hashing at '{source_path}': {err}"
            ))
        })?;
        let source_hash = hash_source_content(document.header.target.as_str(), raw.as_str());
        let source_id = make_unique_source_id(&source_hash, &mut seen_source_ids);

        source_documents.push(SourceAethelDoc {
            source_id,
            source_hash,
            source_path,
            header: document.header,
            data: document.data,
        });
    }

    Ok(source_documents)
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
    use crate::loader::TARGET_WEAPON;
    use crate::merger::MergeSourceInput;
    use crate::test_support::{TempTomlFile, person_document, weapon_document};
    use serde::Deserialize;

    #[derive(Deserialize, Debug, Clone)]
    struct TestWeaponLoader {
        name: Option<TestNameSection>,
    }

    #[derive(Deserialize, Debug, Clone)]
    struct TestNameSection {
        suffix: Option<Vec<String>>,
    }

    impl TargetedLoader for TestWeaponLoader {
        const TARGET: &'static str = TARGET_WEAPON;
    }

    #[test]
    fn test_parse_merge_inputs_collects_paths_and_targets() {
        let weapon = TempTomlFile::new(&weapon_document("weapon set", ""));
        let person = TempTomlFile::new(&person_document("person set", ""));

        let parsed = parse_merge_inputs(&[weapon.path_str(), person.path_str()]).unwrap();

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].target, "weapon");
        assert_eq!(parsed[1].target, "person");
    }

    #[test]
    fn test_build_raw_corpus_from_sources_rejects_mixed_targets() {
        let source_a = weapon_document("weapon set", "");
        let source_b = person_document("person set", "");
        let sources = [
            MergeSourceInput {
                path: "source_a.toml",
                raw: source_a.as_str(),
            },
            MergeSourceInput {
                path: "source_b.toml",
                raw: source_b.as_str(),
            },
        ];

        let error = build_raw_corpus_from_sources(&sources, None).unwrap_err();

        assert!(matches!(
            error,
            MergerError::Loader(LoaderError::TargetMismatch { expected, found })
            if expected == "weapon" && found == "person"
        ));
    }

    #[test]
    fn test_build_corpus_from_sources_validates_loader_target() {
        let person = person_document("person set", "");
        let sources = [MergeSourceInput {
            path: "person.toml",
            raw: person.as_str(),
        }];

        let error = build_corpus_from_sources::<TestWeaponLoader>(&sources, None).unwrap_err();

        assert!(matches!(
            error,
            MergerError::Loader(LoaderError::TargetMismatch { expected, found })
            if expected == "weapon" && found == "person"
        ));
    }

    #[test]
    fn test_build_corpus_from_paths_loads_typed_documents() {
        let weapon = TempTomlFile::new(&weapon_document(
            "weapon set",
            r#"
[name]
suffix = ["iron"]
"#,
        ));

        let corpus =
            build_corpus_from_paths::<TestWeaponLoader>(&[weapon.path_str()], None).unwrap();

        assert_eq!(corpus.target, "weapon");
        assert_eq!(corpus.documents.len(), 1);
        assert!(
            corpus.documents[0]
                .data
                .name
                .as_ref()
                .and_then(|name| name.suffix.as_ref())
                .is_some()
        );
    }

    #[test]
    fn test_build_corpus_from_sources_matches_build_corpus_from_paths() {
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

        let from_paths = build_corpus_from_paths::<TestWeaponLoader>(&paths, None).unwrap();

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
            build_corpus_from_sources::<TestWeaponLoader>(&source_inputs, None).unwrap();

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
            assert_eq!(
                left.data
                    .name
                    .as_ref()
                    .and_then(|name| name.suffix.as_ref()),
                right
                    .data
                    .name
                    .as_ref()
                    .and_then(|name| name.suffix.as_ref())
            );
        }
    }
}
