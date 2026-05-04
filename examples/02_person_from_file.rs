//! generates one person name from a temporary toml file.
//! this demonstrates generator::from_file and generator::generate for person data.

#[path = "common/mod.rs"]
mod support;

use aethellib::generator::{Generator, generator_person::PersonGenerator};
use std::error::Error;
use support::{TempTomlFile, person_document};

fn main() -> Result<(), Box<dyn Error>> {
    let fixture = TempTomlFile::new(&person_document(
        "person demo",
        r#"
[name]
first = ["al", "be", "cor"]
middle = ["ri", "na"]
last = ["son", "ford", "ley"]
"#,
    ));

    let generator = PersonGenerator::from_file(fixture.path_str())?;
    let generated = generator.generate();

    println!("generated person name: {}", generated.name.value);
    Ok(())
}
