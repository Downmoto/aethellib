//! purpose: show direct `merge_weapon_files` and generic `merge_from_files` flows before `WeaponGenerator::new`.

use std::error::Error;

use aethellib::generator::Generator;
use aethellib::generator::generator_weapon::WeaponGenerator;
use aethellib::loader::loader_weapon::WeaponLoader;
use aethellib::merger::{AethelCorpus, merge_from_files, merge_target_files};

fn build_weapon_corpus() -> Result<AethelCorpus<WeaponLoader>, Box<dyn Error>> {
    let paths = [
        "data/weapon_merge_part_1.toml",
        "data/weapon_merge_part_2.toml",
        "data/weapon_merge_part_3.toml",
        "data/weapon_merge_part_4.toml",
    ];

    let corpus = merge_target_files::<WeaponLoader>(&paths, None)?;
    Ok(corpus)
}

fn build_weapon_corpus_via_merge_from_files() -> Result<AethelCorpus<WeaponLoader>, Box<dyn Error>> {
    let paths = [
        "data/weapon_merge_part_1.toml",
        "data/weapon_merge_part_2.toml",
        "data/weapon_merge_part_3.toml",
        "data/weapon_merge_part_4.toml",
    ];

    let merged_docs = merge_from_files(&paths, None)?;
    let Some(first) = merged_docs.into_iter().next() else {
        return Err("expected at least one merged document".into());
    };

    let corpus = first.into_corpus::<WeaponLoader>()?;
    Ok(corpus)
}

fn main() -> Result<(), Box<dyn Error>> {
    let direct_corpus = build_weapon_corpus()?;
    let direct_generator = WeaponGenerator::new(direct_corpus);
    let direct_generated = direct_generator.generate();

    println!("new(corpus) via merge_weapon_files -> {}", direct_generated.name.value);

    let dispatched_corpus = build_weapon_corpus_via_merge_from_files()?;
    let dispatched_generator = WeaponGenerator::new(dispatched_corpus);
    let dispatched_generated = dispatched_generator.generate();

    println!(
        "new(corpus) via merge_from_files -> {}",
        dispatched_generated.name.value
    );
    Ok(())
}
