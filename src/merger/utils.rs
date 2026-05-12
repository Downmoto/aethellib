//! shared merge utilities for parsing, corpus assembly, and source identity.

use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

use sha2::{Digest, Sha256};

use crate::{
    AethelCorpus, AethelDoc, SourceAethelDoc,
    loader::{TargetedLoader, error::LoaderError},
    merger::{
        MergeSourceInput, MergeValidator,
        error::MergerError,
        merger_options::{MergeOptions, MergerOptionError},
    },
};

struct NoopMergeValidator;

impl<T> MergeValidator<T> for NoopMergeValidator
where
    T: TargetedLoader,
{
    fn validate(&self, _document: &AethelDoc<T>, _source_path: &str) -> Result<(), MergerError> {
        Ok(())
    }
}

/// loads and validates source files for one target, then assembles a corpus.
pub fn build_corpus_from_paths<T>(
    paths: &[impl AsRef<Path>],
    opts: Option<MergeOptions>,
) -> Result<AethelCorpus<T>, MergerError>
where
    T: TargetedLoader,
{
    build_corpus_from_paths_with_validator::<T>(paths, opts, &NoopMergeValidator)
}

/// loads and validates source files for one target, then assembles a corpus using a validator.
pub fn build_corpus_from_paths_with_validator<T>(
    paths: &[impl AsRef<Path>],
    opts: Option<MergeOptions>,
    validator: &impl MergeValidator<T>,
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
        let path_ref = path.as_ref();
        let raw = fs::read_to_string(path_ref)
            .map_err(|source| LoaderError::read_for_path(path_ref, source))?;
        sources.push((path_ref.to_string_lossy().to_string(), raw));
    }

    let source_refs: Vec<MergeSourceInput<'_>> = sources
        .iter()
        .map(|(path, raw)| MergeSourceInput {
            path: path.as_str(),
            raw: raw.as_str(),
        })
        .collect();

    build_corpus_from_sources_with_validator::<T>(&source_refs, opts, validator)
}

/// assembles a target corpus from already-loaded source payloads.
pub fn build_corpus_from_sources<T>(
    sources: &[MergeSourceInput<'_>],
    opts: Option<MergeOptions>,
) -> Result<AethelCorpus<T>, MergerError>
where
    T: TargetedLoader,
{
    build_corpus_from_sources_with_validator::<T>(sources, opts, &NoopMergeValidator)
}

/// assembles a target corpus from already-loaded source payloads using a validator.
pub fn build_corpus_from_sources_with_validator<T>(
    sources: &[MergeSourceInput<'_>],
    opts: Option<MergeOptions>,
    validator: &impl MergeValidator<T>,
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
        let parsed = T::from_str(source.path, source.raw)?;

        if !options.identical_title_allowed && !seen_header_names.insert(parsed.header.title.clone())
        {
            return Err(MergerOptionError::IdenticalNameAllowed {
                header: parsed.header.title,
            }
            .into());
        }

        validator.validate(&parsed, source.path)?;

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
pub(crate) fn cast_aethel_docs_to_sources<T>(
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