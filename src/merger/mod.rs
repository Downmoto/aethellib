//! central target-based merge orchestration for aethel documents.

pub mod error;
pub mod merger_options;
pub(crate) mod utils;

use std::path::Path;

use crate::loader::TargetedLoader;
use crate::merger::error::MergerError;
use crate::merger::utils::{
    build_corpus_from_paths, build_corpus_from_paths_with_validator, build_corpus_from_sources,
    build_corpus_from_sources_with_validator,
};
use crate::{AethelCorpus, AethelDoc};
use merger_options::MergeOptions;

/// optional merge validation hook for user-defined policy checks.
pub trait MergeValidator<T>
where
    T: TargetedLoader,
{
    /// validates one parsed document before it is added to the merged corpus.
    fn validate(&self, document: &AethelDoc<T>, source_path: &str) -> Result<(), MergerError>;
}

/// source payload used by target-specific corpus builders.
pub struct MergeSourceInput<'a> {
    /// original source path, or name used for loading.
    pub path: &'a str,
    /// raw source content used for parsing and hashing.
    pub raw: &'a str,
}

/// merges one target file set into one typed corpus.
pub fn merge_files<T>(
    paths: &[impl AsRef<Path>],
    opts: Option<MergeOptions>,
) -> Result<AethelCorpus<T>, MergerError>
where
    T: TargetedLoader,
{
    build_corpus_from_paths::<T>(paths, opts)
}

/// merges one target file set using a custom validation hook.
pub fn merge_files_with_validator<T>(
    paths: &[impl AsRef<Path>],
    opts: Option<MergeOptions>,
    validator: &impl MergeValidator<T>,
) -> Result<AethelCorpus<T>, MergerError>
where
    T: TargetedLoader,
{
    build_corpus_from_paths_with_validator::<T>(paths, opts, validator)
}

/// merges in-memory sources into one typed corpus.
pub fn merge_sources<T>(
    sources: &[MergeSourceInput<'_>],
    opts: Option<MergeOptions>,
) -> Result<AethelCorpus<T>, MergerError>
where
    T: TargetedLoader,
{
    build_corpus_from_sources::<T>(sources, opts)
}

/// merges in-memory sources using a custom validation hook.
pub fn merge_sources_with_validator<T>(
    sources: &[MergeSourceInput<'_>],
    opts: Option<MergeOptions>,
    validator: &impl MergeValidator<T>,
) -> Result<AethelCorpus<T>, MergerError>
where
    T: TargetedLoader,
{
    build_corpus_from_sources_with_validator::<T>(sources, opts, validator)
}