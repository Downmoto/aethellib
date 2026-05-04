//! loads one typed document and inspects common header fields.
//! this demonstrates targetedloader::from_file, aetheldoc, and target constants.

#[path = "common/mod.rs"]
mod support;

use aethellib::loader::{TARGET_WEAPON, Target, TargetedLoader, loader_weapon::WeaponLoader};
use std::error::Error;
use support::{TempTomlFile, weapon_document};

fn main() -> Result<(), Box<dyn Error>> {
    let fixture = TempTomlFile::new(&weapon_document(
        "loader demo",
        r#"
[name]
prefix = ["iron"]
"#,
    ));

    let loaded = WeaponLoader::from_file(fixture.path_str())?;
    let target: Target = loaded.header.target.clone();

    assert_eq!(target, TARGET_WEAPON);
    println!("header name: {}", loaded.header.name);
    println!("header target: {}", target);
    println!("has name section: {}", loaded.data.name.is_some());

    Ok(())
}
