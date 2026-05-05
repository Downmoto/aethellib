//! shows deterministic generation by using the same rng seed.
//! this demonstrates generator::generate_with_rng for reproducible output.

#[path = "common/mod.rs"]
mod support;

use aethellib::generator::{Generator, generator_weapon::WeaponGenerator};
use rand::{SeedableRng, rngs::StdRng};
use std::error::Error;
use support::{TempTomlFile, weapon_document};

fn main() -> Result<(), Box<dyn Error>> {
    // keep the fixture intentionally small so determinism checks focus on rng
    // seeding rather than complex content variance.
    let fixture = TempTomlFile::new(&weapon_document(
        "seeded weapon demo",
        r#"
[name]
prefix = ["stone", "moon"]
suffix = ["of embers", "of tides"]
primitives = ["ra", "th", "iel", "kor"]

[type]
type = ["axe", "blade"]
"#,
    ));

    let generator = WeaponGenerator::from_file(fixture.path_str())?;

    // same seed => same random stream => same sampled candidates.
    let mut rng_a = StdRng::seed_from_u64(42);
    let mut rng_b = StdRng::seed_from_u64(42);

    let a = generator.generate_with_rng(&mut rng_a);
    let b = generator.generate_with_rng(&mut rng_b);

    // assert both a simple field and an optional field to show that deterministic
    // behaviour applies across the whole generated payload.
    assert_eq!(a.name.value, b.name.value);
    assert_eq!(
        a.weapon_type.as_ref().map(|field| &field.value),
        b.weapon_type.as_ref().map(|field| &field.value)
    );

    println!("deterministic name: {}", a.name.value);
    Ok(())
}
