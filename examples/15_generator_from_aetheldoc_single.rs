//! builds a generator from one parsed aethel document via tryfrom casting.
//! this demonstrates TryFrom<AethelDoc<T>> plus generator::from_documents.

#[path = "common/mod.rs"]
mod support;

use support::{TempTomlFile, weapon_document};
use aethellib::loader::{TargetedLoader, loader_weapon::WeaponLoader};
use aethellib::generator::{Generator, generator_weapon::WeaponGenerator};
use aethellib::merger::SourceAethelDoc;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // load + validate one parsed doc first. this pattern is useful when your
    // application parses documents in a different layer than generation.
    let fixture = TempTomlFile::new(&weapon_document(
        "single cast demo",
        r#"
[name]
prefix = ["iron"]
suffix = ["of dawn"]
primitives = ["ka", "lor"]
"#,
    ));

    // cast a single AethelDoc into SourceAethelDoc via TryFrom so source id
    // and source hash follow the same merge casting rules.
    let loaded = WeaponLoader::from_file(fixture.path_str())?;
    let source_document: SourceAethelDoc<WeaponLoader> = loaded.try_into()?;

    let generator = WeaponGenerator::from_documents(vec![source_document]);
    let generated = generator.generate();

    println!("generated from single aetheldoc cast: {}", generated.name.value);
    Ok(())
}
