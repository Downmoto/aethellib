//! generation logic for aethel content types.
//!
//! this module exposes concrete generators that build runtime content
//! from loaded and validated aethel documents.
//! generators should expose a convenience `generate()` method and
//! a deterministic `generate_with_rng(...)` method for reproducible tests.

#[cfg(feature = "person-gen")]
pub mod generator_person;
#[cfg(feature = "weapon-gen")]
pub mod generator_weapon;
#[cfg(any(feature = "person-gen", feature = "weapon-gen"))]
pub(self) mod utils;

use rand::Rng;
use rand::thread_rng;

use crate::loader::TargetedLoader;
use crate::merger::merge_target_files;
use crate::merger::{AethelCorpus, SourceAethelDoc, error::MergerError};

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
    fn from_file(path: &str) -> Result<Self, MergerError> {
        let corpus = merge_target_files::<Self::Loader>(&[path], None)?;
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

#[derive(Debug, Clone)]
/// generated field value with aggregated provenance references.
pub struct GeneratedField<T> {
    /// selected output value.
    pub value: T,
    /// all source refs that can yield this value.
    pub source_refs: Vec<SourceRef>,
}
