//! weapon-target merge entrypoints.

use crate::loader::loader_weapon::WeaponLoader;
use crate::merge::{AethelCorpus, MergeError, MergeSourceInput, build_corpus_from_paths, build_corpus_from_sources};

/// assembles a weapon corpus while preserving file-specific metadata.
pub fn merge_weapon_files(paths: &[&str]) -> Result<AethelCorpus<WeaponLoader>, MergeError> {
    build_corpus_from_paths::<WeaponLoader>(paths)
}

/// assembles a weapon corpus from preloaded source payloads.
pub(crate) fn merge_weapon_sources(
    sources: &[MergeSourceInput<'_>],
) -> Result<AethelCorpus<WeaponLoader>, MergeError> {
    build_corpus_from_sources::<WeaponLoader>(sources)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_weapon_sources_matches_merge_weapon_files() {
        let paths = [
            "data/weapon_merge_part_1.toml",
            "data/weapon_merge_part_2.toml",
            "data/weapon_merge_part_3.toml",
            "data/weapon_merge_part_4.toml",
        ];

        let from_files = merge_weapon_files(&paths).unwrap();

        let loaded_sources: Vec<(String, String)> = paths
            .iter()
            .map(|path| {
                (
                    (*path).to_string(),
                    std::fs::read_to_string(path).unwrap(),
                )
            })
            .collect();

        let source_inputs: Vec<MergeSourceInput<'_>> = loaded_sources
            .iter()
            .map(|(path, raw)| MergeSourceInput {
                path: path.as_str(),
                raw: raw.as_str(),
            })
            .collect();

        let from_sources = merge_weapon_sources(&source_inputs).unwrap();

        assert_eq!(from_files.target, from_sources.target);
        assert_eq!(from_files.documents.len(), from_sources.documents.len());

        for (left, right) in from_files.documents.iter().zip(from_sources.documents.iter()) {
            assert_eq!(left.source_id, right.source_id);
            assert_eq!(left.source_hash, right.source_hash);
            assert_eq!(left.source_path, right.source_path);
            assert_eq!(left.header.name, right.header.name);
            assert_eq!(left.header.target, right.header.target);
            assert_eq!(left.header.version, right.header.version);
        }
    }
}