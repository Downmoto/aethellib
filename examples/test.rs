//! manual api walkthrough for the current aethellib surface.
//!
//! this example intentionally exercises a wide portion of the public api in a
//! single executable flow so contributors can run one command and quickly
//! validate expected behaviour.
//!
//! the example covers:
//! - targeted loader parsing from raw text and from file paths
//! - typed single-target merging and merge option handling
//! - source document conversion helpers
//! - provenance indexing and generated field helpers
//! - generation constructors (`new`, `from_file`, `from_files`, `from_documents`)
//! - deterministic generation via seeded rng with result-based errors
//!
//! fixtures are created under `target/` and removed automatically at exit.

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use aethellib::Target;
use aethellib::generator::{
    GeneratedFieldCandidates, GenerationError, ProvenanceCandidateIndex,
    generated_field_builder,
};
use aethellib::loader::error::LoaderErrorKind;
use aethellib::merger::error::MergerError;
use aethellib::merger::error::MergerErrorKind;
use aethellib::merger::{
    InMemoryMergeSource, MergeValidator, merge_files_with_validator, merge_sources,
    merge_sources_with_validator,
};
use aethellib::prelude::{
    AethelCorpus, AethelDoc, AethelDocHeader, GeneratedField, Generation, MergeOptions,
    SourceAethelDoc, SourceRef, TargetedLoader, merge_files,
};
use rand::Rng;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};

struct FixtureDirGuard {
    path: PathBuf,
}

impl FixtureDirGuard {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn path(&self) -> &Path {
        self.path.as_path()
    }
}

