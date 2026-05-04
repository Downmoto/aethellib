//! merges multiple weapon files into one typed corpus.
//! this demonstrates merge_target_files and corpus document inspection.

#[path = "common/mod.rs"]
mod support;

use aethellib::loader::{TARGET_WEAPON, loader_weapon::WeaponLoader};
use aethellib::merger::merge_target_files;
use std::error::Error;
use support::{TempTomlFile, weapon_document};

fn main() -> Result<(), Box<dyn Error>> {
    // each file contributes partial sections; merging creates one typed corpus
    // that keeps per-source metadata for provenance and auditing.
    let one = TempTomlFile::new(&weapon_document(
        "merge set a",
        r#"
[name]
prefix = ["iron"]
"#,
    ));
    let two = TempTomlFile::new(&weapon_document(
        "merge set b",
        r#"
[name]
suffix = ["of dawn"]
"#,
    ));

    let paths = [one.path_str(), two.path_str()];

    // merge_target_files enforces that all inputs match WeaponLoader::TARGET.
    let corpus = merge_target_files::<WeaponLoader>(&paths, None)?;

    // corpus.target is the canonical target string for the merged payload.
    assert_eq!(corpus.target, TARGET_WEAPON);
    assert_eq!(corpus.documents.len(), 2);

    // source metadata remains attached per document, even after merge.
    for document in &corpus.documents {
        println!(
            "source {} from {} ({})",
            document.source_id, document.source_path, document.header.name
        );
    }

    Ok(())
}
