//! builds a weapon generator directly from source documents.
//! this demonstrates generator::from_documents and sourceaetheldoc usage.

use aethellib::generator::{Generator, generator_weapon::WeaponGenerator};
use aethellib::loader::{
    AthelDocHeader, TARGET_WEAPON,
    loader_weapon::{WeaponLoader, WeaponNameSection, WeaponTypeSection},
};
use aethellib::merger::SourceAethelDoc;

fn main() {
    // from_documents skips file i/o entirely. this is useful when your caller
    // already has validated source documents in memory.
    let documents = vec![
        // source ids/hashes/paths are caller-defined in this entrypoint, so
        // you can align them with your own provenance system.
        SourceAethelDoc {
            source_id: "source-a".to_string(),
            source_hash: "hash-a".to_string(),
            source_path: "inline-a.toml".to_string(),
            header: AthelDocHeader {
                name: "inline set a".to_string(),
                target: TARGET_WEAPON.to_string(),
                desc: None,
                author: None,
                version: Some("1.0".to_string()),
            },
            data: WeaponLoader {
                name: Some(WeaponNameSection {
                    prefix: Some(vec!["sun".to_string()]),
                    suffix: Some(vec!["of frost".to_string()]),
                    primitives: Some(vec!["ka".to_string(), "rin".to_string()]),
                }),
                weapon_type: Some(WeaponTypeSection {
                    types: Some(vec!["sword".to_string()]),
                }),
                qualities: None,
                lore: None,
                visuals: None,
            },
        },
        SourceAethelDoc {
            source_id: "source-b".to_string(),
            source_hash: "hash-b".to_string(),
            source_path: "inline-b.toml".to_string(),
            header: AthelDocHeader {
                name: "inline set b".to_string(),
                target: TARGET_WEAPON.to_string(),
                desc: None,
                author: None,
                version: Some("1.1".to_string()),
            },
            data: WeaponLoader {
                name: Some(WeaponNameSection {
                    prefix: Some(vec!["ember".to_string()]),
                    suffix: None,
                    primitives: Some(vec!["dor".to_string(), "el".to_string()]),
                }),
                weapon_type: Some(WeaponTypeSection {
                    types: Some(vec!["spear".to_string()]),
                }),
                qualities: None,
                lore: None,
                visuals: None,
            },
        },
    ];

    let generator = WeaponGenerator::from_documents(documents);

    // generation logic is unchanged regardless of how the corpus was built.
    let generated = generator.generate();

    println!("generated from documents: {}", generated.name.value);
}
