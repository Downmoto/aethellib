//! purpose: show `from_file` generation plus printing provenance source refs for generated fields.

use std::error::Error;

use aethellib::generators::SourceRef;
use aethellib::generators::generator_weapon::{GeneratedWeapon, WeaponGenerator};

fn print_refs(refs: &[SourceRef]) {
    for source in refs {
        println!(
            "    - {} [{}] {}.{}",
            source.source_name, source.source_id, source.section, source.field
        );
    }
}

fn print_weapon_provenance(label: &str, generated: &GeneratedWeapon) {
    println!("{label} provenance");

    println!("  name refs:");
    print_refs(&generated.name.source_refs);

    if let Some(weapon_type) = &generated.weapon_type {
        println!("  weapon_type refs:");
        print_refs(&weapon_type.source_refs);
    }

    if let Some(rarity) = &generated.rarity {
        println!("  rarity refs:");
        print_refs(&rarity.source_refs);
    }

    if let Some(lore) = &generated.lore {
        println!("  lore refs:");
        print_refs(&lore.source_refs);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let generator = WeaponGenerator::from_file("data/weapon_test_data.toml")?;
    let generated = generator.generate();

    println!("from_file -> {}", generated.name.value);
    print_weapon_provenance("from_file", &generated);
    Ok(())
}
