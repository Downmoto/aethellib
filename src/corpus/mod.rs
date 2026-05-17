pub(crate) mod utils;

use std::{collections::{HashMap, HashSet}, fs, path::{Path, PathBuf}};

use crate::{Document, Target, corpus::utils::build_value_pools, loader::{LoadOptions, LoadValidator, error::LoaderError, parse_document}};


#[derive(Debug, Clone)]
/// tracks where one pooled value came from.
pub struct ValueProvenance {
    /// unique source identifier inside the corpus.
    pub source_id: String,
    /// source document title from `[header].title`.
    pub document_title: String,
    /// section name that contained this value.
    pub section: String,
    /// field name that contained this value.
    pub field: String,
}

#[derive(Debug, Clone)]
/// one pooled value with provenance metadata.
pub struct PooledValue {
    /// string value extracted from one field entry.
    pub value: String,
    /// all origins that produced this value; merged when the same value appears in multiple sources.
    pub provenance: Vec<ValueProvenance>,
}

#[derive(Debug, Clone)]
/// pooled values for one exact section and field pair.
pub struct ValuePool {
    /// section name for this pool.
    pub section: String,
    /// field name for this pool.
    pub field: String,
    values: Vec<PooledValue>
}

impl ValuePool {
    /// returns all pooled values in this pool.
    pub fn values(&self) -> &[PooledValue] {
        &self.values
    }
}

#[derive(Debug, Clone)]
/// a corpus of source documents for one target.
pub struct Corpus {
    /// target represented by all source documents.
    pub target: Target,
    /// source documents in first-seen order.
    pub documents: Vec<Document>,
    /// pools of values based of section -> field.
    pub pools: Vec<ValuePool>
}

impl Corpus {
    /// returns a new corpus builder for the given target.
    pub fn builder(target: impl Into<String>) -> CorpusBuilder {
        CorpusBuilder::new(target.into())
    }

    /// returns the target represented by this corpus.
    pub fn target(&self) -> &str {
        self.target.as_str()
    }

    /// combines two corpora with the same target into one, reassigning duplicate source ids.
    pub fn combine(self, other: Corpus) -> Self {
        let Corpus {
            target,
            mut documents,
            pools: _
        } = self;
        let Corpus {
            target: other_target,
            documents: other_documents,
            pools: _
        } = other;

        assert_eq!(
            target, other_target,
            "cannot combine corpora with different targets: left='{}', right='{}'",
            target, other_target
        );

        documents.extend(other_documents);

        let mut seen_source_hashes: HashMap<String, usize> = HashMap::new();
        for document in &mut documents {
            let count = seen_source_hashes
                .entry(document.source_hash.clone())
                .or_insert(0);
            *count += 1;

            if *count == 1 {
                document.source_id = document.source_hash.clone();
            } else {
                document.source_id = format!("{}:{}", document.source_hash, count);
            }
        }

        let pools = build_value_pools(&documents);

        Self { target, documents, pools }
    }

    /// returns all source ids in corpus order.
    pub fn source_ids(&self) -> Vec<&str> {
        self.documents
            .iter()
            .map(|document| document.source_id.as_str())
            .collect()
    }

    /// returns all source paths in corpus order.
    pub fn source_paths(&self) -> Vec<&str> {
        self.documents
            .iter()
            .map(|document| document.source_path.as_str())
            .collect()
    }

    /// finds one source document by source id.
    pub fn find_source(&self, source_id: &str) -> Option<&Document> {
        self.documents
            .iter()
            .find(|document| document.source_id == source_id)
    }

    /// returns pooled values for one exact section and field pair.
    pub fn pooled_values_for_field_section(&self, field: &str, section: &str) -> Option<&[PooledValue]> {
        self.pools
            .iter()
            .find(|pool| pool.field == field && pool.section == section)
            .map(ValuePool::values)
    }
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
                if !self.opts.skip_source_with_target_mismatch {
                    return Err(LoaderError::TargetMismatch {
                        expected: self.target.clone(),
                        found: metadata.target,
                    });
                }
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

        let pools = build_value_pools(&documents);

        Ok(Corpus {
            target: self.target,
            documents,
            pools,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Corpus;

    #[test]
    fn pooled_values_lookup_keeps_same_field_names_separate_per_section() {
        let doc1 = r#"
[header]
title = "doc one"
target = "weapon"

[name]
first = ["ash", "birch"]

[aliases]
first = ["ember"]
"#;

        let doc2 = r#"
[header]
title = "doc two"
target = "weapon"

[name]
first = ["cedar"]
"#;

        let corpus = Corpus::builder("weapon")
            .add_str("doc-1", doc1)
            .add_str("doc-2", doc2)
            .build()
            .expect("corpus should build");

        let name_first = corpus
            .pooled_values_for_field_section("first", "name")
            .expect("name.first pool should exist");
        let aliases_first = corpus
            .pooled_values_for_field_section("first", "aliases")
            .expect("aliases.first pool should exist");

        assert_eq!(name_first.len(), 3);
        assert_eq!(aliases_first.len(), 1);
        assert!(
            name_first
                .iter()
                .all(|value| value.provenance.iter().all(|p| p.section == "name"))
        );
        assert!(
            aliases_first
                .iter()
                .all(|value| value.provenance.iter().all(|p| p.section == "aliases"))
        );
    }

    #[test]
    fn pooled_values_lookup_returns_none_for_missing_section_field_pair() {
        let raw = r#"
[header]
title = "doc one"
target = "person"

[name]
first = ["al"]
"#;

        let corpus = Corpus::builder("person")
            .add_str("doc-1", raw)
            .build()
            .expect("corpus should build");

        assert!(
            corpus
                .pooled_values_for_field_section("last", "name")
                .is_none()
        );
        assert!(
            corpus
                .pooled_values_for_field_section("first", "aliases")
                .is_none()
        );
    }
}