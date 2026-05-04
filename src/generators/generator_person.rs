//! person generation logic backed by corpus-aware primitive indexing.

use rand::Rng;
use rand::seq::SliceRandom;

use crate::generators::{
    GeneratedField, Generator, SourceRef, StringCandidate, build_pool, extend_unique_source_refs,
};
use crate::loader::loader_person::PersonLoader;
use crate::merger::{AethelCorpus, SourceAethelDoc};


#[derive(Debug)]
/// generated person payload containing assembled fields.
pub struct GeneratedPerson {
    /// assembled display name.
    pub name: GeneratedField<String>,
}

/// person generator backed by a merged person corpus.
pub struct PersonGenerator {
    index: PersonCandidateIndex,
}

impl Generator for PersonGenerator {
    type Loader = PersonLoader;
    type Output = GeneratedPerson;

    /// creates a generator from a merged person corpus.
    fn new(corpus: AethelCorpus<PersonLoader>) -> Self {
        let index = PersonCandidateIndex::from_documents(&corpus.documents);
        Self { index }
    }

    /// builds a single generated person using an injected rng.
    fn generate_with_rng<R: Rng + ?Sized>(&self, rng: &mut R) -> GeneratedPerson {
        let name = build_name(&self.index, rng);
        GeneratedPerson { name }
    }
}

/// indexed primitive pools used for person name sampling.
struct PersonCandidateIndex {
    first_primitives: Vec<StringCandidate>,
    middle_primitives: Vec<StringCandidate>,
    last_primitives: Vec<StringCandidate>,
}

impl PersonCandidateIndex {
    /// builds candidate pools from source person documents.
    fn from_documents(documents: &[SourceAethelDoc<PersonLoader>]) -> Self {
        Self {
            first_primitives: build_pool(documents, "name", "first", |doc| {
                doc.name.as_ref().and_then(|section| section.first.as_ref())
            }),
            middle_primitives: build_pool(documents, "name", "middle", |doc| {
                doc.name.as_ref().and_then(|section| section.middle.as_ref())
            }),
            last_primitives: build_pool(documents, "name", "last", |doc| {
                doc.name.as_ref().and_then(|section| section.last.as_ref())
            }),
        }
    }
}

/// chooses one candidate from a pool.
fn choose_candidate(pool: &[StringCandidate], rng: &mut (impl Rng + ?Sized)) -> Option<StringCandidate> {
    pool.choose(rng).cloned()
}

/// builds a generated full name with merged provenance references.
fn build_name(index: &PersonCandidateIndex, rng: &mut (impl Rng + ?Sized)) -> GeneratedField<String> {
    let (first, first_refs) = build_primitive_segment(&index.first_primitives, rng)
        .unwrap_or_else(|| ("nam".to_string(), Vec::new()));
    let middle_segment = build_primitive_segment(&index.middle_primitives, rng);
    let (last, last_refs) = build_primitive_segment(&index.last_primitives, rng)
        .unwrap_or_else(|| ("less".to_string(), Vec::new()));

    let mut parts = Vec::new();
    let mut source_refs = Vec::new();

    parts.push(first);
    extend_unique_source_refs(&mut source_refs, &first_refs);

    if let Some((middle, middle_refs)) = middle_segment {
        parts.push(middle);
        extend_unique_source_refs(&mut source_refs, &middle_refs);
    }

    parts.push(last);
    extend_unique_source_refs(&mut source_refs, &last_refs);

    GeneratedField {
        value: parts.join(" "),
        source_refs,
    }
}

/// composes one primitive segment and returns its provenance refs.
fn build_primitive_segment(
    primitives_pool: &[StringCandidate],
    rng: &mut (impl Rng + ?Sized),
) -> Option<(String, Vec<SourceRef>)> {
    if primitives_pool.is_empty() {
        return None;
    }

    let part_count = rng.gen_range(2..=4);
    let mut segment = String::new();
    let mut source_refs = Vec::new();

    for _ in 0..part_count {
        if let Some(part) = choose_candidate(primitives_pool, rng) {
            segment.push_str(&part.value);
            extend_unique_source_refs(&mut source_refs, &part.source_refs);
        }
    }

    if segment.is_empty() {
        None
    } else {
        let mut chars = segment.chars();
        if let Some(first) = chars.next() {
            let mut capitalized = String::new();
            capitalized.push(first.to_ascii_uppercase());
            capitalized.push_str(chars.as_str());
            Some((capitalized, source_refs))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn test_generate_person_name_from_file() {
        let generator = PersonGenerator::from_file("data/person_test_data.toml").unwrap();
        let generated = generator.generate();

        assert!(!generated.name.value.is_empty());
        assert!(generated.name.value.split_whitespace().count() >= 2);
    }

    #[test]
    fn test_generate_with_rng_is_deterministic_for_same_seed() {
        let generator = PersonGenerator::from_file("data/person_test_data.toml").unwrap();

        let mut rng_a = StdRng::seed_from_u64(99);
        let mut rng_b = StdRng::seed_from_u64(99);

        let generated_a = generator.generate_with_rng(&mut rng_a);
        let generated_b = generator.generate_with_rng(&mut rng_b);

        assert_eq!(generated_a.name.value, generated_b.name.value);
        assert_eq!(generated_a.name.source_refs, generated_b.name.source_refs);
    }
}
