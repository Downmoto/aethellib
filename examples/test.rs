//! manual api walkthrough for the current aethellib surface.

use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use aethellib::generator::Generator;
use aethellib::loader::TargetedLoader;
use aethellib::merger::error::MergerError;
use aethellib::merger::merge_files;
use aethellib::merger::merger_options::MergeOptions;
use aethellib::{AethelCorpus, AethelDoc, AethelDocHeader, SourceAethelDoc, Target};
use rand::Rng;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ExampleLoader {
    values: ExampleValues,
}

impl TargetedLoader for ExampleLoader {
    const TARGET: &'static str = "example";
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ExampleValues {
    words: Vec<String>,
}

struct ExampleGenerator {
    words: Vec<String>,
}

impl Generator for ExampleGenerator {
    type Loader = ExampleLoader;
    type Output = String;

    fn new(corpus: AethelCorpus<Self::Loader>) -> Self {
        let words = corpus
            .documents
            .iter()
            .flat_map(|doc| doc.data.values.words.iter().cloned())
            .collect();

        Self { words }
    }

    fn generate_with_rng<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::Output {
        let index = rng.gen_range(0..self.words.len());
        self.words[index].clone()
    }
}

#[allow(deprecated)]
fn merge_with_legacy_alias(paths: &[&str]) -> Result<AethelCorpus<ExampleLoader>, MergerError> {
    aethellib::merger::merge_target_files::<ExampleLoader>(paths, None)
}

fn main() -> Result<(), Box<dyn Error>> {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_dir = make_fixture_dir(&workspace_root)?;

    let raw_alpha = build_doc_raw("alpha set", "example", &["oak", "ash", "elm"]);
    let raw_beta = build_doc_raw("beta set", "example", &["birch", "cedar"]);
    let raw_person = build_doc_raw("other set", "person", &["wrong"]);
    let raw_dup_one = build_doc_raw("duplicate", "example", &["left"]);
    let raw_dup_two = build_doc_raw("duplicate", "example", &["right"]);

    let alpha_path = write_fixture(&fixture_dir, "alpha.toml", &raw_alpha)?;
    let beta_path = write_fixture(&fixture_dir, "beta.toml", &raw_beta)?;
    let person_path = write_fixture(&fixture_dir, "person.toml", &raw_person)?;
    let dup_one_path = write_fixture(&fixture_dir, "dup_one.toml", &raw_dup_one)?;
    let dup_two_path = write_fixture(&fixture_dir, "dup_two.toml", &raw_dup_two)?;

    let parsed_inline = ExampleLoader::from_str("inline-alpha", &raw_alpha)?;
    assert_eq!(parsed_inline.header.target, ExampleLoader::TARGET);
    assert_eq!(parsed_inline.header.title, "alpha set");

    let parsed_file = ExampleLoader::from_file(&alpha_path)?;
    assert_eq!(parsed_file.header.title, "alpha set");

    let merge_paths = vec![alpha_path.as_str(), beta_path.as_str()];
    let corpus = merge_files::<ExampleLoader>(&merge_paths, None)?;
    assert_eq!(corpus.target(), "example");
    assert_eq!(corpus.documents.len(), 2);

    let corpus_legacy = merge_with_legacy_alias(&merge_paths)?;
    assert_eq!(corpus_legacy.documents.len(), 2);

    let strict_opts = MergeOptions {
        identical_names_allowed: false,
    };
    let dup_paths = vec![dup_one_path.as_str(), dup_two_path.as_str()];
    let dup_result = merge_files::<ExampleLoader>(&dup_paths, Some(strict_opts));
    assert!(dup_result.is_err());

    let mismatch = ExampleLoader::from_file(&person_path);
    assert!(mismatch.is_err());

    let target_value: Target = ExampleLoader::TARGET.to_string();
    assert_eq!(target_value, "example");

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

    let source_single = SourceAethelDoc::from_aetheldoc(manual_doc.clone())?;
    let source_many = SourceAethelDoc::from_aetheldocs(vec![
        manual_doc,
        ExampleLoader::from_str(
        "inline-beta.toml",
        &raw_beta,
    )?])?;
    assert_eq!(source_many.len(), 2);

    let source_docs = vec![source_single.clone(), source_many[1].clone()];
    let corpus_from_sources = AethelCorpus {
        target: ExampleLoader::TARGET.to_string(),
        documents: source_docs,
    };
    assert_eq!(corpus_from_sources.target(), "example");

    let from_new = ExampleGenerator::new(corpus_from_sources.clone());
    let mut seeded_rng = rand::rngs::StdRng::seed_from_u64(7);
    let seeded_value = from_new.generate_with_rng(&mut seeded_rng);
    assert!(!seeded_value.is_empty());

    let from_file_generator = ExampleGenerator::from_file(alpha_path.as_str())?;
    assert!(!from_file_generator.generate().is_empty());

    let from_files_generator = ExampleGenerator::from_files(&merge_paths)?;
    assert!(!from_files_generator.generate().is_empty());

    let from_documents_generator = ExampleGenerator::from_documents(vec![
        source_single,
        source_many.into_iter().next().ok_or("missing source doc")?,
    ]);
    assert!(!from_documents_generator.generate().is_empty());

    println!("example api walkthrough passed");
    Ok(())
}

fn make_fixture_dir(workspace_root: &PathBuf) -> Result<PathBuf, Box<dyn Error>> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let dir = workspace_root
        .join("target")
        .join("example-fixtures")
        .join(format!("test-{nanos}"));
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn write_fixture(dir: &PathBuf, file_name: &str, raw: &str) -> Result<String, Box<dyn Error>> {
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