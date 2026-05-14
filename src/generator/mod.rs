//! generation logic for aethel content types.
//!
//! this module exposes concrete generators that build runtime content
//! from loaded and validated aethel documents.
//! generation implementations compile reusable candidate state, then expose
//! inherent methods for runtime sampling.
pub mod utils;
pub use crate::generator::utils::generated_field_builder;

use std::path::Path;
use std::{collections::HashMap, hash::Hash};

use rand::Rng;

use crate::AethelCorpus;
use crate::SourceAethelDoc;
use crate::loader::TargetedLoader;
use crate::merger::error::MergerError;
use crate::merger::merge_files;

#[derive(Debug, Clone, PartialEq, Eq)]
/// generation errors for compile and sampling paths.
pub enum GenerationError {
    /// generation state must be compiled before sampling.
    NotCompiled,
    /// one generated field has no candidates.
    EmptyCandidates { field: String },
    /// trace graph shape failed validation.
    InvalidTraceGraph(String),
    /// field builder definition is invalid.
    BuilderDefinition { field: String, reason: String },
}

impl std::fmt::Display for GenerationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenerationError::NotCompiled => write!(f, "generation state is not compiled"),
            GenerationError::EmptyCandidates { field } => {
                write!(f, "no candidates are available for field '{field}'")
            }
            GenerationError::InvalidTraceGraph(message) => write!(f, "invalid trace graph: {message}"),
            GenerationError::BuilderDefinition { field, reason } => {
                write!(f, "invalid builder definition for field '{field}': {reason}")
            }
        }
    }
}

impl std::error::Error for GenerationError {}

impl From<MergerError> for GenerationError {
    fn from(value: MergerError) -> Self {
        GenerationError::BuilderDefinition {
            field: "corpus".to_string(),
            reason: value.to_string(),
        }
    }
}

/// generic generation contract with shared constructors and compile lifecycle.
pub trait Generation: Sized {
    /// loader payload type used by this generator target.
    type Loader: TargetedLoader;
    /// reusable compiled state owned by implementers.
    type CompiledState;

    /// compiles a generation object from a merged target corpus.
    fn compile(corpus: AethelCorpus<Self::Loader>) -> Result<Self, GenerationError>;

    /// creates a generation object directly from source documents.
    fn from_documents(documents: Vec<SourceAethelDoc<Self::Loader>>) -> Result<Self, GenerationError> {
        Self::compile(AethelCorpus {
            target: <Self::Loader as TargetedLoader>::TARGET.to_string(),
            documents,
        })
    }

