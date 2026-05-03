use std::error::Error;

use aethellib::generators::generator_weapon::WeaponGenerator;
use aethellib::loader::loader_weapon::{
    WeaponLoader, WeaponNameSection, WeaponQualitiesSection, WeaponTypeSection,
};
use aethellib::loader::{AthelDocHeader, Target};
use aethellib::merge::merge_weapon::merge_weapon_files;
use aethellib::merge::{AethelCorpus, SourceAethelDoc};

/// demonstrates creating a generator by loading one source file directly.
fn example_from_file() -> Result<(), Box<dyn Error>> {
    let generator = WeaponGenerator::from_file("data/weapon_test_data.toml")?;
    let generated = generator.generate();

    println!("from_file -> {}", generated.name.value);
    print_weapon_provenance("from_file", &generated);
    Ok(())
}

/// demonstrates creating a generator from explicit source documents.
fn example_from_documents() -> Result<(), Box<dyn Error>> {
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
    Ok(())
}

/// demonstrates creating a generator from a pre-built corpus.
fn example_new_with_corpus() -> Result<(), Box<dyn Error>> {
    let corpus = build_weapon_corpus()?;
    let generator = WeaponGenerator::new(corpus);
    let generated = generator.generate();

    println!("new(corpus) -> {}", generated.name.value);
    print_weapon_provenance("new(corpus)", &generated);
    Ok(())
}

/// builds a corpus by merging multiple weapon source files.
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

/// creates a minimal source document for manual examples.
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

/// prints source provenance for key generated weapon fields.
fn print_weapon_provenance(label: &str, generated: &aethellib::generators::generator_weapon::GeneratedWeapon) {
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

/// pretty-prints a list of source refs.
fn print_refs(refs: &[aethellib::generators::SourceRef]) {
    for source in refs {
        println!(
            "    - {} [{}] {}.{}",
            source.source_name, source.source_id, source.section, source.field
        );
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    example_from_file()?;
    example_from_documents()?;
    example_new_with_corpus()?;

    Ok(())
}
