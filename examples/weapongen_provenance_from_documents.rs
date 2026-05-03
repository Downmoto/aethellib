//! purpose: show `from_documents` generation plus printing provenance source refs for generated fields.

use aethellib::generators::SourceRef;
use aethellib::generators::generator_weapon::{GeneratedWeapon, WeaponGenerator};
use aethellib::loader::loader_weapon::{
    WeaponLoader, WeaponNameSection, WeaponQualitiesSection, WeaponTypeSection,
};
use aethellib::loader::{AthelDocHeader, Target};
use aethellib::merge::SourceAethelDoc;

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

fn make_source_doc(source_id: &str, source_name: &str, data: WeaponLoader) -> SourceAethelDoc<WeaponLoader> {
    SourceAethelDoc {
        source_id: source_id.to_string(),
        source_hash: format!("hash-{source_id}"),
        source_path: format!("{source_id}.toml"),
        header: AthelDocHeader {
            name: source_name.to_string(),
            target: Target::Weapon,
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
                weapon_type: Some(WeaponTypeSection {
                    types: Some(vec!["rapier".to_string()]),
                }),
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
    print_weapon_provenance("from_documents", &generated);
}
