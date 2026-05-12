//! central target-based merge orchestration for aethel documents.

pub mod error;
pub mod merger_options;
pub(crate) mod utils;

use crate::loader::TargetedLoader;
use crate::merger::error::MergerError;
use crate::merger::utils::build_corpus_from_paths;
use crate::AethelCorpus;
use merger_options::MergeOptions;

/// source payload used by target-specific corpus builders.
pub(crate) struct MergeSourceInput<'a> {
    /// original source path used for loading.
    pub path: &'a str,
    /// raw source content used for parsing and hashing.
    pub raw: &'a str,
}

/// merges one target file set into one typed corpus.
pub fn merge_files<T>(
    paths: &[&str],
    opts: Option<MergeOptions>,
) -> Result<AethelCorpus<T>, MergerError>
where
    T: TargetedLoader,
{
    build_corpus_from_paths::<T>(paths, opts)
}

#[deprecated(
    since = "0.2.0",
    note = "use merge_files::<T>(...) instead; aethellib now exposes single-target merge only"
)]
/// compatibility alias for the previous merge api.
pub fn merge_target_files<T>(
    paths: &[&str],
    opts: Option<MergeOptions>,
) -> Result<AethelCorpus<T>, MergerError>
where
    T: TargetedLoader,
{
    merge_files::<T>(paths, opts)
}

