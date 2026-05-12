//! central target-based merge orchestration for aethel documents.

pub mod error;
pub mod merger_options;
pub(crate) mod utils;

use std::collections::HashMap;

use crate::loader::TargetedLoader;
use crate::merger::error::MergerError;
use crate::merger::utils::{
    build_corpus_from_paths, build_raw_corpus_from_sources, parse_merge_inputs,
};
use crate::{AethelCorpus, Target};
use merger_options::MergeOptions;

/// parsed source used by merge dispatch before target-specific ingestion.
pub(crate) struct ParsedMergeInput {
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

/// untyped merged document body used for mixed-target merge flows.
pub type Mixed = toml::Table;

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
