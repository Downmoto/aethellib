# aethellib 0.4.0 implementation plan

this document defines the 0.4.0 scope as five ordered commits.

the commit order is fixed:
1. provenance ergonomics
2. merge-from-memory api
3. prelude module
4. normalized error surface
5. validation hook extension points

## release goals

- improve day-to-day ergonomics for downstream users
- keep single-target architecture intact
- avoid locking unstable policy into core before 1.0
- keep migration cost low across 0.x

## commit 1: provenance ergonomics

### objective

add convenience query helpers so provenance metadata is easy to use without manual loops.

### proposed api additions

- `impl<T> AethelCorpus<T>`
- `fn source_ids(&self) -> Vec<&str>`
- `fn source_paths(&self) -> Vec<&str>`
- `fn find_source(&self, source_id: &str) -> Option<&SourceAethelDoc<T>>`

- `impl SourceRef`
- `fn matches(&self, section: &str, field: &str) -> bool`

- `impl<T> GeneratedField<T>`
- `fn has_source_id(&self, source_id: &str) -> bool`
- `fn source_ids(&self) -> Vec<&str>`
- `fn source_paths_in<'a, U>(&self, corpus: &'a AethelCorpus<U>) -> Vec<&'a str>`

### files expected

- `src/lib.rs`
- `src/generator/mod.rs`
- `examples/test.rs`

### acceptance criteria

- consumers can answer common provenance questions without custom iteration helpers
- no behavioural change to existing merge or generation flows
- example demonstrates at least one new provenance helper

### commit message suggestion

`feat(provenance): add convenience query helpers for source lookups`

---

## commit 2: merge-from-memory api

### objective

expose a public merge entrypoint for in-memory source payloads to avoid temporary file creation.

### proposed api additions

- `pub struct InMemoryMergeSource<'a>`
- `pub source_name: &'a str`
- `pub raw: &'a str`

- in `src/merger/mod.rs`
- `pub fn merge_sources<T>(sources: &[InMemoryMergeSource<'_>], opts: Option<MergeOptions>) -> Result<AethelCorpus<T>, MergerError>`

### internal approach

- map `InMemoryMergeSource` to existing internal source representation
- reuse current parsing and source hash logic
- preserve deterministic source id behaviour

### files expected

- `src/merger/mod.rs`
- `src/merger/utils.rs`
- `examples/test.rs`

### acceptance criteria

- callers can merge without filesystem paths
- output corpus shape and ids follow existing deterministic rules
- no duplicate implementation of parse/validation logic

### commit message suggestion

`feat(merger): add merge_sources api for in-memory toml inputs`

---

## commit 3: prelude module

### objective

add a low-friction import path for common traits and core model types.

### proposed api additions

- `pub mod prelude;` in crate root

- `src/prelude.rs` re-exports:
- `Generator`
- `TargetedLoader`
- `merge_files`
- `MergeOptions`
- `AethelDoc`
- `AethelDocHeader`
- `AethelCorpus`
- `SourceAethelDoc`
- `GeneratedField`
- `SourceRef`

### files expected

- `src/lib.rs`
- `src/prelude.rs` (new)
- `examples/test.rs` (optionally show prelude usage)

### acceptance criteria

- downstream users can start with one import path for core workflows
- no type duplication, only re-exports
- crate docs and example compile cleanly with prelude imports

### commit message suggestion

`feat(api): add prelude module for common aethellib imports`

---

## commit 4: normalized error surface

### objective

add machine-readable error categories while preserving human-readable display messages.

### proposed api additions

- in loader errors:
- `pub enum LoaderErrorKind { Read, Parse, TargetMismatch }`
- `fn kind(&self) -> LoaderErrorKind`

- in merger errors:
- `pub enum MergerErrorKind { LoaderRead, LoaderParse, LoaderTargetMismatch, OptionViolation, InvalidInput }`
- `fn kind(&self) -> MergerErrorKind`

### behaviour

- keep existing `Display` messages intact
- expose stable kind values for downstream programmatic handling

### files expected

- `src/loader/error.rs`
- `src/merger/error.rs`
- `examples/test.rs` (optional demonstration)

### acceptance criteria

- downstream code can branch on `kind()` instead of parsing message strings
- no change in current human-facing error text unless explicitly improved

### commit message suggestion

`feat(errors): add stable error kind enums and accessors`

---

## commit 5: validation hook extension points

### objective

allow user-defined validation policies during merge without hardcoding policy into aethellib.

