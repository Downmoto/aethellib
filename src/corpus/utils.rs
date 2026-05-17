//! private loader utilities for hashing and source id assignment.

use std::collections::{BTreeMap, HashMap};

use sha2::{Digest, Sha256};

use crate::{Document, corpus::{PooledValue, ValuePool, ValueProvenance}};

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

/// builds pools keyed by exact section and field pairs, merging provenance for duplicate values.
pub(crate) fn build_value_pools(documents: &[Document]) -> Vec<ValuePool> {
    // outer key: (section, field); inner key: value string → index in Vec<PooledValue>
    let mut grouped: BTreeMap<(String, String), (Vec<PooledValue>, HashMap<String, usize>)> =
        BTreeMap::new();

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
