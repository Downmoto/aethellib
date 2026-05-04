//! builds a generator from parsed aethel documents via batch tryfrom casting.
//! this demonstrates TryFrom<Vec<AethelDoc<T>>> plus generator::from_documents.

#[path = "common/mod.rs"]
mod support;

use support::{TempTomlFile, weapon_document};
use aethellib::loader::{TargetedLoader, loader_weapon::WeaponLoader};
use aethellib::generator::{Generator, generator_weapon::WeaponGenerator};
use aethellib::merger::SourceAethelDocs;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // load + validate two parsed docs first. in real applications this often
    // happens in a separate layer before generator construction.
    let fixture_a = TempTomlFile::new(&weapon_document(
        "loader demo a",
        r#"
[name]
prefix = ["iron"]
"#,
    ));
    let fixture_b = TempTomlFile::new(&weapon_document(
        "loader demo b",
        r#"
[name]
suffix = ["of dawn"]
primitives = ["ka", "lor"]
"#,
    ));

    // batch cast parsed docs into SourceAethelDoc values via TryFrom so
    // hashing/id sequencing aligns with merge casting rules.
    let loaded_a = WeaponLoader::from_file(fixture_a.path_str())?;
    let loaded_b = WeaponLoader::from_file(fixture_b.path_str())?;
    let source_documents = SourceAethelDocs::try_from(vec![loaded_a, loaded_b])?.into_vec();

    let generator = WeaponGenerator::from_documents(source_documents);
    let generated = generator.generate();

    println!("generated from aetheldoc: {}", generated.name.value);
    Ok(())
}
