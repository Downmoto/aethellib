use crate::loader::loader_weapon::WeaponLoader;
use crate::merge::{build_corpus_from_paths, AethelCorpus, MergeError};

/// assembles a weapon corpus while preserving file-specific metadata.
pub fn merge_weapon_files(paths: &[&str]) -> Result<AethelCorpus<WeaponLoader>, MergeError> {
    build_corpus_from_paths::<WeaponLoader>(paths)
}