impl Drop for FixtureDirGuard {
    fn drop(&mut self) {
        // best-effort cleanup keeps local runs tidy without failing the example.
        let _ = fs::remove_dir_all(self.path.as_path());
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ExampleLoader {
    // this field models one flattened top-level table in the example toml.
    values: ExampleValues,
}

impl TargetedLoader for ExampleLoader {
    const TARGET: &'static str = "example";
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ExampleValues {
    // one generated output candidate pool for demonstration purposes.
    words: Vec<String>,
}

struct ExampleGeneration {
    word: GeneratedField<String>,
    // prepared candidates let `new` stay small even as generated fields grow.
    generated_field: Option<GeneratedFieldCandidates<String>>,
}

impl ExampleGeneration {
    fn generate_with_rng<R: Rng + ?Sized>(&self, rng: &mut R) -> Result<Self, GenerationError> {
        // generated output is provenance-rich by default in this example.
        let generated_field = self
            .generated_field
            .as_ref()
            .ok_or(GenerationError::NotCompiled)?;
        let word = generated_field.sample("word", rng)?;

        Ok(Self {
            word,
            generated_field: self.generated_field.clone(),
        })
    }

    fn generate(&self) -> Result<Self, GenerationError> {
        let mut rng = rand::thread_rng();
        self.generate_with_rng(&mut rng)
    }

    fn word(&self) -> &GeneratedField<String> {
        &self.word
    }
}

impl Generation for ExampleGeneration {
    type Loader = ExampleLoader;
    type CompiledState = GeneratedFieldCandidates<String>;
    type Error = GenerationError;

    fn new(corpus: AethelCorpus<Self::Loader>) -> Self {
        // the builder compiles both candidate values and provenance refs.
        let generated_field = generated_field_builder(&corpus, "values", "words", |loader| {
            loader.values.words.clone()
        })
        .build();

        Self {
            word: GeneratedField::pending(String::new()),
            generated_field: Some(generated_field),
        }
    }

    fn compile(&mut self) -> Result<(), Self::Error> {
        if self.generated_field.is_none() {
            return Err(GenerationError::BuilderDefinition {
                field: "word".to_string(),
                reason: "missing compiled candidates".to_string(),
            });
        }

        Ok(())
    }
}

struct TitleContainsValidator {
    needle: &'static str,
}

impl MergeValidator<ExampleLoader> for TitleContainsValidator {
    fn validate(
        &self,
        document: &AethelDoc<ExampleLoader>,
        source_path: &str,
    ) -> Result<(), MergerError> {
        if document.header.title.contains(self.needle) {
            Ok(())
        } else {
            Err(MergerError::InvalidInput(format!(
                "validator rejected source '{source_path}': header.title must contain '{}'",
                self.needle
            )))
        }
    }
}

#[allow(deprecated)]
fn merge_with_legacy_alias(
    paths: &[impl AsRef<Path>],
) -> Result<AethelCorpus<ExampleLoader>, MergerError> {
    aethellib::merger::merge_target_files::<ExampleLoader>(paths, None)
}

fn main() -> Result<(), Box<dyn Error>> {
    // a private fixture root is created per run to avoid cross-run interference.
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_dir = FixtureDirGuard::new(make_fixture_dir(workspace_root.as_path())?);

    // build small raw documents so this example is self-contained.
    let raw_alpha = build_doc_raw("alpha set", "example", &["oak", "ash", "elm"]);
    let raw_beta = build_doc_raw("beta set", "example", &["birch", "cedar"]);
    let raw_person = build_doc_raw("other set", "person", &["wrong"]);
    let raw_dup_one = build_doc_raw("duplicate", "example", &["left"]);
    let raw_dup_two = build_doc_raw("duplicate", "example", &["right"]);

    // persist raw payloads to disk for file-based loader and merge entrypoints.
    let alpha_path = write_fixture(fixture_dir.path(), "alpha.toml", &raw_alpha)?;
    let beta_path = write_fixture(fixture_dir.path(), "beta.toml", &raw_beta)?;
    let person_path = write_fixture(fixture_dir.path(), "person.toml", &raw_person)?;
    let dup_one_path = write_fixture(fixture_dir.path(), "dup_one.toml", &raw_dup_one)?;
    let dup_two_path = write_fixture(fixture_dir.path(), "dup_two.toml", &raw_dup_two)?;

    // verify raw-string parsing path on the targeted loader trait.
    let parsed_inline = ExampleLoader::from_str("inline-alpha", &raw_alpha)?;
    assert_eq!(parsed_inline.header.target, ExampleLoader::TARGET);
    assert_eq!(parsed_inline.header.title, "alpha set");

    // verify file-path parsing path on the same loader trait.
    let parsed_file = ExampleLoader::from_file(&alpha_path)?;
    assert_eq!(parsed_file.header.title, "alpha set");

    // verify typed merge entrypoint plus corpus-level provenance helpers.
    let merge_paths = vec![alpha_path.clone(), beta_path.clone()];
    let corpus = merge_files::<ExampleLoader>(&merge_paths, None)?;
    assert_eq!(corpus.target(), "example");
    assert_eq!(corpus.documents.len(), 2);
    assert_eq!(corpus.source_ids().len(), 2);
    assert_eq!(corpus.source_paths().len(), 2);
    assert!(corpus.find_source(corpus.source_ids()[0]).is_some());

    // verify in-memory merge entrypoint for callers that already hold raw inputs.
    let in_memory_sources = [
        InMemoryMergeSource {
            source_name: "memory-alpha.toml",
            raw: raw_alpha.as_str(),
        },
        InMemoryMergeSource {
            source_name: "memory-beta.toml",
            raw: raw_beta.as_str(),
        },
    ];
    let in_memory_corpus = merge_sources::<ExampleLoader>(&in_memory_sources, None)?;
    assert_eq!(in_memory_corpus.target(), "example");
    assert_eq!(in_memory_corpus.documents.len(), 2);

    // verify validation-hook merge paths for file and in-memory entrypoints.
    let set_validator = TitleContainsValidator { needle: "set" };
    let validated_file_corpus =
        merge_files_with_validator::<ExampleLoader>(&merge_paths, None, &set_validator)?;
    assert_eq!(validated_file_corpus.documents.len(), 2);

    let validated_memory_corpus =
        merge_sources_with_validator::<ExampleLoader>(&in_memory_sources, None, &set_validator)?;
    assert_eq!(validated_memory_corpus.documents.len(), 2);

    let alpha_only_validator = TitleContainsValidator { needle: "alpha" };
    let rejected_file_merge =
        merge_files_with_validator::<ExampleLoader>(&merge_paths, None, &alpha_only_validator);
    assert!(rejected_file_merge.is_err());

    // verify deprecated alias remains functional during migration window.
    let corpus_legacy = merge_with_legacy_alias(&merge_paths)?;
    assert_eq!(corpus_legacy.documents.len(), 2);

    // verify merge option enforcement for duplicate titles.
    let strict_opts = MergeOptions {
        identical_title_allowed: false,
    };
    let dup_paths = vec![dup_one_path, dup_two_path];
    let dup_result = merge_files::<ExampleLoader>(&dup_paths, Some(strict_opts));
    assert!(dup_result.is_err());
    let dup_error = dup_result.err().ok_or("expected duplicate merge error")?;
    assert_eq!(dup_error.kind(), MergerErrorKind::OptionViolation);

    // verify target mismatch errors still propagate through loader parsing.
    let mismatch = ExampleLoader::from_file(&person_path);
    assert!(mismatch.is_err());
    let mismatch_error = mismatch.err().ok_or("expected target mismatch error")?;
    assert_eq!(mismatch_error.kind(), LoaderErrorKind::TargetMismatch);

    // verify target alias usage from public crate root.
    let target_value: Target = ExampleLoader::TARGET.to_string();
    assert_eq!(target_value, "example");

    // manually build one document to exercise source conversion helpers.
    let manual_doc = AethelDoc {
        header: AethelDocHeader {
            title: "manual set".to_string(),
            target: target_value,
            desc: Some("constructed in example".to_string()),
            author: Some("example".to_string()),
            version: Some("1.0".to_string()),
        },
        data: ExampleLoader {
            values: ExampleValues {
                words: vec!["manual".to_string(), "entry".to_string()],
            },
        },
    };

    // verify conversion from one parsed doc and many parsed docs.
    let source_single = SourceAethelDoc::from_aetheldoc(manual_doc.clone())?;
    let source_many = SourceAethelDoc::from_aetheldocs(vec![
        manual_doc,
        ExampleLoader::from_str("inline-beta.toml", &raw_beta)?,
    ])?;
    assert_eq!(source_many.len(), 2);

    // keep one small corpus for generator constructor coverage.
    let source_docs = vec![source_single.clone(), source_many[1].clone()];
    let corpus_from_sources = AethelCorpus {
        target: ExampleLoader::TARGET.to_string(),
        documents: source_docs,
    };
    assert_eq!(corpus_from_sources.target(), "example");

    // verify explicit provenance index construction from source documents.
    let mut candidate_index: ProvenanceCandidateIndex<String> = ProvenanceCandidateIndex::new();
    for source_doc in &corpus_from_sources.documents {
        for word in &source_doc.data.values.words {
            candidate_index.insert(
                word.clone(),
                SourceRef {
                    source_id: source_doc.source_id.clone(),
                    source_name: source_doc.header.title.clone(),
                    section: "values".to_string(),
                    field: "words".to_string(),
                },
            );
        }
    }
    assert!(candidate_index.contains_value(&"manual".to_string()));
    assert!(!candidate_index.refs_for(&"manual".to_string()).is_empty());

    // verify direct constructor path and deterministic generation.
    let mut from_new = ExampleGeneration::new(corpus_from_sources.clone());
    from_new.compile()?;
    let mut seeded_rng = rand::rngs::StdRng::seed_from_u64(7);
    let seeded_value = from_new.generate_with_rng(&mut seeded_rng)?;
    assert!(!seeded_value.word().value().is_empty());
    assert!(!seeded_value.word().source_refs().is_empty());

    // verify provenance convenience helpers on generated output.
    let mut provenance_rng = rand::rngs::StdRng::seed_from_u64(11);
    let generated_field = from_new.generate_with_rng(&mut provenance_rng)?;
    assert!(!generated_field.word().value().is_empty());
    assert!(!generated_field.word().source_refs().is_empty());
    assert_eq!(
        generated_field.word().source_refs()[0].section,
        "values"
    );
    assert_eq!(
        generated_field.word().source_refs()[0].field,
        "words"
    );
    assert!(
        generated_field.word().source_refs()[0].matches("values", "words")
    );
    assert!(
        generated_field
            .word()
            .has_source_id(generated_field.word().source_refs()[0].source_id.as_str())
    );
    assert!(!generated_field.word().source_ids().is_empty());
    assert!(!generated_field
        .word()
        .source_paths_in(&corpus_from_sources)
        .is_empty());
    assert!(!generated_field.word().trace_graph().nodes().is_empty());

    // verify file-backed constructor convenience.
    let mut from_file_generator = ExampleGeneration::from_files(&[alpha_path.as_str()])?;
    from_file_generator.compile()?;
    let generated_from_file = from_file_generator.generate()?;
    assert!(!generated_from_file.word().value().is_empty());
    assert!(!generated_from_file.word().source_refs().is_empty());

    // verify multi-file constructor convenience.
    let mut from_files_generator = ExampleGeneration::from_files(&merge_paths)?;
    from_files_generator.compile()?;
    let generated_from_files = from_files_generator.generate()?;
    assert!(!generated_from_files.word().value().is_empty());
    assert!(!generated_from_files.word().source_refs().is_empty());

    // verify source-document constructor convenience.
    let mut from_documents_generator = ExampleGeneration::from_documents(vec![
        source_single,
        source_many.into_iter().next().ok_or("missing source doc")?,
    ]);
    from_documents_generator.compile()?;
    let generated_from_documents = from_documents_generator.generate()?;
    assert!(!generated_from_documents.word().value().is_empty());
    assert!(!generated_from_documents.word().source_refs().is_empty());

    println!("example api walkthrough passed");
    Ok(())
}

fn make_fixture_dir(workspace_root: &Path) -> Result<PathBuf, Box<dyn Error>> {
    // nanosecond suffix makes collisions extremely unlikely in local runs.
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let dir = workspace_root
        .join("target")
        .join("example-fixtures")
        .join(format!("test-{nanos}"));
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn write_fixture(dir: &Path, file_name: &str, raw: &str) -> Result<String, Box<dyn Error>> {
    // string path output keeps call sites simple in this example.
    let path = dir.join(file_name);
    fs::write(&path, raw)?;
    Ok(path.to_string_lossy().to_string())
}

fn build_doc_raw(title: &str, target: &str, words: &[&str]) -> String {
    // helper generates one minimal toml payload accepted by `ExampleLoader`.
    let serialised_words = words
        .iter()
        .map(|word| format!("\"{word}\""))
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        "[header]\ntitle = \"{title}\"\ntarget = \"{target}\"\n\n[values]\nwords = [{serialised_words}]\n"
    )
}
