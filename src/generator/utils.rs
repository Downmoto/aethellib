use rand::Rng;
use std::hash::Hash;

use crate::{
    AethelCorpus,
    generator::{
        GeneratedField, GeneratedFieldCandidates, GenerationError, ProvenanceCandidateIndex,
        SourceRef,
    },
};

/// creates candidates for one generated field candidate pool.
pub fn generated_field_builder<L, T, F>(
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
pub(crate) fn sample_generated_field<T, R>(
    values: &[T],
    index: &ProvenanceCandidateIndex<T>,
    rng: &mut R,
) -> Result<GeneratedField<T>, GenerationError>
where
    T: Eq + Hash + Clone,
    R: Rng + ?Sized,
{
    if values.is_empty() {
        return Err(GenerationError::EmptyCandidates {
            field: "unknown".to_string(),
        });
    }

    let picked_index = rng.gen_range(0..values.len());
    let value = values[picked_index].clone();
    let source_refs = index.refs_for(&value).to_vec();

    GeneratedField::new(value, source_refs)
}
