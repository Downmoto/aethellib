use std::collections::HashMap;
use rand::{Rng, seq::SliceRandom};

use crate::{generators::{GeneratedField, SourceRef}, merger::SourceAethelDoc};

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
fn push_unique_source_ref(into: &mut Vec<SourceRef>, source_ref: SourceRef) {
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