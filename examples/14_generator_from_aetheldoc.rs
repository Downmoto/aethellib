//! builds a generator from one parsed aethel document.
//! this demonstrates generator::from_aethel_doc.

use aethellib::generator::{Generator, generator_weapon::WeaponGenerator};
use aethellib::loader::{AethelDoc, loader_weapon::WeaponLoader};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let parsed: AethelDoc<WeaponLoader> = toml::from_str(
        r#"
[header]
name = "parsed weapon set"
target = "weapon"

[name]
prefix = ["iron"]
suffix = ["of dawn"]
primitives = ["ka", "lor"]
"#,
    )?;

    let generator = WeaponGenerator::from_aethel_doc(parsed)?;
    let generated = generator.generate();

    println!("generated from aetheldoc: {}", generated.name.value);
    Ok(())
}
