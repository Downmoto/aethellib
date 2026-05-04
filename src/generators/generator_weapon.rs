//! weapon generation logic backed by corpus-aware candidate indexing.

use std::fmt;

use rand::Rng;

use crate::generators::{
    GeneratedField, Generator, SourceRef,
    utils::{StringCandidate, build_pool, extend_unique_source_refs, choose_candidate}
};
use crate::loader::loader_weapon::WeaponLoader;
use crate::merger::{AethelCorpus, SourceAethelDoc};


#[derive(Debug)]
/// generated weapon payload containing assembled fields.
pub struct GeneratedWeapon {
    /// assembled display name.
    pub name: GeneratedField<String>,
    /// selected weapon type, if present in source data.
    pub weapon_type: Option<GeneratedField<String>>,
    /// selected rarity label, if present in source data.
    pub rarity: Option<GeneratedField<String>>,
    /// selected condition label, if present in source data.
    pub condition: Option<GeneratedField<String>>,
    /// generated lore sentence, if templates are available.
    pub lore: Option<GeneratedField<String>>,
    /// generated visual description, if templates are available.
    pub visuals: Option<GeneratedField<String>>,
}

impl fmt::Display for GeneratedWeapon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let weapon_type = self
            .weapon_type
            .as_ref()
            .map(|field| field.value.as_str())
            .unwrap_or("unknown");
        let rarity = self
            .rarity
            .as_ref()
            .map(|field| field.value.as_str())
            .unwrap_or("unspecified");
        let condition = self
            .condition
            .as_ref()
            .map(|field| field.value.as_str())
            .unwrap_or("unspecified");
        let lore = self
            .lore
            .as_ref()
            .map(|field| field.value.as_str())
            .unwrap_or("none");
        let visuals = self
            .visuals
            .as_ref()
            .map(|field| field.value.as_str())
            .unwrap_or("none");

        writeln!(f, "generated weapon")?;
        writeln!(f, "----------------")?;
        writeln!(f, "name      : {}", self.name.value)?;
        writeln!(f, "type      : {weapon_type}")?;
        writeln!(f, "rarity    : {rarity}")?;
        writeln!(f, "condition : {condition}")?;
        writeln!(f, "lore      : {lore}")?;
        write!(f, "visuals   : {visuals}")
    }
}

/// weapon generator backed by a merged weapon corpus.
pub struct WeaponGenerator {
    index: WeaponCandidateIndex,
}

impl Generator for WeaponGenerator {
    type Loader = WeaponLoader;
    type Output = GeneratedWeapon;

    /// creates a generator from a merged weapon corpus.
    fn new(corpus: AethelCorpus<WeaponLoader>) -> Self {
        let index = WeaponCandidateIndex::from_documents(&corpus.documents);
        Self { index }
    }

    /// builds a single generated weapon using an injected rng.
    fn generate_with_rng<R: Rng + ?Sized>(&self, rng: &mut R) -> GeneratedWeapon {
        let weapon_type = choose_candidate(&self.index.weapon_types, rng);

        let rarity = choose_candidate(&self.index.rarity, rng);

        let condition = choose_candidate(&self.index.condition, rng);

        let name = build_name(&self.index, rng);
        let lore = build_lore(&self.index, rng);
        let visuals = build_visuals(&self.index, rng);

        GeneratedWeapon {
            name,
            weapon_type,
            rarity,
            condition,
            lore,
            visuals,
        }
    }
}

/// indexed candidate pools used for weapon field sampling.
struct WeaponCandidateIndex {
    name_prefix: Vec<StringCandidate>,
    name_suffix: Vec<StringCandidate>,
    name_primitives: Vec<StringCandidate>,
    weapon_types: Vec<StringCandidate>,
    rarity: Vec<StringCandidate>,
    condition: Vec<StringCandidate>,
    lore_templates: Vec<StringCandidate>,
    lore_creators: Vec<StringCandidate>,
    lore_deeds: Vec<StringCandidate>,
    lore_quirks: Vec<StringCandidate>,
    visual_templates: Vec<StringCandidate>,
    visual_materials: Vec<StringCandidate>,
    visual_colours: Vec<StringCandidate>,
    visual_accents: Vec<StringCandidate>,
    visual_features: Vec<StringCandidate>,
}