### proposed api additions

- in `src/merger/mod.rs`:
- `pub trait MergeValidator<T>`
- `fn validate(&self, document: &AethelDoc<T>, source_path: &str) -> Result<(), MergerError>`

- add hook-capable merge variants:
- `pub fn merge_files_with_validator<T>(paths: &[impl AsRef<std::path::Path>], opts: Option<MergeOptions>, validator: &impl MergeValidator<T>) -> Result<AethelCorpus<T>, MergerError>`
- `pub fn merge_sources_with_validator<T>(sources: &[InMemoryMergeSource<'_>], opts: Option<MergeOptions>, validator: &impl MergeValidator<T>) -> Result<AethelCorpus<T>, MergerError>`

### internal approach

- run validator after parse + target validation and before final corpus push
- reuse existing error plumbing

### files expected

- `src/merger/mod.rs`
- `src/merger/utils.rs`
- `src/merger/error.rs` (only if needed for hook-specific error variant)
- `examples/test.rs`

### acceptance criteria

- users can enforce custom schema or business rules without forking core merge logic
- hook path does not affect default merge performance materially
- base merge apis remain available and unchanged

### commit message suggestion

`feat(merger): add validator hooks for custom merge policies`

---

## implementation notes

- keep each commit independently buildable and example-runnable
- run on each commit:
- `cargo run --example test`
- `cargo test -q`
- `cargo clippy --all-targets --all-features -q`

## out of scope for 0.4.0

- broad documentation expansion
- test completeness for every new edge case
- mixed-target orchestration inside aethellib core

## definition of done

- all five commits merged in listed order
- example remains green and demonstrates new surfaces
- old APIs still usable unless explicitly replaced in changelog

---

## roadmap extension: automatic provenance generation

this section extends the roadmap with an implementation path for automatic provenance during generation, based on current architecture constraints.

### key constraint to acknowledge

automatic provenance is fully possible in aethellib when value selection flows through shared library helpers.

automatic provenance is not inferable in arbitrary user generation code if users bypass helper paths and construct outputs manually.

### design goal

- make provenance automatic for common generation workflows
- keep custom generators possible without hard constraints
- avoid forcing a full generation dsl pre-1.0

### phased implementation model

#### phase a: provenance candidate index (foundational)

add a reusable index that maps generated candidate values to all source references that can produce them.

proposed additions:

- `pub struct ProvenanceCandidateIndex<T>`
- `fn new() -> Self`
- `fn insert(&mut self, value: T, source_ref: SourceRef)`
- `fn refs_for(&self, value: &T) -> &[SourceRef]`
- `fn contains_value(&self, value: &T) -> bool`

type bounds direction:

- keep `T: Eq + std::hash::Hash + Clone` for index keys
- store refs as deduplicated vectors preserving insertion order

expected files:

- `src/generator/mod.rs`
- `src/lib.rs` (if public type placement belongs there)
- `examples/test.rs`

acceptance criteria:

- users can build a candidate-to-provenance map without handwritten hashmaps
- identical value emitted by multiple sources returns full source ref set

commit message suggestion:

`feat(provenance): add candidate index for value-to-source mapping`

#### phase b: automatic sampling helpers (turnkey path)

add shared helper APIs that sample values and return `GeneratedField<T>` automatically.

proposed additions:

- `pub fn sample_generated_field<T, R>(values: &[T], index: &ProvenanceCandidateIndex<T>, rng: &mut R) -> Option<GeneratedField<T>>`
- `pub fn choose_generated_field<T, R>(pairs: &[(T, Vec<SourceRef>)], rng: &mut R) -> Option<GeneratedField<T>>`

behaviour rules:

- return `None` for empty candidate set
- preserve deterministic behaviour under deterministic rng input
- attach complete ref set for the selected value

expected files:

- `src/generator/mod.rs`
- `examples/test.rs`

acceptance criteria:

- common generators can remove manual provenance assembly code
- example demonstrates direct helper use to produce output with refs

commit message suggestion:

`feat(generator): add automatic provenance sampling helpers`

#### phase c: optional provenance-first trait surface

provide an opt-in trait for users who want provenance-aware output by default.

proposed additions:

- `pub trait ProvenanceGenerator: Generator`
- associated type guidance:
- `type Value`
- output contract:
- `Generator::Output = GeneratedField<Self::Value>`

possible trait shape:

