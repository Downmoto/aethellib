use std::{collections::{HashMap, HashSet}, fs};

use sha2::{Digest, Sha256};

use crate::{loader::{AethelDoc, Target, TargetedLoader, error::LoaderError}, merger::{AethelCorpus, MergeSourceInput, ParsedMergeInput, SourceAethelDoc, error::MergerError, merger_options::{MergeOptions, MergerOptionError}}};



/// parses source files once so dispatch and target ingestion share the same payload.
pub fn parse_merge_inputs(paths: &[&str]) -> Result<Vec<ParsedMergeInput>, MergerError> {
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

/// assembles an untyped target corpus from already-loaded source payloads.
pub fn build_raw_corpus_from_sources(
    sources: &[MergeSourceInput<'_>],
    opts: Option<MergeOptions>,
) -> Result<AethelCorpus<toml::Table>, MergerError> {
    if sources.is_empty() {
        return Err(MergerError::InvalidInput(
            "at least one path is required for merge".to_string(),
        ));
    }

    let options = opts.unwrap_or_default();

    let mut seen_source_ids: HashMap<String, usize> = HashMap::new();
    let mut seen_header_names: HashSet<String> = HashSet::new();
    let mut documents: Vec<SourceAethelDoc<toml::Table>> = Vec::with_capacity(sources.len());
    let mut target: Option<Target> = None;

    for source in sources {
        let parsed: AethelDoc<toml::Table> = toml::from_str(source.raw)
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