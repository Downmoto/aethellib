//! unified loader for parsing and assembling aethel source documents and corpora.

pub mod error;
pub(crate) mod utils;

use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use crate::{Corpus, Document, DocumentMetadata, Field, Rule, Section};

use error::LoaderError;

// ─── public types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// configurable load behaviour applied during corpus assembly.
pub struct LoadOptions {
    /// when `false`, duplicate `header.title` values across source files are rejected.
    pub identical_title_allowed: bool,
}

impl Default for LoadOptions {
    fn default() -> Self {
        Self {
            identical_title_allowed: true,
        }
    }
}

/// optional validation hook applied to each document before it enters the corpus.
pub trait LoadValidator {
    /// validates one parsed document; return `Err` to reject it.
    fn validate(&self, document: &Document) -> Result<(), LoaderError>;
}

// ─── corpus builder ───────────────────────────────────────────────────────────

enum PendingSource {
    File(PathBuf),
    Str { name: String, raw: String },
}

/// incremental builder for assembling a `Corpus` from any mix of files and in-memory strings.
///
/// obtain one via [`Corpus::builder`].
pub struct CorpusBuilder {
    target: String,
    opts: LoadOptions,
    validator: Option<Box<dyn LoadValidator>>,
    pending: Vec<PendingSource>,
}

impl CorpusBuilder {
    pub(crate) fn new(target: String) -> Self {
        Self {
            target,
            opts: LoadOptions::default(),
            validator: None,
            pending: Vec::new(),
        }
    }

    /// queues one file path to be loaded when [`build`](Self::build) is called.
    pub fn add_file(mut self, path: impl AsRef<Path>) -> Self {
        self.pending
            .push(PendingSource::File(path.as_ref().to_path_buf()));
        self
    }

    /// queues one in-memory raw TOML string to be parsed when [`build`](Self::build) is called.
    pub fn add_str(mut self, name: impl Into<String>, raw: impl Into<String>) -> Self {
        self.pending.push(PendingSource::Str {
            name: name.into(),
            raw: raw.into(),
        });
        self
    }

    /// overrides the default load options.
    pub fn with_options(mut self, opts: LoadOptions) -> Self {
        self.opts = opts;
        self
    }

    /// attaches a custom validation hook applied to each document before it is accepted.
    pub fn with_validator(mut self, validator: impl LoadValidator + 'static) -> Self {
        self.validator = Some(Box::new(validator));
        self
    }

    /// resolves all queued sources and assembles them into a [`Corpus`].
    pub fn build(self) -> Result<Corpus, LoaderError> {
        if self.pending.is_empty() {
            return Err(LoaderError::InvalidInput(
                "at least one source is required to build a corpus".to_string(),
            ));
        }

        let mut seen_source_ids: HashMap<String, usize> = HashMap::new();
        let mut seen_titles: HashSet<String> = HashSet::new();
        let mut documents: Vec<Document> = Vec::with_capacity(self.pending.len());

        for pending in &self.pending {
            let (source_path, raw) = match pending {
                PendingSource::File(path) => {
                    let raw = fs::read_to_string(path)
                        .map_err(|e| LoaderError::read_for_path(path, e))?;
                    (path.to_string_lossy().to_string(), raw)
                }
                PendingSource::Str { name, raw } => (name.clone(), raw.clone()),
            };

            let (metadata, sections) = parse_document(&source_path, &raw)?;

            if metadata.target != self.target {
                return Err(LoaderError::TargetMismatch {
                    expected: self.target.clone(),
                    found: metadata.target,
                });
            }

            if !self.opts.identical_title_allowed && !seen_titles.insert(metadata.title.clone()) {
                return Err(LoaderError::OptionViolation(format!(
                    "duplicate header.title '{}' is not allowed when identical_title_allowed is false",
                    metadata.title
                )));
            }

            let source_hash = utils::hash_source_content(&self.target, &raw);
            let source_id = utils::make_unique_source_id(&source_hash, &mut seen_source_ids);

            let doc = Document {
                metadata,
                sections,
                source_id,
                source_hash,
                source_path,
            };

            if let Some(validator) = &self.validator {
                validator.validate(&doc)?;
            }

            documents.push(doc);
        }

        Ok(Corpus {
            target: self.target,
            documents,
        })
    }
}

