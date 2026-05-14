//! expanded 0.5 generation example.
//!
//! this shows a richer generation object with many fields, multi-section
//! documents, selective field re-sampling, and detailed provenance output.
//!
//! walkthrough order:
//! 1) define loader sections (`DemoLoader`, `IdentityValues`, `LoreValues`, `WorldValues`)
//! 2) define one generation struct composed only of `GeneratedField<T>` values
//! 3) attach field-local candidates in `Generation::compile(corpus)`
//! 5) generate full profiles and selectively reroll subsets
//! 6) print value and provenance refs

use std::error::Error;

use aethellib::generator::generated_field_builder;
use aethellib::prelude::{
    AethelCorpus, AethelDoc, AethelDocHeader, GeneratedField, Generation, GenerationError,
    SourceAethelDoc, TargetedLoader,
};
use rand::Rng;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
/// loader payload split into domain sections so field provenance is explicit.
struct DemoLoader {
    identity: IdentityValues,
    lore: LoreValues,
    world: WorldValues,
}

impl TargetedLoader for DemoLoader {
    const TARGET: &'static str = "demo";
}

#[derive(Debug, Clone, Deserialize, Serialize)]
/// identity-focused candidate pools.
struct IdentityValues {
    titles: Vec<String>,
    first_names: Vec<String>,
    middle_names: Vec<String>,
    last_names: Vec<String>,
    suffixes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
/// lore-focused candidate pools.
struct LoreValues {
    epithets: Vec<String>,
    dynasties: Vec<String>,
    eras: Vec<String>,
    omens: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
/// world-focused candidate pools.
struct WorldValues {
    vocations: Vec<String>,
    homelands: Vec<String>,
    relics: Vec<String>,
}

#[derive(Debug, Clone)]
/// generation object with field-owned candidate state.
///
/// note that this struct intentionally has no parallel `compiled_*` members.
/// each `GeneratedField<T>` carries its own candidates for self-managed rerolls.
struct DemoGeneration {
    title: GeneratedField<String>,
    name: GeneratedField<String>,
    middle_name: GeneratedField<String>,
    last_name: GeneratedField<String>,
    suffix: GeneratedField<String>,
    epithet: GeneratedField<String>,
    dynasty: GeneratedField<String>,
    era: GeneratedField<String>,
    vocation: GeneratedField<String>,
    homeland: GeneratedField<String>,
    relic: GeneratedField<String>,
    omen: GeneratedField<String>,
}

impl DemoGeneration {
    /// generates a complete profile by asking each field to regenerate itself.
    fn generate_with_rng<R: Rng + ?Sized>(&self, rng: &mut R) -> Result<Self, GenerationError> {
        let title = self.title.regenerate_with_rng(rng)?;
        let name = self.name.regenerate_with_rng(rng)?;
        let middle_name = self.middle_name.regenerate_with_rng(rng)?;
        let last_name = self.last_name.regenerate_with_rng(rng)?;
        let suffix = self.suffix.regenerate_with_rng(rng)?;
        let epithet = self.epithet.regenerate_with_rng(rng)?;
        let dynasty = self.dynasty.regenerate_with_rng(rng)?;
        let era = self.era.regenerate_with_rng(rng)?;
        let vocation = self.vocation.regenerate_with_rng(rng)?;
        let homeland = self.homeland.regenerate_with_rng(rng)?;
        let relic = self.relic.regenerate_with_rng(rng)?;
        let omen = self.omen.regenerate_with_rng(rng)?;

        Ok(Self {
            title,
            name,
            middle_name,
            last_name,
            suffix,
            epithet,
            dynasty,
            era,
            vocation,
            homeland,
            relic,
            omen,
        })
    }

    fn generate(&self) -> Result<Self, GenerationError> {
        let mut rng = rand::thread_rng();
        self.generate_with_rng(&mut rng)
    }

    fn name(&self) -> &GeneratedField<String> {
        &self.name
    }

    fn middle_name(&self) -> &GeneratedField<String> {
        &self.middle_name
    }

    fn last_name(&self) -> &GeneratedField<String> {
        &self.last_name
    }

    fn suffix(&self) -> &GeneratedField<String> {
        &self.suffix
    }

