//! merges multiple weapon files into one typed corpus.
//! this demonstrates merge_target_files and corpus document inspection.

#[path = "common/mod.rs"]
mod support;

use aethellib::loader::{TARGET_WEAPON, loader_weapon::WeaponLoader};
use aethellib::merger::merge_target_files;
use std::error::Error;
use support::{TempTomlFile, weapon_document};

fn main() -> Result<(), Box<dyn Error>> {
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
    let corpus = merge_target_files::<WeaponLoader>(&paths, None)?;

    assert_eq!(corpus.target, TARGET_WEAPON);
    assert_eq!(corpus.documents.len(), 2);

    for document in &corpus.documents {
        println!(
            "source {} from {} ({})",
            document.source_id, document.source_path, document.header.name
        );
    }

    Ok(())
}