// ─── convenience entrypoints ─────────────────────────────────────────────────

/// loads one or more TOML files for `target` and assembles them into a [`Corpus`].
///
/// a single-file load is just a one-element slice.
pub fn load_files(
    paths: &[impl AsRef<Path>],
    target: &str,
    opts: Option<LoadOptions>,
) -> Result<Corpus, LoaderError> {
    let mut builder = Corpus::builder(target);
    if let Some(opts) = opts {
        builder = builder.with_options(opts);
    }
    for path in paths {
        builder = builder.add_file(path);
    }
    builder.build()
}

/// loads one or more TOML files for `target` using a custom validation hook.
pub fn load_files_with_validator(
    paths: &[impl AsRef<Path>],
    target: &str,
    opts: Option<LoadOptions>,
    validator: impl LoadValidator + 'static,
) -> Result<Corpus, LoaderError> {
    let mut builder = Corpus::builder(target).with_validator(validator);
    if let Some(opts) = opts {
        builder = builder.with_options(opts);
    }
    for path in paths {
        builder = builder.add_file(path);
    }
    builder.build()
}

/// parses a single TOML string into a [`Document`], validating that `target` matches.
pub fn load_str(name: &str, raw: &str, target: &str) -> Result<Document, LoaderError> {
    let (metadata, sections) = parse_document(name, raw)?;

    if metadata.target != target {
        return Err(LoaderError::TargetMismatch {
            expected: target.to_string(),
            found: metadata.target,
        });
    }

    let source_hash = utils::hash_source_content(target, raw);

    Ok(Document {
        metadata,
        sections,
        source_id: source_hash.clone(),
        source_hash,
        source_path: name.to_string(),
    })
}

// ─── private document parsing ─────────────────────────────────────────────────

/// parses raw TOML into `(DocumentMetadata, Vec<Section>)` without target validation.
pub(crate) fn parse_document(name: &str, raw: &str) -> Result<(DocumentMetadata, Vec<Section>), LoaderError> {
    let table: toml::Table = raw
        .parse()
        .map_err(|e| LoaderError::parse_for_path(name, e))?;

    let header_val = table.get("header").ok_or_else(|| {
        LoaderError::InvalidInput(format!("'{name}' is missing a [header] table"))
    })?;

    let metadata: DocumentMetadata = header_val
        .clone()
        .try_into()
        .map_err(|e| LoaderError::parse_for_path(name, e))?;

    let mut sections: Vec<Section> = Vec::new();
    for (key, value) in &table {
        if key == "header" {
            continue;
        }
        sections.push(parse_section(name, key, value)?);
    }

    Ok((metadata, sections))
}

/// parses one TOML table value into a [`Section`].
fn parse_section(name: &str, title: &str, value: &toml::Value) -> Result<Section, LoaderError> {
    let table = match value {
        toml::Value::Table(t) => t,
        _ => {
            return Err(LoaderError::InvalidInput(format!(
                "'{name}': section '{title}' must be a TOML table"
            )));
        }
    };

    let outputs: Vec<String> = match table.get("outputs") {
        Some(v) => v
            .clone()
            .try_into()
            .map_err(|e| LoaderError::parse_for_path(name, e))?,
        None => Vec::new(),
    };

    let rules: Vec<Rule> = match table.get("rules") {
        Some(v) => v
            .clone()
            .try_into()
            .map_err(|e| LoaderError::parse_for_path(name, e))?,
        None => Vec::new(),
    };

    const RESERVED: &[&str] = &["outputs", "rules"];
    let mut fields: Vec<Field> = Vec::new();
    for (key, val) in table {
        if RESERVED.contains(&key.as_str()) {
            continue;
        }
        if let toml::Value::Array(arr) = val {
            let strings: Vec<String> = arr
                .iter()
                .filter_map(|v| {
                    if let toml::Value::String(s) = v {
                        Some(s.clone())
                    } else {
                        None
                    }
                })
                .collect();
            fields.push(Field {
                title: key.clone(),
                values: strings,
            });
        }
    }

    Ok(Section {
        title: title.to_string(),
        outputs,
        rules,
        fields,
    })
}

