---
title: Adding new Target
description: A reference page in my new Starlight docs site.
---

# adding a new target to aethellib

this guide is the exact playbook for adding a new target end to end.

it covers:
- loader
- merge flow usage
- generator
- feature wiring
- test data
- tests
- examples
- docs updates

## target naming used in this guide

replace target_name with your target (example: person, creature, settlement).

replace TargetName with pascal case type/module naming (example: Person, Creature, Settlement).

replace target-name-gen with your feature flag name (example: settlement-gen).

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
- add a feature-gated export for your loader module.

pattern:
- #[cfg(feature = "target-name-gen")]
- pub mod loader_target_name;

optional (for consistency with built-ins):
- add TARGET_TARGET_NAME constant next to TARGET_WEAPON and TARGET_PERSON.

## 3. wire feature flags and examples

edit file:
- Cargo.toml

required changes:
1. add a dedicated feature for your generator (example: target-name-gen = []).
2. if built-in should include it by default, append it to built-in.
3. if you add a new example binary for the target generator, add [[example]] with required-features = ["target-name-gen"].

## 4. merge orchestration and conversion

edit file if needed:
- src/merger/mod.rs

important current behavior:
1. no target-specific merge dispatch wiring is required for merge_from_files; it groups by header.target generically.
2. typed merge for one known target should use merge_target_files::<TargetNameLoader>(paths, opts).
3. mixed-target input should use merge_from_files and then convert each AethelCorpus<Mixed> via:
  - merged_doc.into_corpus::<TargetNameLoader>()
  - or merged_doc.to_corpus::<TargetNameLoader>() when the untyped corpus must be retained.

required tests:
- single-target grouping
- mixed-target first-seen ordering
- typed conversion success with into_corpus or to_corpus
- typed conversion mismatch error path

reference implementation:
- src/merger/mod.rs
- examples/10_merge_mixed_targets.rs
- examples/11_merged_doc_conversions.rs

## 5. add generator module

create file:
- src/generator/generator_target_name.rs

required contents:
- module rustdoc at top
- generated output struct (GeneratedTargetName)
- generator struct (TargetNameGenerator)
- impl Generator for TargetNameGenerator from src/generator/mod.rs
- required trait methods:
  - new(corpus)
  - generate_with_rng(...)
- do not duplicate default methods supplied by the trait:
  - from_documents
  - from_file
  - generate
- candidate index built with helper functions from src/generator/utils.rs when appropriate
- provenance wiring using GeneratedField and SourceRef when relevant

required tests:
- generation happy path
- deterministic same-seed behavior for generate_with_rng

note:
- import the Generator trait anywhere you call trait-provided methods on concrete generators.
- example: use aethellib::generator::Generator;

reference implementation:
- src/generator/generator_person.rs

## 6. export generator module

edit file:
- src/generator/mod.rs

required change:
- add a feature-gated module export.

pattern:
- #[cfg(feature = "target-name-gen")]
- pub mod generator_target_name;

## 7. add fixture data

create file:
- data/target_name_test_data.toml

recommended contents:
- [header] with name, target, optional metadata
- minimal body sections used by generator

note:
- current examples mostly generate temporary toml at runtime; checked-in data files are still useful for manual testing and docs.

reference implementation:
- data/person_test_data.toml

## 8. update examples as needed

check files in:
- examples/

current repo pattern:
- mixed-target handling is based on corpus.target() string checks, not a hardcoded enum match.
- if examples call from_file, from_documents, or generate on concrete generators, import Generator trait.

## 9. add a minimal example for the new target

create at least one example file:
- examples/xx_target_name_from_file.rs

minimum behavior:
- create or load target input
- create generator via from_file
- print generated output

optional:
- add corpus-based and document-based examples, matching existing patterns.

## 10. documentation updates

update:
- README.md

include:
- new target support statement
- the recommended mixed-target extraction pattern (merge_from_files + to_corpus/into_corpus)
- deterministic generation mention if relevant
- feature flag and example usage notes

optional:
- add target-specific notes under docs/ if schema grows.

## 11. validation commands

run all commands before finishing:

1. cargo test
2. cargo check --examples
3. cargo clippy --all-targets --all-features

all three must pass.

## 12. pull request acceptance checklist

all items must be true:

1. new loader file exists and is exported from src/loader/mod.rs with expected feature gate
2. new generator file exists and is exported from src/generator/mod.rs with expected feature gate
3. Cargo.toml includes target feature wiring and example required-features entries as needed
4. mixed-target corpus conversions via AethelCorpus<toml::Table>::into_corpus and to_corpus are validated
5. generator implements Generator trait with new and generate_with_rng
6. fixture data exists when relevant for tests/docs
7. tests cover parse, merge behavior, conversion, and deterministic generation
8. examples compile and run with correct feature flags
9. README updated
10. cargo test, cargo check --examples, cargo clippy all pass

## generator trait quick template

use rand::Rng;

use crate::generator::Generator;
use crate::loader::loader_target_name::TargetNameLoader;
use crate::AethelCorpus;

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

## minimal map of files touched for a new target

- src/loader/loader_target_name.rs
- src/loader/mod.rs
- src/generator/generator_target_name.rs
- src/generator/mod.rs
- src/merger/mod.rs (tests and docs updates as needed)
- Cargo.toml
- data/target_name_test_data.toml (optional but recommended)
- examples/xx_target_name_from_file.rs
- README.md

## person target as canonical minimal template

if you want the smallest working reference in this repository, follow person as the model:
- src/loader/loader_person.rs
- src/generator/generator_person.rs
- src/merger/mod.rs
- examples/02_person_from_file.rs