    fn title(&self) -> &GeneratedField<String> {
        &self.title
    }

    fn epithet(&self) -> &GeneratedField<String> {
        &self.epithet
    }

    fn dynasty(&self) -> &GeneratedField<String> {
        &self.dynasty
    }

    fn era(&self) -> &GeneratedField<String> {
        &self.era
    }

    fn vocation(&self) -> &GeneratedField<String> {
        &self.vocation
    }

    fn homeland(&self) -> &GeneratedField<String> {
        &self.homeland
    }

    fn relic(&self) -> &GeneratedField<String> {
        &self.relic
    }

    fn omen(&self) -> &GeneratedField<String> {
        &self.omen
    }

    /// helper used for readable console output and composition demos.
    fn full_name(&self) -> String {
        format!(
            "{} {} {} {} {}",
            self.title().value(),
            self.name().value(),
            self.middle_name().value(),
            self.last_name().value(),
            self.suffix().value()
        )
    }

    /// rerolls only mutable name components and keeps other identity fields fixed.
    fn reroll_name_components_in_place_with_rng<R: Rng + ?Sized>(
        &mut self,
        rng: &mut R,
    ) -> Result<(), GenerationError> {
        self.name.regenerate_in_place_with_rng(rng)?;
        self.middle_name.regenerate_in_place_with_rng(rng)?;
        self.last_name.regenerate_in_place_with_rng(rng)
    }

    /// rerolls narrative flavour fields without touching identity/world anchors.
    fn reroll_omen_and_relic_in_place_with_rng<R: Rng + ?Sized>(
        &mut self,
        rng: &mut R,
    ) -> Result<(), GenerationError> {
        self.omen.regenerate_in_place_with_rng(rng)?;
        self.relic.regenerate_in_place_with_rng(rng)
    }
}

impl Generation for DemoGeneration {
    type Loader = DemoLoader;
    type CompiledState = ();

