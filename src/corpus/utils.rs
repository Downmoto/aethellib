//! private loader utilities for hashing and source id assignment.

use std::collections::{BTreeMap, HashMap};

use sha2::{Digest, Sha256};

use crate::corpus::{
    PooledValue, ValuePool, ValueProvenance,
    error::CorpusLoaderError,
    types::{Document, DocumentMetadata, Field, Section},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// configurable load behaviour applied during corpus assembly.
pub struct CorpusLoaderOptions {
    /// when `false`, duplicate `header.title` values across source files are rejected.
    pub identical_title_allowed: bool,
    /// When `true`, sources whose `[header].target` does not match are skipped instead 
    /// of returning [`CorpusLoaderError::TargetMismatch`].
    pub skip_source_with_target_mismatch: bool,
}

impl Default for CorpusLoaderOptions {
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
    fn validate(&self, document: &Document) -> Result<(), CorpusLoaderError>;
}

impl LoadValidator for Box<dyn LoadValidator> {
    fn validate(&self, document: &Document) -> Result<(), CorpusLoaderError> {
        (**self).validate(document)
    }
}

/// parses raw TOML into `(DocumentMetadata, Vec<Section>)` without target validation.
pub(crate) fn parse_document(
    name: &str,
    raw: &str,
) -> Result<(DocumentMetadata, Vec<Section>), CorpusLoaderError> {
    let table: toml::Table = raw
        .parse()
        .map_err(|e| CorpusLoaderError::parse_for_path(name, e))?;

    let header_val = table.get("header").ok_or_else(|| {
        CorpusLoaderError::InvalidInput(format!("'{name}' is missing a [header] table"))
    })?;

    let metadata: DocumentMetadata = header_val
        .clone()
        .try_into()
        .map_err(|e| CorpusLoaderError::parse_for_path(name, e))?;

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
fn parse_section(
    name: &str,
    title: &str,
    value: &toml::Value,
) -> Result<Section, CorpusLoaderError> {
    let table = match value {
        toml::Value::Table(t) => t,
        _ => {
            return Err(CorpusLoaderError::InvalidInput(format!(
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

/// hashes canonicalized source content with target context for stable identity.
pub(crate) fn hash_source_content(target: &str, raw: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("{target}\n"));
    hasher.update(canonicalize_raw(raw));
    format!("{:x}", hasher.finalize())
}

/// creates a unique source id from a base hash within one corpus build.
pub(crate) fn make_unique_source_id(base_hash: &str, seen: &mut HashMap<String, usize>) -> String {
    let count = seen.entry(base_hash.to_string()).or_insert(0);
    *count += 1;

    if *count == 1 {
        base_hash.to_string()
    } else {
        format!("{base_hash}:{count}")
    }
}

/// normalizes source text before hashing to reduce platform-specific diffs.
fn canonicalize_raw(raw: &str) -> String {
    raw.replace("\r\n", "\n")
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_stable_across_line_ending_variants() {
        let unix = "person\nfoo = \"bar\"";
        let windows = "person\r\nfoo = \"bar\"";
        assert_eq!(
            hash_source_content("person", unix),
            hash_source_content("person", windows)
        );
    }

    #[test]
    fn make_unique_source_id_suffixes_duplicates() {
        let mut seen = HashMap::new();
        let id1 = make_unique_source_id("abc", &mut seen);
        let id2 = make_unique_source_id("abc", &mut seen);
        let id3 = make_unique_source_id("abc", &mut seen);
        let id4 = make_unique_source_id("xyz", &mut seen);
        assert_eq!(id1, "abc");
        assert_eq!(id2, "abc:2");
        assert_eq!(id3, "abc:3");
        assert_eq!(id4, "xyz");
    }
}

type ValuePoolGroup = (Vec<PooledValue>, HashMap<String, usize>);

/// builds pools keyed by exact section and field pairs, merging provenance for duplicate values.
pub(crate) fn build_value_pools(documents: &[Document]) -> Vec<ValuePool> {
    // outer key: (section, field); inner key: value string → index in Vec<PooledValue>
    let mut grouped: BTreeMap<(String, String), ValuePoolGroup> = BTreeMap::new();

    for document in documents {
        for section in &document.sections {
            for field in &section.fields {
                let (pooled_values, index) = grouped
                    .entry((section.title.clone(), field.title.clone()))
                    .or_default();

                let prov = ValueProvenance {
                    source_id: document.source_id.clone(),
                    document_title: document.metadata.title.clone(),
                    section: section.title.clone(),
                    field: field.title.clone(),
                };

                for value in &field.values {
                    if let Some(&existing) = index.get(value.as_str()) {
                        // same value string already present — append provenance
                        pooled_values[existing].provenance.push(prov.clone());
                    } else {
                        index.insert(value.clone(), pooled_values.len());
                        pooled_values.push(PooledValue {
                            value: value.clone(),
                            provenance: vec![prov.clone()],
                        });
                    }
                }
            }
        }
    }

    grouped
        .into_iter()
        .map(|((section, field), (values, _))| ValuePool {
            section,
            field,
            values,
        })
        .collect()
}
