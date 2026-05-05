# adding a new target to aethellib

this guide is the exact playbook for adding a new target end to end.

it covers:
- loader
- merge orchestration wiring
- merge dispatch wiring
- generator
- test data
- tests
- examples
- docs updates

## target naming used in this guide

replace target_name with your target (example: person, creature, settlement).

replace TargetName with pascal case type/module naming (example: Person, Creature, Settlement).

## 1. add loader module

create file:
- src/loader/loader_target_name.rs

required contents:
- module rustdoc at top
- loader struct for target body
- minimal target sections
- impl TargetedLoader with const TARGET set correctly
- loader tests for:
  - happy path parse
  - missing body sections allowed
  - missing fields in existing section allowed

note:
- do not add an inherent from_file on the loader.
- use the trait-provided method from TargetedLoader.

reference implementation:
- src/loader/loader_person.rs

## 2. export loader module

edit file:
- src/loader/mod.rs

required change:
- add pub mod loader_target_name;

note:
- define `const TARGET: &'static str` in your loader to match the header target string.

## 3. wire target into merge orchestration

edit file:
- src/merge/mod.rs

required changes:
1. no merge dispatch wiring is required for generic merge_from_files.
2. use typed merge for your loader:
  - merge_target_files::<TargetNameLoader>(paths, opts)
3. for mixed-target inputs, use merge_from_files and convert each merged doc:
  - merged_doc.into_corpus::<TargetNameLoader>()
4. add merge tests for:
   - single-target grouping
   - mixed-target first-seen ordering
  - path-based corpus and source-based corpus equivalence
  - typed conversion via into_corpus::<TargetNameLoader>()

reference implementation:
- src/merger/mod.rs

## 4. add generator module

create file:
- src/generators/generator_target_name.rs

required contents:
- module rustdoc at top
- generated output struct (GeneratedTargetName)
- generator struct (TargetNameGenerator)
- impl Generator for TargetNameGenerator from src/generators/mod.rs
- required trait methods:
  - new(corpus)
  - generate_with_rng(...)
- do not duplicate default methods supplied by the trait:
  - from_documents
  - from_file
  - generate
- candidate index built with build_pool helpers from src/generators/mod.rs
- provenance wiring using GeneratedField and SourceRef

required tests:
- generation happy path
- deterministic same-seed behaviour for generate_with_rng

note:
- import the Generator trait anywhere you call trait-provided methods on concrete generators.
- example: use aethellib::generators::Generator;

reference implementation:
- src/generators/generator_person.rs

## 5. export generator module

edit file:
- src/generators/mod.rs

required change:
- add pub mod generator_target_name;

## 6. add fixture data

create file:
- data/target_name_test_data.toml

required contents:
- [header] with name, target, optional metadata
- minimal body sections used by generator

reference implementation:
- data/person_test_data.toml

## 7. update existing examples for enum exhaustiveness

when mixed-target examples branch by target, existing matches may become non-exhaustive as new targets are added.

check and update files in:
- examples/

pattern:
- add explicit branch for unrelated variants and return an error when expected variant is missing.
- if examples call from_file, from_documents, or generate on concrete generators, import Generator trait.

## 8. add a minimal example for the new target

create at least one example file:
- examples/targetgen_from_file.rs

minimum behaviour:
- load from data/target_name_test_data.toml
- create generator via from_file
- print generated output

optional:
- add corpus and document-based examples, matching existing weapon pattern.

## 9. documentation updates

update:
- README.md

include:
- new target support statement
- merge extraction approach for new variant
- deterministic generation mention if relevant

optional:
- add target-specific notes under docs/ if schema grows.

## 10. validation commands

run all commands before finishing:

1. cargo test
2. cargo check --examples
3. cargo clippy --all-targets --all-features

all three must pass.

## 11. pull request acceptance checklist

all items must be true:

1. new loader file exists and is exported
2. merge entrypoint exists in src/merge/mod.rs
3. merge dispatch handles target without unsupported-target error
4. mixed-target corpus conversions via `AethelCorpus<toml::Table>::into_corpus` and `to_corpus` are validated
5. generator implements Generator trait with new and generate_with_rng
6. fixture data exists for tests/examples
7. tests cover parse, merge parity, and deterministic generation
8. examples compile after enum changes
9. README updated
10. cargo test, cargo check --examples, cargo clippy all pass

## generator trait quick template

```rust
use rand::Rng;

use crate::generators::Generator;
use crate::loader::loader_target_name::TargetNameLoader;
use crate::merge::AethelCorpus;

pub struct TargetNameGenerator {
  index: TargetNameIndex,
}

impl Generator for TargetNameGenerator {
  type Loader = TargetNameLoader;
  type Output = GeneratedTargetName;

  fn new(corpus: AethelCorpus<Self::Loader>) -> Self {
    let index = TargetNameIndex::from_documents(&corpus.documents);
    Self { index }
  }

  fn generate_with_rng<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::Output {
    // target-specific generation
    unimplemented!()
  }
}
```

## minimal map of files touched for a new target

- src/loader/loader_target_name.rs
- src/loader/mod.rs
- src/merge/mod.rs
- src/generators/generator_target_name.rs
- src/generators/mod.rs
- data/target_name_test_data.toml
- examples/targetgen_from_file.rs
- README.md

## person target as canonical minimal template

if you want the smallest working reference in this repository, follow person as the model:
- src/loader/loader_person.rs
- src/merge/mod.rs
- src/generators/generator_person.rs
- data/person_test_data.toml
