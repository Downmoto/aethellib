//! aethellib provides loaders, merge helpers, and generators for aethel datasets.

use serde::{Deserialize, Serialize};

use crate::{
    loader::{TargetedLoader, error::LoaderError},
    merger::{Mixed, error::MergerError, utils::cast_aethel_docs_to_sources},
};

/// generation module entrypoint.
pub mod generator;
/// loader module entrypoint.
pub mod loader;
/// merge module entrypoint.
pub mod merger;

#[cfg(test)]
/// shared test helpers for inline fixtures and temp files.
pub(crate) mod test_support;

/// open target identifier used by loaders, mergers, and generators.
pub type Target = String;

#[derive(Deserialize, Serialize, Debug, Clone)]
/// common metadata required in each input file header.
pub struct AethelDocHeader {
    /// dataset display name.
    pub name: String,
    /// target category used for loader validation.
    pub target: Target,
    /// optional dataset description.
    pub desc: Option<String>,
    /// optional dataset author.
    pub author: Option<String>,
    /// optional dataset version.
    pub version: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
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
}

impl AethelCorpus<Mixed> {
    /// consumes this value and converts source tables into a typed corpus.
    pub fn into_corpus<T>(self) -> Result<AethelCorpus<T>, MergerError>
    where
        T: TargetedLoader,
    {
        if self.target != T::TARGET {
            return Err(LoaderError::TargetMismatch {
                expected: T::TARGET.to_string(),
                found: self.target,
            }
            .into());
        }

        let mut documents: Vec<SourceAethelDoc<T>> = Vec::with_capacity(self.documents.len());

        for source in self.documents {
            let data: T = toml::Value::Table(source.data)
                .try_into()
                .map_err(|err| LoaderError::parse_for_path(source.source_path.as_str(), err))?;

            documents.push(SourceAethelDoc {
                source_id: source.source_id,
                source_hash: source.source_hash,
                source_path: source.source_path,
                header: source.header,
                data,
            });
        }

        Ok(AethelCorpus {
            target: T::TARGET.to_string(),
            documents,
        })
    }

    /// clones and converts source tables into a typed corpus.
    pub fn to_corpus<T>(&self) -> Result<AethelCorpus<T>, MergerError>
    where
        T: TargetedLoader,
    {
        self.clone().into_corpus::<T>()
    }
}
