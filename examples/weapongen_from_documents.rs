//! purpose: show how to create a weapon generator from in-memory `SourceAethelDoc` values.

use aethellib::generators::Generator;
use aethellib::generators::generator_weapon::WeaponGenerator;
use aethellib::loader::loader_weapon::{
    WeaponLoader, WeaponNameSection, WeaponQualitiesSection, WeaponTypeSection,
};
use aethellib::loader::{AthelDocHeader, TARGET_WEAPON};
use aethellib::merger::SourceAethelDoc;

fn make_source_doc(source_id: &str, source_name: &str, data: WeaponLoader) -> SourceAethelDoc<WeaponLoader> {
    SourceAethelDoc {
        source_id: source_id.to_string(),
        source_hash: format!("hash-{source_id}"),
        source_path: format!("{source_id}.toml"),
        header: AthelDocHeader {
            name: source_name.to_string(),
            target: TARGET_WEAPON.to_string(),
            desc: None,
            author: None,
            version: None,
        },
        data,
    }
}

fn main() {
    let documents = vec![
        make_source_doc(
            "doc-a",
            "manual source a",
            WeaponLoader {
                name: Some(WeaponNameSection {
                    prefix: Some(vec!["iron".to_string()]),
                    suffix: Some(vec!["of dawn".to_string()]),
                    primitives: Some(vec!["ka".to_string(), "li".to_string()]),
                }),
                weapon_type: Some(WeaponTypeSection {
                    types: Some(vec!["longsword".to_string()]),
                }),
                qualities: Some(WeaponQualitiesSection {
                    rarity: Some(vec!["rare".to_string()]),
                    condition: Some(vec!["pristine".to_string()]),
                }),
                lore: None,
                visuals: None,
            },
        ),
        make_source_doc(
            "doc-b",
            "manual source b",
            WeaponLoader {
                name: Some(WeaponNameSection {
                    prefix: Some(vec!["steel".to_string()]),
                    suffix: Some(vec!["of dusk".to_string()]),
                    primitives: Some(vec!["ra".to_string(), "na".to_string()]),
                }),

                // does not contribute to candidate pool, this will always be "longsword"
                weapon_type: None,                 

                qualities: Some(WeaponQualitiesSection {
                    rarity: Some(vec!["common".to_string()]),
                    condition: Some(vec!["worn".to_string()]),
                }),
                lore: None,
                visuals: None,
            },
        ),
    ];

    let generator = WeaponGenerator::from_documents(documents);
    let generated = generator.generate();

    println!("from_documents -> {}", generated.name.value);
}
