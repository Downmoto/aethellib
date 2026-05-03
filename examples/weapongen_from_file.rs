//! purpose: show the simplest weapon generator flow using `WeaponGenerator::from_file`.

use std::error::Error;

use aethellib::generators::generator_weapon::WeaponGenerator;

fn main() -> Result<(), Box<dyn Error>> {
    let generator = WeaponGenerator::from_file("data/weapon_test_data.toml")?;
    let generated = generator.generate();

    println!("from_file -> {}", generated.name.value);
    Ok(())
}
