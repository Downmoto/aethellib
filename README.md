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

the examples directory is the primary practical documentation for this crate.
all examples are self-contained and create temporary toml inputs at runtime.

start with these:

- [examples/01_weapon_from_file.rs](examples/01_weapon_from_file.rs)
- [examples/02_person_from_file.rs](examples/02_person_from_file.rs)
- [examples/03_weapon_deterministic_rng.rs](examples/03_weapon_deterministic_rng.rs)
- [examples/10_merge_mixed_targets.rs](examples/10_merge_mixed_targets.rs)
- [examples/13_custom_target.rs](examples/13_custom_target.rs)

## api coverage map

| api area | what to run |
| --- | --- |
| weapon generator from file (`WeaponGenerator::from_file`, `Generator::generate`) | [examples/01_weapon_from_file.rs](examples/01_weapon_from_file.rs) |
| person generator from file (`PersonGenerator::from_file`, `Generator::generate`) | [examples/02_person_from_file.rs](examples/02_person_from_file.rs) |
| deterministic generation (`Generator::generate_with_rng`) | [examples/03_weapon_deterministic_rng.rs](examples/03_weapon_deterministic_rng.rs) |
| generator from source documents (`Generator::from_documents`, `SourceAethelDoc<T>`) | [examples/04_generator_from_documents.rs](examples/04_generator_from_documents.rs) |
| generator from explicit corpus (`Generator::new`, `AethelCorpus<T>`) | [examples/05_generator_new_with_corpus.rs](examples/05_generator_new_with_corpus.rs) |
| typed loader flow (`TargetedLoader::from_file`, `AethelDoc<T>`, `TARGET_WEAPON`) | [examples/06_targeted_loader_from_file.rs](examples/06_targeted_loader_from_file.rs) |
| loader errors (`LoaderError::ReadError`, `LoaderError::ParseError`, `LoaderError::TargetMismatch`) | [examples/07_loader_error_handling.rs](examples/07_loader_error_handling.rs) |
| single-target merging (`merge_target_files::<T>`) | [examples/08_merge_target_files.rs](examples/08_merge_target_files.rs) |
| merge options (`MergeOptions`, `MergerOptionError`) | [examples/09_merge_with_options.rs](examples/09_merge_with_options.rs) |
| mixed-target merge dispatch (`merge_from_files`, `AethelCorpus::target`) | [examples/10_merge_mixed_targets.rs](examples/10_merge_mixed_targets.rs) |
| merged doc conversion (`AethelCorpus::to_corpus`, `AethelCorpus::into_corpus`) | [examples/11_merged_doc_conversions.rs](examples/11_merged_doc_conversions.rs) |
| provenance (`GeneratedField<T>`, `SourceRef`) | [examples/12_weapon_provenance.rs](examples/12_weapon_provenance.rs) |
| custom target extensibility (`TargetedLoader`, `Generator`) | [examples/13_custom_target.rs](examples/13_custom_target.rs) |
| parsed aetheldoc casting (`SourceAethelDoc::from_aetheldocs`, `SourceAethelDoc<T>`) | [examples/14_generator_from_aetheldoc.rs](examples/14_generator_from_aetheldoc.rs) |
| single parsed aetheldoc cast (`SourceAethelDoc::from_aetheldoc`, `Generator::from_documents`) | [examples/15_generator_from_aetheldoc_single.rs](examples/15_generator_from_aetheldoc_single.rs) |

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

example fixtures are created as temporary files at runtime, so examples do not depend on checked-in data files.

## architecture overview

1. loaders parse and validate target (`src/loader/**`)
2. merge builds target corpora and keeps source-level metadata (`src/merger/mod.rs`)
3. generators build output values from corpus candidate pools (`src/generator/**`)

key model types:
- `AethelCorpus<T>`: per-target corpus with ordered source documents
- `SourceAethelDoc<T>`: one source document with metadata + body
- `AethelCorpus<Mixed>`: mixed-target merge result carrying target id + untyped tables
- `GeneratedField<T>`: generated value + provenance refs

## examples

run all examples:

```bash
cargo run --example 01_weapon_from_file --features weapon-gen
cargo run --example 02_person_from_file --features person-gen
cargo run --example 03_weapon_deterministic_rng --features weapon-gen
cargo run --example 04_generator_from_documents --features weapon-gen
cargo run --example 05_generator_new_with_corpus --features weapon-gen
cargo run --example 06_targeted_loader_from_file --features weapon-gen
cargo run --example 07_loader_error_handling
cargo run --example 08_merge_target_files --features weapon-gen
cargo run --example 09_merge_with_options
cargo run --example 10_merge_mixed_targets
cargo run --example 11_merged_doc_conversions
cargo run --example 12_weapon_provenance --features weapon-gen
cargo run --example 13_custom_target
cargo run --example 14_generator_from_aetheldoc --features weapon-gen
cargo run --example 15_generator_from_aetheldoc_single --features weapon-gen
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
