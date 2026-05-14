use rand::Rng;
use std::hash::Hash;

use crate::{
    AethelCorpus,
    generator::{
        GeneratedField, GeneratedFieldCandidates, GenerationError, ProvenanceCandidateIndex,
        SourceRef, TraceEdge, TraceEdgeKind, TraceGraph, TraceNode, TraceNodeKind,
    },
};

/// fluent builder for one generated field candidate pool.
pub struct GeneratedFieldBuilder<'a, L, T, F>
where
    T: Eq + Hash + Clone,
    F: FnMut(&L) -> Vec<T>,
{
    corpus: &'a AethelCorpus<L>,
    section: String,
    field: String,
    extract_values: F,
}

impl<'a, L, T, F> GeneratedFieldBuilder<'a, L, T, F>
where
    T: Eq + Hash + Clone,
    F: FnMut(&L) -> Vec<T>,
{
    /// builds candidate values and provenance refs from the configured extractor.
    pub fn build(mut self) -> GeneratedFieldCandidates<T> {
        let mut values: Vec<T> = Vec::new();
        let mut provenance: ProvenanceCandidateIndex<T> = ProvenanceCandidateIndex::new();

        for source_document in &self.corpus.documents {
            let extracted_values = (self.extract_values)(&source_document.data);
            for value in extracted_values {
                values.push(value.clone());
                provenance.insert(
                    value,
                    SourceRef {
                        source_id: source_document.source_id.clone(),
                        source_name: source_document.header.title.clone(),
                        section: self.section.clone(),
                        field: self.field.clone(),
                    },
                );
            }
        }

        GeneratedFieldCandidates { values, provenance }
    }
}

/// creates a fluent builder for one generated field candidate pool.
pub fn generated_field_builder<'a, L, T, F>(
    corpus: &'a AethelCorpus<L>,
    section: &str,
    field: &str,
    extract_values: F,
) -> GeneratedFieldBuilder<'a, L, T, F>
where
    T: Eq + Hash + Clone,
    F: FnMut(&L) -> Vec<T>,
{
    GeneratedFieldBuilder {
        corpus,
        section: section.to_string(),
        field: field.to_string(),
        extract_values,
    }
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
    let root_id = "selection:0".to_string();

    let mut nodes: Vec<TraceNode> = Vec::new();
    nodes.push(TraceNode {
        id: root_id.clone(),
        kind: TraceNodeKind::Selection,
        label: "sample selection".to_string(),
        source_ref: None,
    });

    let mut edges: Vec<TraceEdge> = Vec::new();
    for (idx, source_ref) in source_refs.iter().enumerate() {
        let source_node_id = format!("source:{idx}");
        nodes.push(TraceNode {
            id: source_node_id.clone(),
            kind: TraceNodeKind::Source,
            label: source_ref.source_name.clone(),
            source_ref: Some(source_ref.clone()),
        });
        edges.push(TraceEdge {
            from: source_node_id,
            to: root_id.clone(),
            kind: TraceEdgeKind::DerivedFrom,
        });
    }

    let trace_graph = TraceGraph::new(nodes, edges);

    GeneratedField::new(value, source_refs, trace_graph, root_id)
}
