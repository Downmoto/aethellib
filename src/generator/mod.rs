//! generation logic for aethel content types.
//!
//! this module exposes concrete generators that build runtime content
//! from loaded and validated aethel documents.
//! generators should expose a convenience `generate()` method and
//! a deterministic `generate_with_rng(...)` method for reproducible tests.

use std::path::Path;
use std::{collections::HashMap, hash::Hash};

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

/// optional provenance-first generator interface.
///
/// this trait does not replace `Generator`; it provides a clearer semantic path
/// for generator implementations whose output is always `GeneratedField<T>`.
pub trait ProvenanceGenerator: Generator<Output = GeneratedField<Self::Value>> {
    /// scalar generated value type wrapped by `GeneratedField`.
    type Value;

    /// builds one provenance-rich output value using the supplied rng.
    fn generate_field_with_rng<R: Rng + ?Sized>(&self, rng: &mut R) -> GeneratedField<Self::Value> {
        self.generate_with_rng(rng)
    }

    /// builds one provenance-rich output value with thread-local randomness.
    fn generate_field(&self) -> GeneratedField<Self::Value> {
        let mut rng = thread_rng();
        self.generate_field_with_rng(&mut rng)
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

#[derive(Debug, Clone)]
/// prepared candidates and provenance index for one generated output field.
pub struct GeneratedFieldCandidates<T> {
    /// candidate values used for sampling.
    pub values: Vec<T>,
    /// value-to-provenance lookup built from source documents.
    pub provenance: ProvenanceCandidateIndex<T>,
}

impl<T> GeneratedFieldCandidates<T>
where
    T: Eq + Hash + Clone,
{
    /// samples one generated field using the prepared values and provenance.
    pub fn sample_with_rng<R: Rng + ?Sized>(&self, rng: &mut R) -> Option<GeneratedField<T>> {
        sample_generated_field(&self.values, &self.provenance, rng)
    }
}

#[derive(Debug, Clone)]
/// index mapping generated values to all matching provenance references.
pub struct ProvenanceCandidateIndex<T> {
    entries: HashMap<T, Vec<SourceRef>>,
}

impl<T> ProvenanceCandidateIndex<T>
where
    T: Eq + Hash,
{
    /// creates an empty provenance candidate index.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// inserts one value-to-source mapping and keeps first-seen order for refs.
    pub fn insert(&mut self, value: T, source_ref: SourceRef) {
        let refs = self.entries.entry(value).or_default();
        if !refs.contains(&source_ref) {
            refs.push(source_ref);
        }
    }

    /// returns all provenance refs for one candidate value.
    pub fn refs_for(&self, value: &T) -> &[SourceRef] {
        self.entries
            .get(value)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// returns true when the index contains at least one mapping for a value.
    pub fn contains_value(&self, value: &T) -> bool {
        self.entries.contains_key(value)
    }
}

impl<T> Default for ProvenanceCandidateIndex<T>
where
    T: Eq + Hash,
{
    fn default() -> Self {
        Self::new()
    }
}

/// collects candidate values and provenance refs from one corpus field extractor.
pub fn collect_generated_field_candidates<L, T, F>(
    corpus: &AethelCorpus<L>,
    section: &str,
    field: &str,
    mut extract_values: F,
) -> GeneratedFieldCandidates<T>
where
    T: Eq + Hash + Clone,
    F: FnMut(&L) -> Vec<T>,
{
    let mut values: Vec<T> = Vec::new();
    let mut provenance: ProvenanceCandidateIndex<T> = ProvenanceCandidateIndex::new();

    for source_document in &corpus.documents {
        let extracted_values = extract_values(&source_document.data);
        for value in extracted_values {
            values.push(value.clone());
            provenance.insert(
                value,
                SourceRef {
                    source_id: source_document.source_id.clone(),
                    source_name: source_document.header.title.clone(),
                    section: section.to_string(),
                    field: field.to_string(),
                },
            );
        }
    }

    GeneratedFieldCandidates { values, provenance }
}

/// samples one candidate value and returns it with all known provenance refs.
pub fn sample_generated_field<T, R>(
    values: &[T],
    index: &ProvenanceCandidateIndex<T>,
    rng: &mut R,
) -> Option<GeneratedField<T>>
where
    T: Eq + Hash + Clone,
    R: Rng + ?Sized,
{
    if values.is_empty() {
        return None;
    }

    let picked_index = rng.gen_range(0..values.len());
    let value = values[picked_index].clone();
    let source_refs = index.refs_for(&value).to_vec();

    Some(GeneratedField { value, source_refs })
}

/// samples one `(value, refs)` pair and returns a generated field.
pub fn choose_generated_field<T, R>(
    pairs: &[(T, Vec<SourceRef>)],
    rng: &mut R,
) -> Option<GeneratedField<T>>
where
    T: Clone,
    R: Rng + ?Sized,
{
    if pairs.is_empty() {
        return None;
    }

    let picked_index = rng.gen_range(0..pairs.len());
    let (value, source_refs) = &pairs[picked_index];

    Some(GeneratedField {
        value: value.clone(),
        source_refs: source_refs.clone(),
    })
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
