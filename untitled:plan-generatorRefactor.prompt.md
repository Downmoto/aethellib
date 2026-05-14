## Plan: Generator to Generation v0.5 Step 1

Refactor the generator API with an intentional hard break in v0.5.0 to reduce user boilerplate and establish a richer provenance model: rename Generator to Generation, move generation behaviour to inherent methods on the user-defined generation struct, add a compile step that prepares reusable sampling state, and redesign GeneratedField to support private internals plus structured dependency-graph trace metadata. Keep loader and merger compatibility unchanged.

**Steps**
1. Phase 1: Contract and type-shape redesign (blocks all later work)
2. Define the new Generation trait in src/generator/mod.rs with required methods focused on lifecycle, including new(corpus) and compile(corpus) -> Self::CompiledState (or equivalent associated type contract if compile returns a prepared state owned by Self).
3. Remove legacy generation methods from the trait contract (generate and generate_with_rng) and shift runtime generation calls into inherent methods on user generation structs.
4. Remove the Generated marker trait and its get helper from src/generator/mod.rs to simplify the output contract (single user type holding GeneratedField members).
5. Redesign GeneratedField<T> as encapsulated (private internals) with method-based API in src/generator/mod.rs and add structured trace storage intended for dependency-graph provenance.
6. Define a Result-based generation error model in src/generator (new error type or module) and wire GeneratedField and generation flows to return Result instead of panic-driven behaviour.
7. Phase 2: Ergonomics API for schema declaration (depends on 1-6)
8. Introduce a fluent builder DSL in src/generator/utils.rs (or a new builder module under src/generator/) to register field candidate extraction without manual index plumbing.
9. Ensure the DSL emits or compiles into reusable prepared sampling state so one generation object can produce many outputs efficiently.
10. Keep convenience constructors as trait defaults (from_file, from_files, from_documents) on Generation, preserving the current loader/merger entrypoint behaviour.
11. Phase 3: Module exports, examples, and docs (parallel with Phase 2 once core trait signatures settle)
12. Update root exports and prelude to expose Generation and updated GeneratedField API in src/prelude.rs and src/lib.rs.
13. Update example usage in examples/test.rs to the new single-type generation pattern (same type declares GeneratedField members and owns inherent generation methods).
14. Update crate-level and module docs in src/generator/mod.rs and README.md to document the new lifecycle: construct -> compile -> generate via inherent methods, and Result-based error handling.
15. Add a roadmap subsection in README.md for derive macro follow-up with concrete acceptance criteria (no derive implementation in this step).
16. Phase 4: Release prep and verification (depends on 1-15)
17. Bump crate version from 0.4.0 to 0.5.0 in Cargo.toml after step-1 refactor lands complete.
18. Run tests/examples and fix breakages caused by hard API changes.
19. Final pass: verify loader/merger API compatibility and unchanged input format behaviour.

**Relevant files**
- /Users/arad/Developer/aethellib/src/generator/mod.rs — primary trait/type refactor: Generation contract, GeneratedField redesign, removal of Generator/Generated legacy behaviour.
- /Users/arad/Developer/aethellib/src/generator/utils.rs — builder DSL and candidate compilation helpers.
- /Users/arad/Developer/aethellib/src/prelude.rs — export surface updates from Generator to Generation and updated field API.
- /Users/arad/Developer/aethellib/src/lib.rs — crate root re-export updates and documentation alignment.
- /Users/arad/Developer/aethellib/examples/test.rs — migrate to new generation usage pattern and Result-based generation flow.
- /Users/arad/Developer/aethellib/README.md — migration notes, v0.5.0 usage docs, derive roadmap milestone.
- /Users/arad/Developer/aethellib/Cargo.toml — version bump to 0.5.0 after code complete.

**Verification**
1. Compile check: cargo check confirms Generator/Generated legacy references are removed and Generation-based API compiles.
2. Test run: cargo test passes for existing loader and merger tests and updated generation example coverage.
3. Example run: cargo test --example test (or equivalent example command in this repo) passes with new generation lifecycle.
4. Behaviour validation: verify from_file, from_files, and from_documents still load/merge equivalently to pre-refactor behaviour.
5. Provenance validation: assert GeneratedField exposes source provenance and dependency-graph trace events through public methods only.
6. Error handling validation: empty candidate scenarios and compile/generate failures return Result errors, with no panic in library paths.

**Decisions**
- Hard break in v0.5.0 is in scope and acceptable.
- Generation trait remains full lifecycle contract but generation execution methods move to inherent impls.
- User-facing generation type is single-type (schema and output together), with every generated value represented by GeneratedField<T>.
- Provenance target is field-level plus structured dependency-graph tracing from step 1.
- Builder DSL is required in step 1 to reduce current boilerplate.
- Loader and merger formats/semantics are explicitly out of scope for behavioural changes.
- Version bump occurs only once step-1 refactor is complete.


**Contract Draft (Step 1 locked)**
- Generation trait intent: full lifecycle coordination for a reusable compiled sampler; generation execution is not a required trait method.
- Required associated types:
  - Loader: TargetedLoader
  - CompiledState: reusable prepared state used by inherent generation methods
  - Error: generation domain error type used by compile and generation paths
- Required lifecycle methods on Generation:
  - new(corpus: AethelCorpus<Self::Loader>) -> Self
  - compile(&mut self) -> Result<(), Self::Error>
- Required default constructors on Generation (trait defaults preserved):
  - from_documents(documents) -> Self
  - from_file(path) -> Result<Self, MergerError>
  - from_files(paths) -> Result<Self, MergerError>
- Non-goal for trait in step 1: no generate / generate_with_rng required methods.
- Inherent API expectation on user generation structs:
  - generate_with_rng(&self, rng) -> Result<Self, GenerationError>
  - generate(&self) -> Result<Self, GenerationError>
  - both rely on already-compiled internal state; generate uses thread-local rng.
- GeneratedField<T> contract:
  - private internals; no public value/source vectors.
  - constructor/factory methods are crate-controlled to preserve invariants.
  - read-only accessors for value, source refs, source ids, and source path resolution.
  - structured dependency-graph trace stored on each field (per-field graph value, not global singleton).
- Trace graph minimum schema for step 1:
  - node kinds: source, selection, transform.
  - edge kinds: derived_from.
  - each generated field carries root node id and graph payload.
- Error handling contract:
  - library generation paths return Result; no panic on empty candidate pools.
  - baseline variants: NotCompiled, EmptyCandidates { field }, InvalidTraceGraph, BuilderDefinition { field, reason }.
- Builder DSL contract (required in step 1):
  - fluent registration of generated fields and extractors from loader payload.
  - compile-time output is reusable candidate/provenance state attached to the generation struct.
  - removes manual ProvenanceCandidateIndex construction from user code.
- Migration contract for v0.5.0:
  - hard break: remove Generator trait and Generated marker from public API.
  - export Generation and updated GeneratedField from prelude and crate root.
  - keep loader/merger input compatibility unchanged.

**Further Considerations**
1. Derive roadmap milestone: define acceptance criteria for a future derive macro that can auto-register builder field extractors while preserving explicit trace-graph hooks.
2. Performance guardrail: benchmark compile and generate throughput before and after v0.5.0 to ensure trace graph richness does not regress hot-path sampling noticeably.
3. Migration docs quality: include a concise old-to-new mapping table (Generator/Generated/public fields -> Generation/inherent methods/accessors) in the release notes.