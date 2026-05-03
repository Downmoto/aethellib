//! purpose: show how to merge multiple files into a corpus and call `WeaponGenerator::new`.

use std::error::Error;

use aethellib::generators::generator_weapon::WeaponGenerator;
use aethellib::loader::loader_weapon::WeaponLoader;
use aethellib::merge::AethelCorpus;
use aethellib::merge::merge_weapon::merge_weapon_files;

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
    Ok(())
}
