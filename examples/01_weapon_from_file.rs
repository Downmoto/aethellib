//! generates one weapon from a temporary toml file.
//! this demonstrates generator::from_file and generator::generate.

#[path = "common/mod.rs"]
mod support;

use aethellib::generator::{Generator, generator_weapon::WeaponGenerator};
use std::error::Error;
use support::{TempTomlFile, weapon_document};

fn main() -> Result<(), Box<dyn Error>> {
    // this inline toml defines a complete weapon dataset in one place so readers
    // can see exactly which sections feed each generated field.
    let fixture = TempTomlFile::new(&weapon_document(
        "weapon demo",
        r#"
[name]
prefix = ["iron", "ember"]
suffix = ["of dawn", "of ash"]
primitives = ["ka", "el", "dor", "ven"]

[type]
type = ["sword", "spear"]

[qualities]
rarity = ["common", "rare"]
condition = ["worn", "pristine"]

[lore]
creators = ["the old smith", "a nameless guild"]
deeds = ["broke a siege", "guarded the pass"]
quirks = ["hums at dusk", "glows near storms"]
templates = ["forged by {creator}, it once {deed} and now {quirk}."]

[visuals]
materials = ["steel", "obsidian"]
colours = ["silver", "black"]
accents = ["etched runes", "braided leather"]
features = ["a split fuller", "a crescent pommel"]
templates = ["a {colour} {material} blade with {accent} and {feature}."]
"#,
    ));

    // from_file performs load + parse + target validation, then builds the
    // generator from a one-document corpus under the hood.
    let generator = WeaponGenerator::from_file(fixture.path_str())?;

    // generate uses thread-local randomness, which is ideal for "just run it"
    // behaviour in quick examples.
    let generated = generator.generate();

    // generated weapon implements display, so this prints all major fields
    // (name/type/qualities/lore/visuals) in a readable multi-line block.
    println!("{}", generated);
    Ok(())
}
