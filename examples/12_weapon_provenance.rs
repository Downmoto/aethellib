//! inspects generated field provenance for traceability.
//! this demonstrates generatedfield and sourceref values from weapon output.

#[path = "common/mod.rs"]
mod support;

use aethellib::generator::{Generator, SourceRef, generator_weapon::WeaponGenerator};
use aethellib::loader::loader_weapon::WeaponLoader;
use aethellib::merger::merge_target_files;
use rand::{SeedableRng, rngs::StdRng};
use std::error::Error;
use support::{TempTomlFile, weapon_document};

fn main() -> Result<(), Box<dyn Error>> {
    // split fixtures by concern so provenance output clearly shows which source
    // contributed which generated field candidates.
    let names = TempTomlFile::new(&weapon_document(
        "name source",
        r#"
[name]
prefix = ["iron"]
suffix = ["of dawn"]
primitives = ["ka", "lor"]
"#,
    ));

    let lore = TempTomlFile::new(&weapon_document(
        "lore source",
        r#"
[lore]
creators = ["the first forge"]
deeds = ["held the bridge"]
quirks = ["sings in rain"]
templates = ["forged by {creator}, it {deed} and now {quirk}."]
"#,
    ));

    // merge preserves source metadata that later becomes SourceRef entries.
    let corpus = merge_target_files::<WeaponLoader>(&[names.path_str(), lore.path_str()], None)?;
    let generator = WeaponGenerator::new(corpus);

    // seeded rng makes this example output stable for easier learning/debugging.
    let mut rng = StdRng::seed_from_u64(7);
    let generated = generator.generate_with_rng(&mut rng);

    println!("generated name: {}", generated.name.value);
    print_refs("name", &generated.name.source_refs);

    if let Some(lore_field) = &generated.lore {
        println!("generated lore: {}", lore_field.value);
        print_refs("lore", &lore_field.source_refs);
    }

    Ok(())
}

fn print_refs(label: &str, refs: &[SourceRef]) {
    // each SourceRef points to section + field + source identity metadata,
    // allowing traceability from final output back to input datasets.
    println!("{label} provenance count: {}", refs.len());
    for source in refs {
        println!(
            "  {}:{} from {} ({})",
            source.section, source.field, source.source_name, source.source_id
        );
    }
}
