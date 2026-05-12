//! aethellib provides loaders, merge helpers, and generators for aethel datasets.

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