    /// builds field-local candidates from corpus data and returns a ready generation object.
    fn compile(corpus: AethelCorpus<Self::Loader>) -> Result<Self, GenerationError> {
        let mut title = GeneratedField::pending(String::new());
        title.set_candidates(
            "title",
            generated_field_builder(&corpus, "identity", "titles", |loader| {
                loader.identity.titles.clone()
            }),
        );

        let mut name = GeneratedField::pending(String::new());
        name.set_candidates(
            "name",
            generated_field_builder(&corpus, "identity", "first_names", |loader| {
                loader.identity.first_names.clone()
            }),
        );

        let mut middle_name = GeneratedField::pending(String::new());
        middle_name.set_candidates(
            "middle_name",
            generated_field_builder(&corpus, "identity", "middle_names", |loader| {
                loader.identity.middle_names.clone()
            }),
        );

        let mut last_name = GeneratedField::pending(String::new());
        last_name.set_candidates(
            "last_name",
            generated_field_builder(&corpus, "identity", "last_names", |loader| {
                loader.identity.last_names.clone()
            }),
        );

        let mut suffix = GeneratedField::pending(String::new());
        suffix.set_candidates(
            "suffix",
            generated_field_builder(&corpus, "identity", "suffixes", |loader| {
                loader.identity.suffixes.clone()
            }),
        );

        let mut epithet = GeneratedField::pending(String::new());
        epithet.set_candidates(
            "epithet",
            generated_field_builder(&corpus, "lore", "epithets", |loader| {
                loader.lore.epithets.clone()
            }),
        );

        let mut dynasty = GeneratedField::pending(String::new());
        dynasty.set_candidates(
            "dynasty",
            generated_field_builder(&corpus, "lore", "dynasties", |loader| {
                loader.lore.dynasties.clone()
            }),
        );

        let mut era = GeneratedField::pending(String::new());
        era.set_candidates(
            "era",
            generated_field_builder(&corpus, "lore", "eras", |loader| loader.lore.eras.clone()),
        );

        let mut vocation = GeneratedField::pending(String::new());
        vocation.set_candidates(
            "vocation",
            generated_field_builder(&corpus, "world", "vocations", |loader| {
                loader.world.vocations.clone()
            }),
        );

        let mut homeland = GeneratedField::pending(String::new());
        homeland.set_candidates(
            "homeland",
            generated_field_builder(&corpus, "world", "homelands", |loader| {
                loader.world.homelands.clone()
            }),
        );

        let mut relic = GeneratedField::pending(String::new());
        relic.set_candidates(
            "relic",
            generated_field_builder(&corpus, "world", "relics", |loader| {
                loader.world.relics.clone()
            }),
        );

        let mut omen = GeneratedField::pending(String::new());
        omen.set_candidates(
            "omen",
            generated_field_builder(&corpus, "lore", "omens", |loader| loader.lore.omens.clone()),
        );

        Ok(Self {
            title,
            name,
            middle_name,
            last_name,
            suffix,
            epithet,
            dynasty,
            era,
            vocation,
            homeland,
            relic,
            omen,
        })
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // the document set is intentionally broad to make provenance output varied.
    let documents = vec![
        AethelDoc {
            header: AethelDocHeader {
                title: "highland annals".to_string(),
                target: "demo".to_string(),
                desc: Some("old houses of the northern spine".to_string()),
                author: Some("scribe anwyn".to_string()),
                version: Some("2.1".to_string()),
            },
            data: DemoLoader {
                identity: IdentityValues {
                    titles: vec!["archon".to_string(), "warden".to_string()],
                    first_names: vec!["alder".to_string(), "brynn".to_string(), "cael".to_string()],
                    middle_names: vec![
                        "vale".to_string(),
                        "thorn".to_string(),
                        "rowan".to_string(),
                    ],
                    last_names: vec!["blackmere".to_string(), "stormglass".to_string()],
                    suffixes: vec!["of dawn".to_string(), "of stone".to_string()],
                },
                lore: LoreValues {
                    epithets: vec!["the oathbound".to_string(), "the ember-witness".to_string()],
                    dynasties: vec!["house gryph".to_string(), "house cindervale".to_string()],
                    eras: vec![
                        "third lantern age".to_string(),
                        "fracture reign".to_string(),
                    ],
                    omens: vec![
                        "ashen rain at noon".to_string(),
                        "silver crows in spiral".to_string(),
                    ],
                },
                world: WorldValues {
                    vocations: vec![
                        "border cartographer".to_string(),
                        "ritual engineer".to_string(),
                    ],
                    homelands: vec!["northreach terraces".to_string(), "glassfjord".to_string()],
                    relics: vec![
                        "compass of nine tides".to_string(),
                        "obsidian treaty seal".to_string(),
                    ],
                },
            },
        },
        AethelDoc {
            header: AethelDocHeader {
                title: "river court ledger".to_string(),
                target: "demo".to_string(),
                desc: Some("bloodlines and civic vows of the southern deltas".to_string()),
                author: Some("clerk mirel".to_string()),
                version: Some("1.4".to_string()),
            },
            data: DemoLoader {
                identity: IdentityValues {
                    titles: vec!["magistrate".to_string(), "legate".to_string()],
                    first_names: vec![
                        "darian".to_string(),
                        "elowen".to_string(),
                        "fenn".to_string(),
                    ],
                    middle_names: vec![
                        "quill".to_string(),
                        "marsh".to_string(),
                        "tide".to_string(),
                    ],
                    last_names: vec!["reedmarch".to_string(), "halcyon".to_string()],
                    suffixes: vec!["of mist".to_string(), "of flame".to_string()],
                },
                lore: LoreValues {
                    epithets: vec![
                        "the flood-scribe".to_string(),
                        "the gate-keeper".to_string(),
                    ],
                    dynasties: vec!["house delta".to_string(), "house palewater".to_string()],
                    eras: vec!["concord cycle".to_string(), "river partition".to_string()],
                    omens: vec![
                        "mirrorfish breaching at dusk".to_string(),
                        "violet foam on the weir".to_string(),
                    ],
                },
                world: WorldValues {
                    vocations: vec![
                        "charter advocate".to_string(),
                        "bridge tactician".to_string(),
                    ],
                    homelands: vec!["low delta bastions".to_string(), "brinecourt".to_string()],
                    relics: vec![
                        "salt-etched decree blade".to_string(),
                        "ledger of drowned crowns".to_string(),
                    ],
                },
            },
        },
        AethelDoc {
            header: AethelDocHeader {
                title: "sunken observatory archive".to_string(),
                target: "demo".to_string(),
                desc: Some("deep vault records from the western trench".to_string()),
                author: Some("custodian irys".to_string()),
                version: Some("0.9".to_string()),
            },
            data: DemoLoader {
                identity: IdentityValues {
                    titles: vec!["prefect".to_string(), "oracle".to_string()],
                    first_names: vec!["galen".to_string(), "heli".to_string(), "iora".to_string()],
                    middle_names: vec![
                        "hollow".to_string(),
                        "echo".to_string(),
                        "lumen".to_string(),
                    ],
                    last_names: vec!["deepwake".to_string(), "starbreak".to_string()],
                    suffixes: vec!["of tides".to_string(), "of embers".to_string()],
                },
                lore: LoreValues {
                    epithets: vec![
                        "the trench-eyed".to_string(),
                        "the patient flame".to_string(),
                    ],
                    dynasties: vec!["house meridian".to_string(), "house undertorch".to_string()],
                    eras: vec![
                        "seventh eclipse".to_string(),
                        "bell of red winter".to_string(),
                    ],
                    omens: vec![
                        "a silent aurora beneath the surf".to_string(),
                        "ink tides rising uphill".to_string(),
                    ],
                },
                world: WorldValues {
                    vocations: vec![
                        "abyssal archivist".to_string(),
                        "signal alchemist".to_string(),
                    ],
                    homelands: vec![
                        "sunken meridian".to_string(),
                        "the brass trench".to_string(),
                    ],
                    relics: vec![
                        "astrolabe of drowned noon".to_string(),
                        "vacuum pearl reliquary".to_string(),
                    ],
                },
            },
        },
        AethelDoc {
            header: AethelDocHeader {
                title: "highland annals 2".to_string(),
                target: "demo".to_string(),
                desc: Some("old houses of the northern spine".to_string()),
                author: Some("scribe anwyn".to_string()),
                version: Some("2.1".to_string()),
            },
            data: DemoLoader {
                identity: IdentityValues {
                    titles: vec!["archon".to_string(), "warden".to_string()],
                    first_names: vec!["alder".to_string(), "brynn".to_string(), "cael".to_string()],
                    middle_names: vec![
                        "vale".to_string(),
                        "thorn".to_string(),
                        "rowan".to_string(),
                    ],
                    last_names: vec!["blackmere".to_string(), "stormglass".to_string()],
                    suffixes: vec!["of dawn".to_string(), "of stone".to_string()],
                },
                lore: LoreValues {
                    epithets: vec!["the oathbound".to_string(), "the ember-witness".to_string()],
                    dynasties: vec!["house gryph".to_string(), "house cindervale".to_string()],
                    eras: vec![
                        "third lantern age".to_string(),
                        "fracture reign".to_string(),
                    ],
                    omens: vec![
                        "ashen rain at noon".to_string(),
                        "silver crows in spiral".to_string(),
                    ],
                },
                world: WorldValues {
                    vocations: vec![
                        "border cartographer".to_string(),
                        "ritual engineer".to_string(),
                    ],
                    homelands: vec!["northreach terraces".to_string(), "glassfjord".to_string()],
                    relics: vec![
                        "compass of nine tides".to_string(),
                        "obsidian treaty seal".to_string(),
                    ],
                },
            },
        },
        AethelDoc {
            header: AethelDocHeader {
                title: "river court ledger 2".to_string(),
                target: "demo".to_string(),
                desc: Some("bloodlines and civic vows of the southern deltas".to_string()),
                author: Some("clerk mirel".to_string()),
                version: Some("1.4".to_string()),
            },
            data: DemoLoader {
                identity: IdentityValues {
                    titles: vec!["magistrate".to_string(), "legate".to_string()],
                    first_names: vec![
                        "darian".to_string(),
                        "elowen".to_string(),
                        "fenn".to_string(),
                    ],
                    middle_names: vec![
                        "quill".to_string(),
                        "marsh".to_string(),
                        "tide".to_string(),
                    ],
                    last_names: vec!["reedmarch".to_string(), "halcyon".to_string()],
                    suffixes: vec!["of mist".to_string(), "of flame".to_string()],
                },
                lore: LoreValues {
                    epithets: vec![
                        "the flood-scribe".to_string(),
                        "the gate-keeper".to_string(),
                    ],
                    dynasties: vec!["house delta".to_string(), "house palewater".to_string()],
                    eras: vec!["concord cycle".to_string(), "river partition".to_string()],
                    omens: vec![
                        "mirrorfish breaching at dusk".to_string(),
                        "violet foam on the weir".to_string(),
                    ],
                },
                world: WorldValues {
                    vocations: vec![
                        "charter advocate".to_string(),
                        "bridge tactician".to_string(),
                    ],
                    homelands: vec!["low delta bastions".to_string(), "brinecourt".to_string()],
                    relics: vec![
                        "salt-etched decree blade".to_string(),
                        "ledger of drowned crowns".to_string(),
                    ],
                },
            },
        },
        AethelDoc {
            header: AethelDocHeader {
                title: "sunken observatory archive 2".to_string(),
                target: "demo".to_string(),
                desc: Some("deep vault records from the western trench".to_string()),
                author: Some("custodian irys".to_string()),
                version: Some("0.9".to_string()),
            },
            data: DemoLoader {
                identity: IdentityValues {
                    titles: vec!["prefect".to_string(), "oracle".to_string()],
                    first_names: vec!["galen".to_string(), "heli".to_string(), "iora".to_string()],
                    middle_names: vec![
                        "hollow".to_string(),
                        "echo".to_string(),
                        "lumen".to_string(),
                    ],
                    last_names: vec!["deepwake".to_string(), "starbreak".to_string()],
                    suffixes: vec!["of tides".to_string(), "of embers".to_string()],
                },
                lore: LoreValues {
                    epithets: vec![
                        "the trench-eyed".to_string(),
                        "the patient flame".to_string(),
                    ],
                    dynasties: vec!["house meridian".to_string(), "house undertorch".to_string()],
                    eras: vec![
                        "seventh eclipse".to_string(),
                        "bell of red winter".to_string(),
                    ],
                    omens: vec![
                        "a silent aurora beneath the surf".to_string(),
                        "ink tides rising uphill".to_string(),
                    ],
                },
                world: WorldValues {
                    vocations: vec![
                        "abyssal archivist".to_string(),
                        "signal alchemist".to_string(),
                    ],
                    homelands: vec![
                        "sunken meridian".to_string(),
                        "the brass trench".to_string(),
                    ],
                    relics: vec![
                        "astrolabe of drowned noon".to_string(),
                        "vacuum pearl reliquary".to_string(),
                    ],
                },
            },
        },
        AethelDoc {
            header: AethelDocHeader {
                title: "highland annals 3".to_string(),
                target: "demo".to_string(),
                desc: Some("old houses of the northern spine".to_string()),
                author: Some("scribe anwyn".to_string()),
                version: Some("2.1".to_string()),
            },
            data: DemoLoader {
                identity: IdentityValues {
                    titles: vec!["archon".to_string(), "warden".to_string()],
                    first_names: vec!["alder".to_string(), "brynn".to_string(), "cael".to_string()],
                    middle_names: vec![
                        "vale".to_string(),
                        "thorn".to_string(),
                        "rowan".to_string(),
                    ],
                    last_names: vec!["blackmere".to_string(), "stormglass".to_string()],
                    suffixes: vec!["of dawn".to_string(), "of stone".to_string()],
                },
                lore: LoreValues {
                    epithets: vec!["the oathbound".to_string(), "the ember-witness".to_string()],
                    dynasties: vec!["house gryph".to_string(), "house cindervale".to_string()],
                    eras: vec![
                        "third lantern age".to_string(),
                        "fracture reign".to_string(),
                    ],
                    omens: vec![
                        "ashen rain at noon".to_string(),
                        "silver crows in spiral".to_string(),
                    ],
                },
                world: WorldValues {
                    vocations: vec![
                        "border cartographer".to_string(),
                        "ritual engineer".to_string(),
                    ],
                    homelands: vec!["northreach terraces".to_string(), "glassfjord".to_string()],
                    relics: vec![
                        "compass of nine tides".to_string(),
                        "obsidian treaty seal".to_string(),
                    ],
                },
            },
        },
        AethelDoc {
            header: AethelDocHeader {
                title: "river court ledger 3".to_string(),
                target: "demo".to_string(),
                desc: Some("bloodlines and civic vows of the southern deltas".to_string()),
                author: Some("clerk mirel".to_string()),
                version: Some("1.4".to_string()),
            },
            data: DemoLoader {
                identity: IdentityValues {
                    titles: vec!["magistrate".to_string(), "legate".to_string()],
                    first_names: vec![
                        "darian".to_string(),
                        "elowen".to_string(),
                        "fenn".to_string(),
                    ],
                    middle_names: vec![
                        "quill".to_string(),
                        "marsh".to_string(),
                        "tide".to_string(),
                    ],
                    last_names: vec!["reedmarch".to_string(), "halcyon".to_string()],
                    suffixes: vec!["of mist".to_string(), "of flame".to_string()],
                },
                lore: LoreValues {
                    epithets: vec![
                        "the flood-scribe".to_string(),
                        "the gate-keeper".to_string(),
                    ],
                    dynasties: vec!["house delta".to_string(), "house palewater".to_string()],
                    eras: vec!["concord cycle".to_string(), "river partition".to_string()],
                    omens: vec![
                        "mirrorfish breaching at dusk".to_string(),
                        "violet foam on the weir".to_string(),
                    ],
                },
                world: WorldValues {
                    vocations: vec![
                        "charter advocate".to_string(),
                        "bridge tactician".to_string(),
                    ],
                    homelands: vec!["low delta bastions".to_string(), "brinecourt".to_string()],
                    relics: vec![
                        "salt-etched decree blade".to_string(),
                        "ledger of drowned crowns".to_string(),
                    ],
                },
            },
        },
        AethelDoc {
            header: AethelDocHeader {
                title: "sunken observatory archive 3".to_string(),
                target: "demo".to_string(),
                desc: Some("deep vault records from the western trench".to_string()),
                author: Some("custodian irys".to_string()),
                version: Some("0.9".to_string()),
            },
            data: DemoLoader {
                identity: IdentityValues {
                    titles: vec!["prefect".to_string(), "oracle".to_string()],
                    first_names: vec!["galen".to_string(), "heli".to_string(), "iora".to_string()],
                    middle_names: vec![
                        "hollow".to_string(),
                        "echo".to_string(),
                        "lumen".to_string(),
                    ],
                    last_names: vec!["deepwake".to_string(), "starbreak".to_string()],
                    suffixes: vec!["of tides".to_string(), "of embers".to_string()],
                },
                lore: LoreValues {
                    epithets: vec![
                        "the trench-eyed".to_string(),
                        "the patient flame".to_string(),
                    ],
                    dynasties: vec!["house meridian".to_string(), "house undertorch".to_string()],
                    eras: vec![
                        "seventh eclipse".to_string(),
                        "bell of red winter".to_string(),
                    ],
                    omens: vec![
                        "a silent aurora beneath the surf".to_string(),
                        "ink tides rising uphill".to_string(),
                    ],
                },
                world: WorldValues {
                    vocations: vec![
                        "abyssal archivist".to_string(),
                        "signal alchemist".to_string(),
                    ],
                    homelands: vec![
                        "sunken meridian".to_string(),
                        "the brass trench".to_string(),
                    ],
                    relics: vec![
                        "astrolabe of drowned noon".to_string(),
                        "vacuum pearl reliquary".to_string(),
                    ],
                },
            },
        },
        AethelDoc {
            header: AethelDocHeader {
                title: "highland annals 4".to_string(),
                target: "demo".to_string(),
                desc: Some("old houses of the northern spine".to_string()),
                author: Some("scribe anwyn".to_string()),
                version: Some("2.1".to_string()),
            },
            data: DemoLoader {
                identity: IdentityValues {
                    titles: vec!["archon".to_string(), "warden".to_string()],
                    first_names: vec!["alder".to_string(), "brynn".to_string(), "cael".to_string()],
                    middle_names: vec![
                        "vale".to_string(),
                        "thorn".to_string(),
                        "rowan".to_string(),
                    ],
                    last_names: vec!["blackmere".to_string(), "stormglass".to_string()],
                    suffixes: vec!["of dawn".to_string(), "of stone".to_string()],
                },
                lore: LoreValues {
                    epithets: vec!["the oathbound".to_string(), "the ember-witness".to_string()],
                    dynasties: vec!["house gryph".to_string(), "house cindervale".to_string()],
                    eras: vec![
                        "third lantern age".to_string(),
                        "fracture reign".to_string(),
                    ],
                    omens: vec![
                        "ashen rain at noon".to_string(),
                        "silver crows in spiral".to_string(),
                    ],
                },
                world: WorldValues {
                    vocations: vec![
                        "border cartographer".to_string(),
                        "ritual engineer".to_string(),
                    ],
                    homelands: vec!["northreach terraces".to_string(), "glassfjord".to_string()],
                    relics: vec![
                        "compass of nine tides".to_string(),
                        "obsidian treaty seal".to_string(),
                    ],
                },
            },
        },
        AethelDoc {
            header: AethelDocHeader {
                title: "river court ledger 4".to_string(),
                target: "demo".to_string(),
                desc: Some("bloodlines and civic vows of the southern deltas".to_string()),
                author: Some("clerk mirel".to_string()),
                version: Some("1.4".to_string()),
            },
            data: DemoLoader {
                identity: IdentityValues {
                    titles: vec!["magistrate".to_string(), "legate".to_string()],
                    first_names: vec![
                        "darian".to_string(),
                        "elowen".to_string(),
                        "fenn".to_string(),
                    ],
                    middle_names: vec![
                        "quill".to_string(),
                        "marsh".to_string(),
                        "tide".to_string(),
                    ],
                    last_names: vec!["reedmarch".to_string(), "halcyon".to_string()],
                    suffixes: vec!["of mist".to_string(), "of flame".to_string()],
                },
                lore: LoreValues {
                    epithets: vec![
                        "the flood-scribe".to_string(),
                        "the gate-keeper".to_string(),
                    ],
                    dynasties: vec!["house delta".to_string(), "house palewater".to_string()],
                    eras: vec!["concord cycle".to_string(), "river partition".to_string()],
                    omens: vec![
                        "mirrorfish breaching at dusk".to_string(),
                        "violet foam on the weir".to_string(),
                    ],
                },
                world: WorldValues {
                    vocations: vec![
                        "charter advocate".to_string(),
                        "bridge tactician".to_string(),
                    ],
                    homelands: vec!["low delta bastions".to_string(), "brinecourt".to_string()],
                    relics: vec![
                        "salt-etched decree blade".to_string(),
                        "ledger of drowned crowns".to_string(),
                    ],
                },
            },
        },
        AethelDoc {
            header: AethelDocHeader {
                title: "sunken observatory archive 4".to_string(),
                target: "demo".to_string(),
                desc: Some("deep vault records from the western trench".to_string()),
                author: Some("custodian irys".to_string()),
                version: Some("0.9".to_string()),
            },
            data: DemoLoader {
                identity: IdentityValues {
                    titles: vec!["prefect".to_string(), "oracle".to_string()],
                    first_names: vec!["galen".to_string(), "heli".to_string(), "iora".to_string()],
                    middle_names: vec![
                        "hollow".to_string(),
                        "echo".to_string(),
                        "lumen".to_string(),
                    ],
                    last_names: vec!["deepwake".to_string(), "starbreak".to_string()],
                    suffixes: vec!["of tides".to_string(), "of embers".to_string()],
                },
                lore: LoreValues {
                    epithets: vec![
                        "the trench-eyed".to_string(),
                        "the patient flame".to_string(),
                    ],
                    dynasties: vec!["house meridian".to_string(), "house undertorch".to_string()],
                    eras: vec![
                        "seventh eclipse".to_string(),
                        "bell of red winter".to_string(),
                    ],
                    omens: vec![
                        "a silent aurora beneath the surf".to_string(),
                        "ink tides rising uphill".to_string(),
                    ],
                },
                world: WorldValues {
                    vocations: vec![
                        "abyssal archivist".to_string(),
                        "signal alchemist".to_string(),
                    ],
                    homelands: vec![
                        "sunken meridian".to_string(),
                        "the brass trench".to_string(),
                    ],
                    relics: vec![
                        "astrolabe of drowned noon".to_string(),
                        "vacuum pearl reliquary".to_string(),
                    ],
                },
            },
        },
    ];

    let source_docs = SourceAethelDoc::from_aetheldocs(documents)?;
    let generation = DemoGeneration::from_documents(source_docs)?;

    // this generated profile is reproducible and useful for snapshots/tests.
    let mut seeded_rng = rand::rngs::StdRng::seed_from_u64(42);
    let deterministic = generation.generate_with_rng(&mut seeded_rng)?;
    println!("seeded profile:");
    print_profile(&deterministic);

    // this profile is for interactive exploration with selective rerolls.
    let mut sampled = generation.generate()?;
    // these values are expected to remain stable across selective reroll operations.
    let locked_suffix = sampled.suffix().value().clone();
    let locked_homeland = sampled.homeland().value().clone();
    let locked_dynasty = sampled.dynasty().value().clone();

    println!("\nfull profile before rerolls:");
    print_profile(&sampled);

    // deep provenance output demonstrates source-level tracing per field.
    println!("\nprovenance deep-dive (name components and lore anchors):");
    print_field_provenance("title", sampled.title());
    print_field_provenance("first_name", sampled.name());
    print_field_provenance("middle_name", sampled.middle_name());
    print_field_provenance("last_name", sampled.last_name());
    print_field_provenance("suffix", sampled.suffix());
    print_field_provenance("epithet", sampled.epithet());
    print_field_provenance("omen", sampled.omen());
    print_field_provenance("relic", sampled.relic());

    // reroll only selected subsets to demonstrate lockable profile composition.
    let mut identity_reroll_rng = rand::rngs::StdRng::seed_from_u64(7);
    sampled.reroll_name_components_in_place_with_rng(&mut identity_reroll_rng)?;

    let mut story_reroll_rng = rand::rngs::StdRng::seed_from_u64(99);
    sampled.reroll_omen_and_relic_in_place_with_rng(&mut story_reroll_rng)?;

    println!("\nprofile after selective rerolls:");
    print_profile(&sampled);

    println!("\nlocked field checks:");
    println!("suffix before: {locked_suffix}");
    println!("suffix after:  {}", sampled.suffix().value());
    println!("homeland before: {locked_homeland}");
    println!("homeland after:  {}", sampled.homeland().value());
    println!("dynasty before: {locked_dynasty}");
    println!("dynasty after:  {}", sampled.dynasty().value());

    assert_eq!(sampled.suffix().value(), &locked_suffix);
    assert_eq!(sampled.homeland().value(), &locked_homeland);
    assert_eq!(sampled.dynasty().value(), &locked_dynasty);

    // by-value regenerate: sampled is unchanged.
    let name_before_value_regen = sampled.name().value().clone();
    let regenerated_name = sampled.name().regenerate()?;
    println!("\nname before by-value regenerate: {name_before_value_regen}");
    println!("name returned by regenerate: {}", regenerated_name.value());
    println!("name still on sampled: {}", sampled.name().value());

    // in-place regenerate: sampled.name is mutated directly.
    let name_before_in_place = sampled.name().value().clone();
    sampled.name.regenerate_in_place()?;
    println!("name before in-place regenerate: {name_before_in_place}");
    println!("name after in-place regenerate: {}", sampled.name().value());
    dbg!("\nall name() options: {}", sampled.name().all_options().unwrap());

    Ok(())
}

fn print_profile(generation: &DemoGeneration) {
    println!("full name: {}", generation.full_name());
    println!("epithet: {}", generation.epithet().value());
    println!(
        "role: {} of {}",
        generation.vocation().value(),
        generation.homeland().value()
    );
    println!("dynasty: {}", generation.dynasty().value());
    println!("era: {}", generation.era().value());
    println!("relic: {}", generation.relic().value());
    println!("omen: {}", generation.omen().value());
}

/// prints value-level provenance for one generated field.
fn print_field_provenance(label: &str, field: &GeneratedField<String>) {
    println!("\n[{label}] {}", field.value());
    println!("source ids: {:?}", field.source_ids());
    for source_ref in field.source_refs() {
        println!(
            "  source: {} | {}.{}  ",
            source_ref.source_name, source_ref.section, source_ref.field,
        );
    }

}
