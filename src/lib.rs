//! aethellib provides loaders, merge helpers, and generators for aethel datasets.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    loader::TargetedLoader,
    merger::{error::MergerError, utils::cast_aethel_docs_to_sources},
};

/// generation module entrypoint.
pub mod generator;
/// loader module entrypoint.
pub mod loader;
/// merge module entrypoint.
pub mod merger;
/// prelude module for common imports.
pub mod prelude;

/// open target identifier used by loaders, mergers, and generators.
pub type Target = String;

#[derive(Deserialize, Serialize, Debug, Clone)]
/// common metadata required in each input file header.
pub struct AethelDocHeader {
    /// dataset display name.
    pub title: String,
    /// target category used for loader validation.
    pub target: Target,
    /// optional dataset description.
    pub desc: Option<String>,
    /// optional dataset author.
    pub author: Option<String>,
    /// optional dataset version.
    pub version: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
/// parsed toml payload with header plus target-specific body data.
pub struct AethelDoc<T> {
    /// parsed file header.
    pub header: AethelDocHeader,
    /// target-specific sections flattened from the same root document.
    #[serde(flatten)]
    pub data: T,
}

#[derive(Debug, Clone)]
/// one source document retained in a target corpus.
pub struct SourceAethelDoc<T> {
    /// unique id for this source within a corpus instance.
    pub source_id: String,
    /// deterministic hash derived from canonicalized document content and target.
    pub source_hash: String,
    /// original source path used for loading.
    pub source_path: String,
    /// metadata from the source header.
    pub header: AethelDocHeader,
    /// source data body.
    pub data: T,
}

impl<T> SourceAethelDoc<T>
where
    T: TargetedLoader + serde::Serialize,
{
    /// casts one parsed aethel document into one source document.
    pub fn from_aetheldoc(document: AethelDoc<T>) -> Result<Self, MergerError> {
        let mut source_documents = cast_aethel_docs_to_sources::<T>(vec![document])?;
        Ok(source_documents.remove(0))
    }

    /// casts parsed aethel documents into source documents using merge hash/id rules.
    pub fn from_aetheldocs(documents: Vec<AethelDoc<T>>) -> Result<Vec<Self>, MergerError> {
        cast_aethel_docs_to_sources::<T>(documents)
    }
}

#[derive(Debug, Clone)]
/// per-target corpus retaining all source documents and metadata.
pub struct AethelCorpus<T> {
    /// target represented by all source documents.
    pub target: Target,
    /// source documents in first-seen order.
    pub documents: Vec<SourceAethelDoc<T>>,
}

impl<T> AethelCorpus<T> {
    /// returns the target represented by this corpus.
    pub fn target(&self) -> &str {
        self.target.as_str()
    }

    pub fn combine(self, other: AethelCorpus<T>) -> Self {
        let AethelCorpus {
            target,
            mut documents,
        } = self;
        let AethelCorpus {
            target: other_target,
            documents: other_documents,
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

        Self { target, documents }
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
    pub fn find_source(&self, source_id: &str) -> Option<&SourceAethelDoc<T>> {
        self.documents
            .iter()
            .find(|document| document.source_id == source_id)
    }
}

#[cfg(test)]
mod tests {
    use super::{AethelCorpus, AethelDocHeader, SourceAethelDoc};

    fn source(hash: &str, id: &str, target: &str) -> SourceAethelDoc<()> {
        SourceAethelDoc {
            source_id: id.to_string(),
            source_hash: hash.to_string(),
            source_path: format!("/{id}.toml"),
            header: AethelDocHeader {
                title: id.to_string(),
                target: target.to_string(),
                desc: None,
                author: None,
                version: None,
            },
            data: (),
        }
    }

    #[test]
    fn combine_appends_documents_and_renumbers_duplicate_hash_ids() {
        let left = AethelCorpus {
            target: "weapon".to_string(),
            documents: vec![source("hash-a", "old-1", "weapon")],
        };

        let right = AethelCorpus {
            target: "weapon".to_string(),
            documents: vec![
                source("hash-a", "old-2", "weapon"),
                source("hash-b", "old-3", "weapon"),
            ],
        };

        let combined = left.combine(right);

        assert_eq!(combined.target, "weapon");
        assert_eq!(combined.documents.len(), 3);
        assert_eq!(combined.documents[0].source_id, "hash-a");
        assert_eq!(combined.documents[1].source_id, "hash-a:2");
        assert_eq!(combined.documents[2].source_id, "hash-b");
    }

    #[test]
    #[should_panic(expected = "cannot combine corpora with different targets")]
    fn combine_panics_for_different_targets() {
        let left = AethelCorpus {
            target: "weapon".to_string(),
            documents: vec![source("hash-a", "old-1", "weapon")],
        };

        let right = AethelCorpus {
            target: "person".to_string(),
            documents: vec![source("hash-b", "old-2", "person")],
        };

        let _ = left.combine(right);
    }
}
