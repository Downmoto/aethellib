//! api walkthrough for the aethellib 0.6 loader surface.
//!
//! exercises the main public API in a single self-contained executable:
//! - `load_str` for inline TOML parsing
//! - `load_files` for file-based loading (single and multi-file)
//! - `Corpus::builder` for mixed file + in-memory assembly with a validator hook
//! - `Corpus::combine` for merging two corpora
//! - `LoadOptions` for title-uniqueness enforcement
//! - error kind inspection via `LoaderErrorKind`
//!
//! fixtures are created under `target/` and removed automatically at exit.

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use aethellib::loader::error::LoaderErrorKind;
use aethellib::prelude::{
    Corpus, Document, LoadOptions, LoadValidator, LoaderError, load_files, load_str,
};

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
        let _ = fs::remove_dir_all(self.path.as_path());
    }
}

struct TitleContainsValidator {
    needle: &'static str,
}

impl LoadValidator for TitleContainsValidator {
    fn validate(&self, document: &Document) -> Result<(), LoaderError> {
        if document.metadata.title.contains(self.needle) {
            Ok(())
        } else {
            Err(LoaderError::InvalidInput(format!(
                "validator rejected '{}': title must contain '{}'",
                document.source_path, self.needle
            )))
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_dir = FixtureDirGuard::new(make_fixture_dir(workspace_root.as_path())?);

    let raw_alpha = build_doc_raw("alpha set", "example", &["oak", "ash", "elm"]);
    let raw_beta = build_doc_raw("beta set", "example", &["birch", "cedar"]);
    let raw_person = build_doc_raw("other set", "person", &["wrong"]);
    let raw_dup = build_doc_raw("duplicate", "example", &["left"]);

    let alpha_path = write_fixture(fixture_dir.path(), "alpha.toml", &raw_alpha)?;
    let beta_path = write_fixture(fixture_dir.path(), "beta.toml", &raw_beta)?;
    let person_path = write_fixture(fixture_dir.path(), "person.toml", &raw_person)?;
    let dup_one_path = write_fixture(fixture_dir.path(), "dup_one.toml", &raw_dup)?;
    let dup_two_path = write_fixture(fixture_dir.path(), "dup_two.toml", &raw_dup)?;

    // inline parsing via load_str
    let doc = load_str("inline-alpha", &raw_alpha, "example")?;
    assert_eq!(doc.metadata.target, "example");
    assert_eq!(doc.metadata.title, "alpha set");
    assert_eq!(doc.source_path, "inline-alpha");
    assert!(!doc.source_hash.is_empty());
    assert_eq!(doc.source_id, doc.source_hash);

    // section and field access
    assert_eq!(doc.sections.len(), 1);
    let section = &doc.sections[0];
    assert_eq!(section.title, "values");
    assert!(!section.fields.is_empty());
    assert!(doc.section("values").is_some());
    assert!(doc.section("nonexistent").is_none());

    // target mismatch from load_str
    let mismatch = load_str("inline-person", &raw_person, "example");
    assert!(mismatch.is_err());
    assert_eq!(
        mismatch.unwrap_err().kind(),
        LoaderErrorKind::TargetMismatch
    );

    // single-file load via load_files (one-element slice)
    let single_corpus = load_files(&[alpha_path.as_str()], "example", None)?;
    assert_eq!(single_corpus.target(), "example");
    assert_eq!(single_corpus.documents.len(), 1);

    // multi-file load
    let merge_paths = vec![alpha_path.as_str(), beta_path.as_str()];
    let corpus = load_files(&merge_paths, "example", None)?;
    assert_eq!(corpus.target(), "example");
    assert_eq!(corpus.documents.len(), 2);
    assert_eq!(corpus.source_ids().len(), 2);
    assert_eq!(corpus.source_paths().len(), 2);
    assert!(corpus.find_source(corpus.source_ids()[0]).is_some());

    // target mismatch propagates through load_files
    let mismatch_files = load_files(&[person_path.as_str()], "example", None);
    assert!(mismatch_files.is_err());
    assert_eq!(
        mismatch_files.unwrap_err().kind(),
        LoaderErrorKind::TargetMismatch
    );

    // duplicate source hashes get unique ids
    let dup_paths = vec![dup_one_path.as_str(), dup_two_path.as_str()];
    let dup_corpus = load_files(&dup_paths, "example", None)?;
    assert_eq!(dup_corpus.documents[0].source_id, dup_corpus.documents[0].source_hash);
    assert!(dup_corpus.documents[1].source_id.contains(':'));

    // LoadOptions: reject identical titles
    let strict_opts = LoadOptions {
        identical_title_allowed: false,
    };
    let strict_result = load_files(&dup_paths, "example", Some(strict_opts));
    assert!(strict_result.is_err());
    assert_eq!(
        strict_result.unwrap_err().kind(),
        LoaderErrorKind::OptionViolation
    );

    // CorpusBuilder: mixed file + in-memory sources
    let builder_corpus = Corpus::builder("example")
        .add_file(alpha_path.as_str())
        .add_str("memory-beta", &raw_beta)
        .build()?;
    assert_eq!(builder_corpus.documents.len(), 2);

    // CorpusBuilder: with validator
    let set_validator = TitleContainsValidator { needle: "set" };
    let validated_corpus = Corpus::builder("example")
        .with_validator(set_validator)
        .add_file(alpha_path.as_str())
        .add_file(beta_path.as_str())
        .build()?;
    assert_eq!(validated_corpus.documents.len(), 2);

    let reject_validator = TitleContainsValidator { needle: "alpha" };
    let rejected = Corpus::builder("example")
        .with_validator(reject_validator)
        .add_file(alpha_path.as_str())
        .add_file(beta_path.as_str())
        .build();
    assert!(rejected.is_err());

    // CorpusBuilder: empty sources is an error
    let empty_result = Corpus::builder("example").build();
    assert!(empty_result.is_err());
    assert_eq!(empty_result.unwrap_err().kind(), LoaderErrorKind::InvalidInput);

    // Corpus::combine merges two corpora
    let left = load_files(&[alpha_path.as_str()], "example", None)?;
    let right = load_files(&[beta_path.as_str()], "example", None)?;
    let combined = left.combine(right);
    assert_eq!(combined.target(), "example");
    assert_eq!(combined.documents.len(), 2);

    println!("api walkthrough passed");
    Ok(())
}

fn make_fixture_dir(workspace_root: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let dir = workspace_root
        .join("target")
        .join("example-fixtures")
        .join(format!("test-{nanos}"));
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn write_fixture(dir: &Path, file_name: &str, raw: &str) -> Result<String, Box<dyn Error>> {
    let path = dir.join(file_name);
    fs::write(&path, raw)?;
    Ok(path.to_string_lossy().to_string())
}

fn build_doc_raw(title: &str, target: &str, words: &[&str]) -> String {
    let serialised_words = words
        .iter()
        .map(|word| format!("\"{word}\""))
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        "[header]\ntitle = \"{title}\"\ntarget = \"{target}\"\n\n[values]\nwords = [{serialised_words}]\n"
    )
}
