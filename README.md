# aethellib

aethellib is a rust library for loading, merging, and generating aethel target datasets from toml files.

it currently supports:
- weapon target generation
- person target generation (minimal primitive-name model)

## features

- typed target loaders with header validation
- corpus-based merge pipeline that preserves source metadata per input file
- deterministic source identity (`source_hash`, `source_id`, `source_path`)
- provenance-aware generated fields via `GeneratedField<T>` and `SourceRef`
- deterministic generation support through `generate_with_rng(...)`
- target-extensible architecture for loaders, merge variants, and generators

## installation

add this crate to your `Cargo.toml`:

```toml
[dependencies]
aethellib = { git = "https://github.com/Downmoto/aethellib", branch = "master" }
```

## quick start

### from a single weapon file

```rust
use aethellib::generators::Generator;
use aethellib::generators::generator_weapon::WeaponGenerator;

let generator = WeaponGenerator::from_file("data/weapon_test_data.toml")?;
let generated = generator.generate();

println!("weapon: {}", generated.name.value);
```

### merge mixed targets from multiple files

```rust
use aethellib::merge::{MergedAethelDoc, merge_from_files};
use aethellib::generators::Generator;
use aethellib::generators::generator_weapon::WeaponGenerator;
use aethellib::generators::generator_person::PersonGenerator;

let merged = merge_from_files(&[
	"data/person_test_data.toml",
	"data/weapon_test_data.toml",
], None)?;

for doc in merged {
	match doc {
		MergedAethelDoc::Person(corpus) => {
			println!("person corpus docs: {}", corpus.documents.len());
			let generator = PersonGenerator::new(corpus);
			let generated = generator.generate();
			// ...
		}
		MergedAethelDoc::Weapon(corpus) => {
			println!("weapon corpus docs: {}", corpus.documents.len());
			let generator = WeaponGenerator::new(corpus);
			let generated = generator.generate();
			// ...
		}
	}
}
```

### deterministic generation

```rust
use rand::SeedableRng;
use rand::rngs::StdRng;

use aethellib::generators::Generator;
use aethellib::generators::generator_person::PersonGenerator;

let generator = PersonGenerator::from_file("data/person_test_data.toml")?;
let mut rng = StdRng::seed_from_u64(42);
let generated = generator.generate_with_rng(&mut rng);

println!("person: {}", generated.name.value);
# Ok::<(), Box<dyn std::error::Error>>(())
```

## data format

all input toml files must include a `header` section:

```toml
[header]
name = "dataset name"
target = "weapon" # or "person"
desc = "optional"
author = "optional"
version = "optional"
```

target sections are target-specific:

- weapon: `name`, `type`, `qualities`, `lore`, `visuals`
- person (minimal): `name.first`, `name.middle`, `name.last`

see reference fixtures in:
- [data/weapon_test_data.toml](data/weapon_test_data.toml)
- [data/person_test_data.toml](data/person_test_data.toml)

## important

the `data` directory is temporary and only exists to support the current fixtures and examples.
when it is removed, tests and examples will be updated to use the replacement test data flow.

## architecture overview

1. loaders parse and validate target (`src/loader/**`)
2. merge builds target corpora and keeps source-level metadata (`src/merge/mod.rs`)
3. generators build output values from corpus candidate pools (`src/generators/**`)

key model types:
- `AethelCorpus<T>`: per-target corpus with ordered source documents
- `SourceAethelDoc<T>`: one source document with metadata + body
- `MergedAethelDoc`: mixed-target merge result enum
- `GeneratedField<T>`: generated value + provenance refs

## examples

run all examples:

```bash
cargo run --example weapongen_from_file
cargo run --example weapongen_from_documents
cargo run --example weapongen_new_corpus
cargo run --example weapongen_provenance_from_file
cargo run --example weapongen_provenance_from_documents
cargo run --example weapongen_provenance_new_corpus
cargo run --example persongen_from_file
```

## adding a new target

use the exact target authoring guide:
- [docs/adding-new-target.md](docs/adding-new-target.md)

that document covers loader wiring, merge variant/accessors, generator trait implementation, fixtures, tests, and validation commands.

## development

standard checks:

```bash
cargo test
cargo check --examples
cargo clippy --all-targets --all-features
```

## error model

loader and merge read/parse errors include file path context.

example:

```text
unable to parse toml file 'data/weapon_bad.toml': expected a right bracket, found eof
```
