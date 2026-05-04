//! purpose: show the simplest person generator flow using `PersonGenerator::from_file`.

use std::error::Error;

use aethellib::generators::Generator;
use aethellib::generators::generator_person::PersonGenerator;

fn main() -> Result<(), Box<dyn Error>> {
    let generator = PersonGenerator::from_file("data/person_test_data.toml")?;
    let generated = generator.generate();

    println!("from_file -> {}", generated.name.value);
    Ok(())
}