impl WeaponCandidateIndex {
    /// builds all candidate pools from source weapon documents.
    fn from_documents(documents: &[SourceAethelDoc<WeaponLoader>]) -> Self {
        Self {
            name_prefix: build_pool(documents, "name", "prefix", |doc| {
                doc.name.as_ref().and_then(|section| section.prefix.as_ref())
            }),
            name_suffix: build_pool(documents, "name", "suffix", |doc| {
                doc.name.as_ref().and_then(|section| section.suffix.as_ref())
            }),
            name_primitives: build_pool(documents, "name", "primitives", |doc| {
                doc.name
                    .as_ref()
                    .and_then(|section| section.primitives.as_ref())
            }),
            weapon_types: build_pool(documents, "type", "type", |doc| {
                doc.weapon_type
                    .as_ref()
                    .and_then(|section| section.types.as_ref())
            }),
            rarity: build_pool(documents, "qualities", "rarity", |doc| {
                doc.qualities
                    .as_ref()
                    .and_then(|section| section.rarity.as_ref())
            }),
            condition: build_pool(documents, "qualities", "condition", |doc| {
                doc.qualities
                    .as_ref()
                    .and_then(|section| section.condition.as_ref())
            }),
            lore_templates: build_pool(documents, "lore", "templates", |doc| {
                doc.lore.as_ref().and_then(|section| section.templates.as_ref())
            }),
            lore_creators: build_pool(documents, "lore", "creators", |doc| {
                doc.lore.as_ref().and_then(|section| section.creators.as_ref())
            }),
            lore_deeds: build_pool(documents, "lore", "deeds", |doc| {
                doc.lore.as_ref().and_then(|section| section.deeds.as_ref())
            }),
            lore_quirks: build_pool(documents, "lore", "quirks", |doc| {
                doc.lore.as_ref().and_then(|section| section.quirks.as_ref())
            }),
            visual_templates: build_pool(documents, "visuals", "templates", |doc| {
                doc.visuals
                    .as_ref()
                    .and_then(|section| section.templates.as_ref())
            }),
            visual_materials: build_pool(documents, "visuals", "materials", |doc| {
                doc.visuals
                    .as_ref()
                    .and_then(|section| section.materials.as_ref())
            }),
            visual_colours: build_pool(documents, "visuals", "colours", |doc| {
                doc.visuals
                    .as_ref()
                    .and_then(|section| section.colours.as_ref())
            }),
            visual_accents: build_pool(documents, "visuals", "accents", |doc| {
                doc.visuals
                    .as_ref()
                    .and_then(|section| section.accents.as_ref())
            }),
            visual_features: build_pool(documents, "visuals", "features", |doc| {
                doc.visuals
                    .as_ref()
                    .and_then(|section| section.features.as_ref())
            }),
        }
    }
}

/// builds a generated name with aggregated provenance refs.
fn build_name(index: &WeaponCandidateIndex, rng: &mut (impl Rng + ?Sized)) -> GeneratedField<String> {
    let prefix = choose_candidate(&index.name_prefix, rng);
    let suffix = choose_candidate(&index.name_suffix, rng);
    let (core, core_refs) = build_primitive_core(&index.name_primitives, rng)
        .unwrap_or_else(|| ("weapon".to_string(), Vec::new()));

    let mut parts = Vec::new();
    let mut source_refs = Vec::new();

    if let Some(prefix) = prefix {
        parts.push(prefix.value);
        extend_unique_source_refs(&mut source_refs, &prefix.source_refs);
    }

    parts.push(core);
    extend_unique_source_refs(&mut source_refs, &core_refs);

    if let Some(suffix) = suffix {
        parts.push(suffix.value);
        extend_unique_source_refs(&mut source_refs, &suffix.source_refs);
    }

    GeneratedField {
        value: parts.join(" "),
        source_refs,
    }
}

