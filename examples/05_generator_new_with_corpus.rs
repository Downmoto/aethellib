//! constructs a corpus manually and creates a generator with generator::new.
//! this demonstrates aethelcorpus and sourceaetheldoc wiring.

use aethellib::generator::{Generator, generator_weapon::WeaponGenerator};
use aethellib::loader::{AthelDocHeader, TARGET_WEAPON, loader_weapon::{WeaponLoader, WeaponNameSection}};
use aethellib::merger::{AethelCorpus, SourceAethelDoc};

fn main() {
    let corpus = AethelCorpus {
        target: TARGET_WEAPON.to_string(),
        documents: vec![SourceAethelDoc {
            source_id: "manual-source".to_string(),
            source_hash: "manual-hash".to_string(),
            source_path: "manual.toml".to_string(),
            header: AthelDocHeader {
                name: "manual corpus".to_string(),
                target: TARGET_WEAPON.to_string(),
                desc: Some("built directly in code".to_string()),
                author: Some("example".to_string()),
                version: Some("1.0".to_string()),
            },
            data: WeaponLoader {
                name: Some(WeaponNameSection {
                    prefix: Some(vec!["gale".to_string()]),
                    suffix: Some(vec!["of the west".to_string()]),
                    primitives: Some(vec!["lor".to_string(), "an".to_string()]),
                }),
                weapon_type: None,
                qualities: None,
                lore: None,
                visuals: None,
            },
        }],
    };

    let generator = WeaponGenerator::new(corpus);
    let generated = generator.generate();

    println!("generated from explicit corpus: {}", generated.name.value);
}