// ─── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const PERSON_TOML: &str = r#"
[header]
title = "person test set"
target = "person"
desc = "minimal person name primitives"
author = "aethellib"
version = "0.1"
schema = "01"

[name]
outputs = ["name"]
rules = [
    { for = "name", rule = "scramble", sample_count = [1, 3], from = ["first", "middle", "last"] },
]

first = ["al", "ra", "jo", "el"]
middle = ["an", "is", "or"]
last = ["ton", "ric", "den", "vale"]
"#;

    #[test]
    fn load_str_parses_person_toml_correctly() {
        let doc = load_str("test", PERSON_TOML, "person").expect("should parse");

        assert_eq!(doc.metadata.title, "person test set");
        assert_eq!(doc.metadata.target, "person");
        assert_eq!(doc.metadata.schema.as_deref(), Some("01"));
        assert_eq!(doc.sections.len(), 1);

        let section = &doc.sections[0];
        assert_eq!(section.title, "name");
        assert_eq!(section.outputs, vec!["name"]);
        assert_eq!(section.rules.len(), 1);

        let rule = &section.rules[0];
        assert_eq!(rule.for_field, "name");
        assert_eq!(rule.rule, "scramble");
        assert!(rule.params.contains_key("sample_count"));
        assert!(rule.params.contains_key("from"));

        assert_eq!(section.fields.len(), 3);
        let field_titles: Vec<&str> = section.fields.iter().map(|f| f.title.as_str()).collect();
        assert!(field_titles.contains(&"first"));
        assert!(field_titles.contains(&"middle"));
        assert!(field_titles.contains(&"last"));

        let first = section.fields.iter().find(|f| f.title == "first").unwrap();
        assert_eq!(first.values, vec!["al", "ra", "jo", "el"]);
    }

    #[test]
    fn load_str_rejects_target_mismatch() {
        let err = load_str("test", PERSON_TOML, "weapon").expect_err("should fail");
        assert_eq!(err.kind(), error::LoaderErrorKind::TargetMismatch);
    }

    #[test]
    fn load_str_errors_on_missing_header() {
        let raw = "[name]\nfirst = [\"a\"]\n";
        let err = load_str("test", raw, "person").expect_err("should fail");
        assert_eq!(err.kind(), error::LoaderErrorKind::InvalidInput);
    }

    #[test]
    fn corpus_builder_assembles_multiple_str_sources() {
        let corpus = Corpus::builder("person")
            .add_str("first", PERSON_TOML)
            .add_str("second", PERSON_TOML)
            .build()
            .expect("should build");

        assert_eq!(corpus.target(), "person");
        assert_eq!(corpus.documents.len(), 2);
        assert_eq!(corpus.documents[0].source_id, corpus.documents[0].source_hash);
        assert!(corpus.documents[1].source_id.contains(':'));
    }

    #[test]
    fn corpus_builder_rejects_empty_sources() {
        let err = Corpus::builder("person").build().expect_err("should fail");
        assert_eq!(err.kind(), error::LoaderErrorKind::InvalidInput);
    }

    #[test]
    fn load_options_identical_title_rejected_when_disabled() {
        let opts = LoadOptions {
            identical_title_allowed: false,
        };
        let err = Corpus::builder("person")
            .with_options(opts)
            .add_str("a", PERSON_TOML)
            .add_str("b", PERSON_TOML)
            .build()
            .expect_err("should fail");
        assert_eq!(err.kind(), error::LoaderErrorKind::OptionViolation);
    }

    #[test]
    fn load_validator_can_reject_documents() {
        struct RejectAll;
        impl LoadValidator for RejectAll {
            fn validate(&self, _doc: &Document) -> Result<(), LoaderError> {
                Err(LoaderError::InvalidInput("rejected".to_string()))
            }
        }

        let err = Corpus::builder("person")
            .with_validator(RejectAll)
            .add_str("a", PERSON_TOML)
            .build()
            .expect_err("should fail");
        assert_eq!(err.kind(), error::LoaderErrorKind::InvalidInput);
    }
}
