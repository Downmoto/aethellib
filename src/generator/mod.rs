//! generation logic for aethel content types.
//!
//! this module exposes concrete generators that build runtime content
//! from loaded and validated aethel documents.
//! generators should expose a convenience `generate()` method and
//! a deterministic `generate_with_rng(...)` method for reproducible tests.

use std::path::Path;

use rand::Rng;
use rand::thread_rng;

use crate::AethelCorpus;
use crate::SourceAethelDoc;
use crate::loader::TargetedLoader;
use crate::merger::error::MergerError;
use crate::merger::merge_files;

/// generic generator contract with shared constructor and generation helpers.
pub trait Generator: Sized {
    /// loader payload type used by this generator target.
    type Loader: TargetedLoader;
    /// generated output type.
    type Output;

    /// creates a generator from a merged target corpus.
    fn new(corpus: AethelCorpus<Self::Loader>) -> Self;

    /// builds one output value using the supplied rng.
    fn generate_with_rng<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::Output;

    /// creates a generator directly from source documents.
    fn from_documents(documents: Vec<SourceAethelDoc<Self::Loader>>) -> Self {
        Self::new(AethelCorpus {
            target: <Self::Loader as TargetedLoader>::TARGET.to_string(),
            documents,
        })
    }

    /// loads one target file and creates a corpus-backed generator.
    fn from_file(path: impl AsRef<Path>) -> Result<Self, MergerError> {
        let paths = [path];
        let corpus = merge_files::<Self::Loader>(&paths, None)?;
        Ok(Self::new(corpus))
    }

    /// loads target files and creates a corpus-backed generator.
    fn from_files(paths: &[impl AsRef<Path>]) -> Result<Self, MergerError> {
        let corpus = merge_files::<Self::Loader>(paths, None)?;
        Ok(Self::new(corpus))
    }

    /// builds one output by sampling with thread-local randomness.
    fn generate(&self) -> Self::Output {
        let mut rng = thread_rng();
        self.generate_with_rng(&mut rng)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// provenance reference for one source field.
pub struct SourceRef {
    /// source id from merged corpus.
    pub source_id: String,
    /// source display name from document header.
    pub source_name: String,
    /// top-level section name in source payload.
    pub section: String,
    /// field name inside section.
    pub field: String,
}

impl SourceRef {
    /// returns true when this reference points to the given section and field.
    pub fn matches(&self, section: &str, field: &str) -> bool {
        self.section == section && self.field == field
    }
}

#[derive(Debug, Clone)]
/// generated field value with aggregated provenance references.
pub struct GeneratedField<T> {
    /// selected output value.
    pub value: T,
    /// all source refs that can yield this value.
    pub source_refs: Vec<SourceRef>,
}

impl<T> GeneratedField<T> {
    /// returns true when provenance contains the given source id.
    pub fn has_source_id(&self, source_id: &str) -> bool {
        self.source_refs
            .iter()
            .any(|source_ref| source_ref.source_id == source_id)
    }

    /// returns distinct source ids in first-seen order.
    pub fn source_ids(&self) -> Vec<&str> {
        let mut ids: Vec<&str> = Vec::new();

        for source_ref in &self.source_refs {
            let id = source_ref.source_id.as_str();
            if !ids.contains(&id) {
                ids.push(id);
            }
        }

        ids
    }

    /// resolves source ids in this generated field into source paths in corpus order.
    pub fn source_paths_in<'a, U>(&self, corpus: &'a AethelCorpus<U>) -> Vec<&'a str> {
        let source_ids = self.source_ids();
        let mut source_paths: Vec<&'a str> = Vec::new();

        for source_id in source_ids {
            if let Some(source_document) = corpus.find_source(source_id) {
                source_paths.push(source_document.source_path.as_str());
            }
        }

        source_paths
    }
}
