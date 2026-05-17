//! unified loader for parsing and assembling aethel source documents and corpora.

pub mod error;

use std::path::Path;

use crate::corpus::{Corpus, types::{Document, DocumentMetadata, Field, Section}};

use error::LoaderError;

// ─── public types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// configurable load behaviour applied during corpus assembly.
pub struct LoadOptions {
    /// when `false`, duplicate `header.title` values across source files are rejected.
    pub identical_title_allowed: bool,
    /// when `true` consumes error when a source does not have a matching target and does not add it to [`Corpus`]
    pub skip_source_with_target_mismatch: bool,
}

impl Default for LoadOptions {
    fn default() -> Self {
        Self {
            identical_title_allowed: true,
            skip_source_with_target_mismatch: false,
        }
    }
}

/// optional validation hook applied to each document before it enters the corpus.
pub trait LoadValidator {
    /// validates one parsed document; return `Err` to reject it.
    fn validate(&self, document: &Document) -> Result<(), LoaderError>;
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

    let mut fields: Vec<Field> = Vec::new();
    for (key, val) in table {
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
            skip_source_with_target_mismatch: false
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
