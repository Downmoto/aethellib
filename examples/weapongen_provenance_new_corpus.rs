//! purpose: show `new(corpus)` generation plus printing provenance source refs for generated fields.

use std::error::Error;

use aethellib::generators::SourceRef;
use aethellib::generators::generator_weapon::{GeneratedWeapon, WeaponGenerator};
use aethellib::loader::loader_weapon::WeaponLoader;
use aethellib::merge::AethelCorpus;
use aethellib::merge::merge_weapon::merge_weapon_files;

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

fn build_weapon_corpus() -> Result<AethelCorpus<WeaponLoader>, Box<dyn Error>> {
    let paths = [
        "data/weapon_merge_part_1.toml",
        "data/weapon_merge_part_2.toml",
        "data/weapon_merge_part_3.toml",
        "data/weapon_merge_part_4.toml",
    ];

    let corpus = merge_weapon_files(&paths)?;
    Ok(corpus)
}

fn main() -> Result<(), Box<dyn Error>> {
    let corpus = build_weapon_corpus()?;
    let generator = WeaponGenerator::new(corpus);
    let generated = generator.generate();

    println!("new(corpus) -> {}", generated.name.value);
    print_weapon_provenance("new(corpus)", &generated);
    Ok(())
}
