//! generation logic for aethel content types.
//!
//! this module exposes concrete generators that build runtime content
//! from loaded and validated aethel documents.
//! generators should expose a convenience `generate()` method and
//! a deterministic `generate_with_rng(...)` method for reproducible tests.

use std::collections::HashMap;

use rand::Rng;
use rand::thread_rng;
use rand::seq::SliceRandom;

use crate::loader::TargetedLoader;
use crate::merger::merge_target_files;
use crate::merger::{AethelCorpus, MergerError, SourceAethelDoc};

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

pub(crate) type StringCandidate = GeneratedField<String>;

/// chooses one candidate from a pool.
pub(crate) fn choose_candidate(
    pool: &[StringCandidate],
    rng: &mut (impl Rng + ?Sized),
) -> Option<StringCandidate> {
    pool.choose(rng).cloned()
}

/// adapter trait for generic candidate-pool construction from source docs.
pub(crate) trait PoolDocument<TData> {
	/// stable source id used in provenance.
	fn source_id(&self) -> &str;
	/// source display name used in provenance.
	fn source_name(&self) -> &str;
	/// source body data used by extractor callbacks.
	fn data(&self) -> &TData;
}

impl<TData> PoolDocument<TData> for SourceAethelDoc<TData> {
	fn source_id(&self) -> &str {
		&self.source_id
	}

	fn source_name(&self) -> &str {
		&self.header.name
	}

	fn data(&self) -> &TData {
		&self.data
	}
}

/// builds a deduplicated candidate pool for a section field across source documents.
pub(crate) fn build_pool<TDoc, TData, F>(
	documents: &[TDoc],
	section: &str,
	field: &str,
	extractor: F,
) -> Vec<StringCandidate>
where
	TDoc: PoolDocument<TData>,
	F: for<'a> Fn(&'a TData) -> Option<&'a Vec<String>>,
{
	let mut candidates: Vec<StringCandidate> = Vec::new();
	let mut value_index: HashMap<String, usize> = HashMap::new();

	for source in documents {
		let Some(values) = extractor(source.data()) else {
			continue;
		};

		for value in values {
			let source_ref = SourceRef {
				source_id: source.source_id().to_string(),
				source_name: source.source_name().to_string(),
				section: section.to_string(),
				field: field.to_string(),
			};

			if let Some(idx) = value_index.get(value) {
				push_unique_source_ref(&mut candidates[*idx].source_refs, source_ref);
			} else {
				value_index.insert(value.clone(), candidates.len());
				candidates.push(StringCandidate {
					value: value.clone(),
					source_refs: vec![source_ref],
				});
			}
		}
	}

	candidates
}

/// appends a source ref only if it is not already present.
pub(crate) fn push_unique_source_ref(into: &mut Vec<SourceRef>, source_ref: SourceRef) {
	if !into.contains(&source_ref) {
		into.push(source_ref);
	}
}

/// extends source refs with uniqueness preservation.
pub(crate) fn extend_unique_source_refs(into: &mut Vec<SourceRef>, refs: &[SourceRef]) {
	for source_ref in refs {
		push_unique_source_ref(into, source_ref.clone());
	}
}

pub mod generator_person;
pub mod generator_weapon;