    /// loads target files and creates a corpus-backed generation object.
    fn from_files(paths: &[impl AsRef<Path>]) -> Result<Self, GenerationError> {
        let corpus = merge_files::<Self::Loader>(paths, None)?;
        Self::compile(corpus)
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

#[derive(Debug, Clone, PartialEq, Eq)]
/// node kind used in field-level dependency graphs.
pub enum TraceNodeKind {
    Source,
    Selection,
    Transform,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// edge kind used in field-level dependency graphs.
pub enum TraceEdgeKind {
    DerivedFrom,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// one trace node in a generated field graph.
pub struct TraceNode {
    /// stable node id within one graph.
    pub id: String,
    /// semantic node kind.
    pub kind: TraceNodeKind,
    /// human-readable node label.
    pub label: String,
    /// optional source reference for source nodes.
    pub source_ref: Option<SourceRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// one trace edge in a generated field graph.
pub struct TraceEdge {
    /// upstream node id.
    pub from: String,
    /// downstream node id.
    pub to: String,
    /// semantic edge kind.
    pub kind: TraceEdgeKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
/// structured trace graph for one generated field.
pub struct TraceGraph {
    nodes: Vec<TraceNode>,
    edges: Vec<TraceEdge>,
}

impl TraceGraph {
    /// creates a graph from node and edge lists.
    pub fn new(nodes: Vec<TraceNode>, edges: Vec<TraceEdge>) -> Self {
        Self { nodes, edges }
    }

    /// returns all nodes.
    pub fn nodes(&self) -> &[TraceNode] {
        self.nodes.as_slice()
    }

    /// returns all edges.
    pub fn edges(&self) -> &[TraceEdge] {
        self.edges.as_slice()
    }

    /// returns true when a node id exists in the graph.
    pub fn has_node(&self, id: &str) -> bool {
        self.nodes.iter().any(|node| node.id == id)
    }
}

#[derive(Debug, Clone)]
/// generated field value with aggregated provenance references.
pub struct GeneratedField<T> {
    value: T,
    source_refs: Vec<SourceRef>,
    trace_graph: TraceGraph,
    trace_root_id: String,
    field_name: Option<String>,
    compiled_candidates: Option<GeneratedFieldCandidates<T>>,
}

impl<T> GeneratedField<T> {
    /// creates one generated field with validated trace metadata.
    pub(crate) fn new(
        value: T,
        source_refs: Vec<SourceRef>,
        trace_graph: TraceGraph,
        trace_root_id: String,
    ) -> Result<Self, GenerationError> {
        if !trace_graph.has_node(trace_root_id.as_str()) {
            return Err(GenerationError::InvalidTraceGraph(format!(
                "missing root node '{trace_root_id}'"
            )));
        }

        Ok(Self {
            value,
            source_refs,
            trace_graph,
            trace_root_id,
            field_name: None,
            compiled_candidates: None,
        })
    }

    /// creates a default uncompiled generated field value.
    pub fn pending(value: T) -> Self {
        Self {
            value,
            source_refs: Vec::new(),
            trace_graph: TraceGraph::default(),
            trace_root_id: String::new(),
            field_name: None,
            compiled_candidates: None,
        }
    }

    /// attaches compiled candidates to this field for self-managed re-sampling.
    pub fn set_candidates(&mut self, field: impl Into<String>, candidates: GeneratedFieldCandidates<T>) {
        self.field_name = Some(field.into());
        self.compiled_candidates = Some(candidates);
    }

    /// returns true when this field has compiled candidates attached.
    pub fn is_compiled(&self) -> bool {
        self.compiled_candidates.is_some()
    }

    /// returns the selected output value.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// returns all provenance references attached to this value.
    pub fn source_refs(&self) -> &[SourceRef] {
        self.source_refs.as_slice()
    }

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

    /// returns the dependency trace graph for this field.
    pub fn trace_graph(&self) -> &TraceGraph {
        &self.trace_graph
    }

    /// returns the trace root node id for this field.
    pub fn trace_root_id(&self) -> &str {
        self.trace_root_id.as_str()
    }

    /// re-samples this field from its own compiled candidates and returns a new value.
    pub fn regenerate_with_rng<R: Rng + ?Sized>(&self, rng: &mut R) -> Result<Self, GenerationError>
    where
        T: Eq + Hash + Clone,
    {
        let field = self
            .field_name
            .as_deref()
            .ok_or(GenerationError::NotCompiled)?;
        let candidates = self
            .compiled_candidates
            .as_ref()
            .ok_or(GenerationError::NotCompiled)?;

        candidates.sample(field, rng)
    }

    /// re-samples this field from its own compiled candidates using thread-local randomness.
    pub fn regenerate(&self) -> Result<Self, GenerationError>
    where
        T: Eq + Hash + Clone,
    {
        let mut rng = rand::thread_rng();
        self.regenerate_with_rng(&mut rng)
    }

    /// re-samples this field in place from its own compiled candidates.
    pub fn regenerate_in_place_with_rng<R: Rng + ?Sized>(&mut self, rng: &mut R) -> Result<(), GenerationError>
    where
        T: Eq + Hash + Clone,
    {
        let next_value = self.regenerate_with_rng(rng)?;
        *self = next_value;
        Ok(())
    }

    /// re-samples this field in place using thread-local randomness.
    pub fn regenerate_in_place(&mut self) -> Result<(), GenerationError>
    where
        T: Eq + Hash + Clone,
    {
        let mut rng = rand::thread_rng();
        self.regenerate_in_place_with_rng(&mut rng)
    }
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
    pub fn sample<R: Rng + ?Sized>(
        &self,
        field: &str,
        rng: &mut R,
    ) -> Result<GeneratedField<T>, GenerationError> {
        let mut generated_field = utils::sample_generated_field(&self.values, &self.provenance, rng)
            .map_err(|error| match error {
                GenerationError::EmptyCandidates { .. } => GenerationError::EmptyCandidates {
                    field: field.to_string(),
                },
                _ => error,
            })?;

        generated_field.set_candidates(field.to_string(), self.clone());
        Ok(generated_field)
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
        self.entries.get(value).map(Vec::as_slice).unwrap_or(&[])
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
