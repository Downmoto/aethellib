//! loads one typed document and inspects common header fields.
//! this demonstrates targetedloader::from_file, aetheldoc, and target constants.

#[path = "common/mod.rs"]
mod support;

use aethellib::{
    Target,
    loader::{TARGET_WEAPON, TargetedLoader, loader_weapon::WeaponLoader},
};
use std::error::Error;
use support::{TempTomlFile, weapon_document};

fn main() -> Result<(), Box<dyn Error>> {
    // this fixture only defines one section so it is obvious what gets parsed
    // into the typed loader payload.
    let fixture = TempTomlFile::new(&weapon_document(
        "loader demo",
        r#"
[name]
prefix = ["iron"]
"#,
    ));

    // from_file returns AethelDoc<WeaponLoader>, which includes both common
    // header metadata and target-specific body data.
    let loaded = WeaponLoader::from_file(fixture.path_str())?;

    // header.target is a string-backed Target alias, so clone it if you need
    // to move it around independently of the loaded struct.
    let target: Target = loaded.header.target.clone();

    // verify target consistency and inspect parsed content.
    assert_eq!(target, TARGET_WEAPON);
    println!("header name: {}", loaded.header.name);
    println!("header target: {}", target);
    println!("has name section: {}", loaded.data.name.is_some());

    Ok(())
}