- `fn generate_field_with_rng<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> GeneratedField<Self::Value>`
- default bridge:
- `fn generate_with_rng<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Self::Output`

note:

this remains optional to avoid breaking existing generators that return plain value types.

expected files:

- `src/generator/mod.rs`
- `examples/test.rs`

acceptance criteria:

- users can adopt a provenance-first trait without rewriting core merge/load logic
- existing `Generator` implementations remain valid

commit message suggestion:

`feat(generator): add optional provenance-first generator trait`

#### phase d: ergonomic documentation and migration path

define a clear supported path that guarantees automatic provenance.

required guidance text:

- if generation uses library candidate index + sampling helpers, provenance is automatic
- if generation bypasses helpers, provenance must be attached manually
- recommend `ProvenanceGenerator` for new implementations that want guaranteed provenance output

acceptance criteria:

- users understand when provenance is guaranteed versus optional
- examples show both baseline and provenance-first patterns

commit message suggestion:

`docs(generator): document automatic provenance helper path and guarantees`

### ordering relative to current 0.4.0 commits

the original five 0.4.0 commits remain unchanged and in the same order.

recommended placement for this extension:

- after commit 1 (provenance ergonomics), start phase a and phase b
- phase c can follow commit 3 (prelude), since re-exports may include new provenance types
- phase d can be finalized after commit 5

### risk and compatibility notes

- low risk for merge/load logic: changes are generation-layer additions
- medium risk for api surface sprawl if helper names are too broad
- keep helper naming explicit (`generated_field`, `provenance`, `candidate_index`) to reduce confusion

compatibility posture:

- additive only during 0.4.x
- no forced migration for existing generators

### done criteria for this extension

- end users can get provenance-rich generation output without writing custom ref assembly loops
- helper path is deterministic, documented, and example-backed
- existing generator workflows continue to compile unchanged

### commit-slot mapping (one feature per commit)

this maps the extension into explicit commit slots that fit the existing 0.4.0 plan without rewriting the original five commits.

slot sequence:

1. `1` commit 1: provenance ergonomics
2. `1a` phase a: provenance candidate index
3. `1b` phase b: automatic provenance sampling helpers
4. `2` commit 2: merge-from-memory api
5. `3` commit 3: prelude module
6. `3a` phase c: optional provenance-first trait
7. `4` commit 4: normalized error surface
8. `5` commit 5: validation hook extension points
9. `5a` phase d: provenance helper path documentation and migration guidance

### slot details

#### slot `1a` (after commit 1)

scope:

- add `ProvenanceCandidateIndex<T>`
- wire minimal constructor/insert/query methods
- update example to demonstrate index creation from merged sources

acceptance checks:

- `cargo run --example test`
- `cargo test -q`
- `cargo clippy --all-targets --all-features -q`

commit message suggestion:

`feat(provenance): add candidate index for automatic source mapping`

#### slot `1b` (after slot 1a)

scope:

- add helper samplers that return `GeneratedField<T>`
- ensure deterministic behaviour with seeded rng
- update example to use helper path instead of manual provenance assembly where possible

acceptance checks:

- `cargo run --example test`
- `cargo test -q`
- `cargo clippy --all-targets --all-features -q`

commit message suggestion:

`feat(generator): add automatic GeneratedField sampling helpers`

#### slot `3a` (after commit 3)

scope:

- add optional `ProvenanceGenerator` trait
- keep `Generator` trait unchanged and compatible
- optionally re-export trait in prelude if naming is stable

acceptance checks:

- existing generator implementations compile with no source changes
- provenance-first generator example compiles and runs

commit message suggestion:

`feat(generator): add optional provenance-first generator trait`

#### slot `5a` (after commit 5)

scope:

- add concise documentation section clarifying provenance guarantees
- define supported automatic path and manual path expectations
- include migration notes for adopters moving from manual provenance wiring

acceptance checks:

- docs text matches actual helper and trait names in code
- example references are valid and current

commit message suggestion:

`docs(provenance): define automatic provenance guarantees and migration path`

### final ordered checklist for execution

- [x] commit `1`: provenance ergonomics
- [x] commit `1a`: provenance candidate index
- [x] commit `1b`: automatic provenance sampling helpers
- [x] commit `2`: merge-from-memory api
- [x] commit `3`: prelude module
- [x] commit `3a`: optional provenance-first trait
- [ ] commit `4`: normalized error surface
- [ ] commit `5`: validation hook extension points
- [ ] commit `5a`: provenance docs and migration guidance