/// composes a primitive core segment and returns its provenance refs.
fn build_primitive_core(
    primitives_pool: &[StringCandidate],
    rng: &mut (impl Rng + ?Sized),
) -> Option<(String, Vec<SourceRef>)> {
    if primitives_pool.is_empty() {
        return None;
    }

    let part_count = rng.gen_range(3..=5);
    let mut core = String::new();
    let mut source_refs = Vec::new();

    for _ in 0..part_count {
        if let Some(part) = choose_candidate(primitives_pool, rng) {
            core.push_str(&part.value);
            extend_unique_source_refs(&mut source_refs, &part.source_refs);
        }
    }

    if core.is_empty() {
        None
    } else {
        let mut chars = core.chars();
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

/// chooses optional text and returns an empty fallback when no candidates exist.
fn choose_optional_text(
    pool: &[StringCandidate],
    rng: &mut (impl Rng + ?Sized),
) -> (String, Vec<SourceRef>) {
    match choose_candidate(pool, rng) {
        Some(candidate) => (candidate.value, candidate.source_refs),
        None => (String::new(), Vec::new()),
    }
}

/// builds lore text from template and token candidates with merged provenance.
fn build_lore(
    index: &WeaponCandidateIndex,
    rng: &mut (impl Rng + ?Sized),
) -> Option<GeneratedField<String>> {
    let template = choose_candidate(&index.lore_templates, rng)?;
    let (creator, creator_refs) = choose_optional_text(&index.lore_creators, rng);
    let (deed, deed_refs) = choose_optional_text(&index.lore_deeds, rng);
    let (quirk, quirk_refs) = choose_optional_text(&index.lore_quirks, rng);

    let value = template
        .value
        .replace("{creator}", &creator)
        .replace("{deed}", &deed)
        .replace("{quirk}", &quirk);

    let mut source_refs = template.source_refs;
    extend_unique_source_refs(&mut source_refs, &creator_refs);
    extend_unique_source_refs(&mut source_refs, &deed_refs);
    extend_unique_source_refs(&mut source_refs, &quirk_refs);

    Some(GeneratedField { value, source_refs })
}

/// builds visual description text with merged provenance refs.
fn build_visuals(
    index: &WeaponCandidateIndex,
    rng: &mut (impl Rng + ?Sized),
) -> Option<GeneratedField<String>> {
    let template = choose_candidate(&index.visual_templates, rng)?;
    let (material, material_refs) = choose_optional_text(&index.visual_materials, rng);
    let (colour, colour_refs) = choose_optional_text(&index.visual_colours, rng);
    let (accent, accent_refs) = choose_optional_text(&index.visual_accents, rng);
    let (feature, feature_refs) = choose_optional_text(&index.visual_features, rng);

    let value = template
        .value
        .replace("{material}", &material)
        .replace("{colour}", &colour)
        .replace("{accent}", &accent)
        .replace("{feature}", &feature);

    let mut source_refs = template.source_refs;
    extend_unique_source_refs(&mut source_refs, &material_refs);
    extend_unique_source_refs(&mut source_refs, &colour_refs);
    extend_unique_source_refs(&mut source_refs, &accent_refs);
    extend_unique_source_refs(&mut source_refs, &feature_refs);

    Some(GeneratedField { value, source_refs })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;
    use crate::loader::AthelDocHeader;
    use crate::loader::TARGET_WEAPON;
    use crate::loader::loader_weapon::{
        WeaponLoreSection, WeaponNameSection, WeaponQualitiesSection, WeaponTypeSection,
        WeaponVisualSection,
    };

    #[test]
    fn test_dbg_random_weapon() {
        let generator = WeaponGenerator::from_file("data/weapon_test_data.toml").unwrap();
        let generated = generator.generate();

        assert!(!generated.name.value.is_empty());

    }

    #[test]
    fn test_generated_weapon_display_looks_nice() {
        let generated = GeneratedWeapon {
            name: GeneratedField {
                value: "Steel Longsword of Dawn".to_string(),
                source_refs: Vec::new(),
            },
            weapon_type: Some(GeneratedField {
                value: "longsword".to_string(),
                source_refs: Vec::new(),
            }),
            rarity: Some(GeneratedField {
                value: "rare".to_string(),
                source_refs: Vec::new(),
            }),
            condition: Some(GeneratedField {
                value: "pristine".to_string(),
                source_refs: Vec::new(),
            }),
            lore: Some(GeneratedField {
                value: "forged by old masters".to_string(),
                source_refs: Vec::new(),
            }),
            visuals: Some(GeneratedField {
                value: "silver blade with blue accents".to_string(),
                source_refs: Vec::new(),
            }),
        };

        let output = format!("{generated}");

        assert!(output.contains("generated weapon"));
        assert!(output.contains("name      : Steel Longsword of Dawn"));
        assert!(output.contains("type      : longsword"));
        assert!(output.contains("rarity    : rare"));
        assert!(output.contains("condition : pristine"));
        assert!(output.contains("lore      : forged by old masters"));
        assert!(output.contains("visuals   : silver blade with blue accents"));
    }

    #[test]
    fn test_generate_with_rng_is_deterministic_for_same_seed() {
        let generator = WeaponGenerator::from_file("data/weapon_test_data.toml").unwrap();

        let mut rng_a = StdRng::seed_from_u64(42);
        let mut rng_b = StdRng::seed_from_u64(42);

        let generated_a = generator.generate_with_rng(&mut rng_a);
        let generated_b = generator.generate_with_rng(&mut rng_b);

        assert_eq!(generated_a.name.value, generated_b.name.value);
        assert_eq!(generated_a.name.source_refs, generated_b.name.source_refs);
        assert_eq!(
            generated_a.weapon_type.as_ref().map(|field| &field.value),
            generated_b.weapon_type.as_ref().map(|field| &field.value)
        );
        assert_eq!(
            generated_a.rarity.as_ref().map(|field| &field.value),
            generated_b.rarity.as_ref().map(|field| &field.value)
        );
        assert_eq!(
            generated_a.condition.as_ref().map(|field| &field.value),
            generated_b.condition.as_ref().map(|field| &field.value)
        );
        assert_eq!(
            generated_a.lore.as_ref().map(|field| &field.value),
            generated_b.lore.as_ref().map(|field| &field.value)
        );
        assert_eq!(
            generated_a.visuals.as_ref().map(|field| &field.value),
            generated_b.visuals.as_ref().map(|field| &field.value)
        );
    }

    #[test]
    fn test_rarity_retains_all_sources_after_dedup() {
        let doc_a = test_source_doc(
            "source-a",
            "set-a",
            WeaponLoader {
                name: None,
                weapon_type: None,
                qualities: Some(WeaponQualitiesSection {
                    rarity: Some(vec!["Rare".to_string()]),
                    condition: None,
                }),
                lore: None,
                visuals: None,
            },
        );

        let doc_b = test_source_doc(
            "source-b",
            "set-b",
            WeaponLoader {
                name: None,
                weapon_type: None,
                qualities: Some(WeaponQualitiesSection {
                    rarity: Some(vec!["Rare".to_string()]),
                    condition: None,
                }),
                lore: None,
                visuals: None,
            },
        );

        let generator = WeaponGenerator::from_documents(vec![doc_a, doc_b]);
        let generated = generator.generate();

        let rarity = generated.rarity.unwrap();
        assert_eq!(rarity.value, "Rare");
        assert_eq!(rarity.source_refs.len(), 2);
    }

    #[test]
    fn test_lore_contains_template_and_token_sources() {
        let doc = test_source_doc(
            "source-lore",
            "lore-set",
            WeaponLoader {
                name: Some(WeaponNameSection {
                    prefix: Some(vec!["Iron".to_string()]),
                    suffix: None,
                    primitives: Some(vec!["ka".to_string()]),
                }),
                weapon_type: Some(WeaponTypeSection {
                    types: Some(vec!["longsword".to_string()]),
                }),
                qualities: Some(WeaponQualitiesSection {
                    rarity: Some(vec!["Rare".to_string()]),
                    condition: Some(vec!["Pristine".to_string()]),
                }),
                lore: Some(WeaponLoreSection {
                    creators: Some(vec!["a blind smith".to_string()]),
                    deeds: Some(vec!["to guard the vale".to_string()]),
                    quirks: Some(vec!["it sings softly".to_string()]),
                    templates: Some(vec!["forged by {creator} {deed}. {quirk}".to_string()]),
                }),
                visuals: Some(WeaponVisualSection {
                    materials: Some(vec!["steel".to_string()]),
                    colours: Some(vec!["grey".to_string()]),
                    accents: Some(vec!["in leather".to_string()]),
                    features: Some(vec!["etched with runes".to_string()]),
                    templates: Some(vec![
                        "made of {material}, {accent}, {colour}, {feature}".to_string(),
                    ]),
                }),
            },
        );

        let generator = WeaponGenerator::from_documents(vec![doc]);
        let generated = generator.generate();

        let lore = generated.lore.unwrap();

        assert!(
            lore.source_refs
                .iter()
                .any(|source| source.section == "lore" && source.field == "templates")
        );
        assert!(
            lore.source_refs
                .iter()
                .any(|source| source.section == "lore" && source.field == "creators")
        );
        assert!(
            lore.source_refs
                .iter()
                .any(|source| source.section == "lore" && source.field == "deeds")
        );
        assert!(
            lore.source_refs
                .iter()
                .any(|source| source.section == "lore" && source.field == "quirks")
        );
    }

    #[test]
    fn test_generate_without_name_section_uses_weapon_fallback() {
        let doc = test_source_doc(
            "source-no-name",
            "no-name-set",
            WeaponLoader {
                name: None,
                weapon_type: Some(WeaponTypeSection {
                    types: Some(vec!["longsword".to_string()]),
                }),
                qualities: Some(WeaponQualitiesSection {
                    rarity: Some(vec!["Rare".to_string()]),
                    condition: Some(vec!["Pristine".to_string()]),
                }),
                lore: None,
                visuals: None,
            },
        );

        let generator = WeaponGenerator::from_documents(vec![doc]);
        let generated = generator.generate();

        assert_eq!(generated.name.value, "weapon");
        assert!(generated.name.source_refs.is_empty());
    }

    fn test_source_doc(
        source_id: &str,
        name: &str,
        data: WeaponLoader,
    ) -> SourceAethelDoc<WeaponLoader> {
        SourceAethelDoc {
            source_id: source_id.to_string(),
            source_hash: format!("hash-{source_id}"),
            source_path: format!("{source_id}.toml"),
            header: AthelDocHeader {
                name: name.to_string(),
                target: TARGET_WEAPON.to_string(),
                desc: None,
                author: None,
                version: None,
            },
            data,
        }
    }
}
