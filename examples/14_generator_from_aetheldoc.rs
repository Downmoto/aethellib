//! builds a generator from one parsed aethel document.
//! this demonstrates generator::from_aethel_doc.

#[path = "common/mod.rs"]
mod support;

use support::{TempTomlFile, weapon_document};
use aethellib::loader::{TargetedLoader, loader_weapon::WeaponLoader};
use aethellib::generator::{Generator, generator_weapon::WeaponGenerator};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let fixture = TempTomlFile::new(&weapon_document(
        "loader demo",
        r#"
[name]
prefix = ["iron"]
"#,
    ));

    let loaded = WeaponLoader::from_file(fixture.path_str())?;
    let generator = WeaponGenerator::from_aethel_doc(loaded)?;
    let generated = generator.generate();

    println!("generated from aetheldoc: {}", generated.name.value);
    Ok(())
}
