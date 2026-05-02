use serde::Deserialize;

use crate::loader::{LoaderError, AethelDoc, Target, TargetedLoader};

#[derive(Deserialize, Debug, Clone)]
/// weapon body sections loaded from a weapon-target toml document.
pub struct WeaponLoader {
    /// optional name generation section.
    pub name: Option<WeaponNameSection>,
    #[serde(rename = "type")]
    /// optional weapon type section.
    pub weapon_type: Option<WeaponTypeSection>,
    /// optional quality section.
    pub qualities: Option<WeaponQualitiesSection>,
    /// optional lore section.
    pub lore: Option<WeaponLoreSection>,
    /// optional visual description section.
    pub visuals: Option<WeaponVisualSection>,
}

#[derive(Deserialize, Debug, Clone)]
/// values used to compose generated weapon names.
pub struct WeaponNameSection {
    /// optional name prefixes.
    pub prefix: Option<Vec<String>>,
    /// optional name suffixes.
    pub suffix: Option<Vec<String>>,
    /// optional primitive fragments for synthetic names.
    pub primitives: Option<Vec<String>>,
}

#[derive(Deserialize, Debug, Clone)]
/// available weapon base types.
pub struct WeaponTypeSection {
    #[serde(rename = "type")]
    /// optional weapon type list.
    pub types: Option<Vec<String>>,
}

#[derive(Deserialize, Debug, Clone)]
/// quality-related attributes for generated weapons.
pub struct WeaponQualitiesSection {
    /// optional rarity labels.
    pub rarity: Option<Vec<String>>,
    /// optional condition labels.
    pub condition: Option<Vec<String>>,
}

#[derive(Deserialize, Debug, Clone)]
/// lore templates and lore source values.
pub struct WeaponLoreSection {
    /// optional creator fragments.
    pub creators: Option<Vec<String>>,
    /// optional deed fragments.
    pub deeds: Option<Vec<String>>,
    /// optional quirk fragments.
    pub quirks: Option<Vec<String>>,
    /// optional lore sentence templates.
    pub templates: Option<Vec<String>>,
}

#[derive(Deserialize, Debug, Clone)]
/// visual templates and visual source values.
pub struct WeaponVisualSection {
    /// optional material options.
    pub materials: Option<Vec<String>>,
    /// optional colour options.
    pub colours: Option<Vec<String>>,
    /// optional accent options.
    pub accents: Option<Vec<String>>,
    /// optional visual feature options.
    pub features: Option<Vec<String>>,
    /// optional visual sentence templates.
    pub templates: Option<Vec<String>>,
}

impl TargetedLoader for WeaponLoader {
    const TARGET: Target = Target::Weapon;

    /// load and merge weapon files using first-seen-order dedup for list values.
    ///
    /// every file is parsed with target validation, then merged into an empty
    /// accumulator so duplicates are removed even when they appear in the first file.
    fn merge_from_files(paths: &[&str]) -> Result<AethelDoc<Self>, LoaderError> {
        let first_path = paths.first().ok_or_else(|| {
            LoaderError::ReadError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "at least one path is required for merge",
            ))
        })?;

        let first_loaded = Self::from_file(first_path)?;
        let mut merged = AethelDoc {
            header: first_loaded.header,
            data: WeaponLoader {
                name: None,
                weapon_type: None,
                qualities: None,
                lore: None,
                visuals: None,
            },
        };

        for path in paths {
            let next = Self::from_file(path)?;
            merge_weapon_loader(&mut merged.data, &next.data);
        }

        Ok(merged)
    }
}

impl WeaponLoader {
    /// load a weapon toml file and validate its target header.
    pub fn from_file(path: &str) -> Result<AethelDoc<Self>, LoaderError> {
        <Self as TargetedLoader>::from_file(path)
    }
}

/// merge top-level weapon sections from `incoming` into `base`.
fn merge_weapon_loader(base: &mut WeaponLoader, incoming: &WeaponLoader) {
    merge_name_section(&mut base.name, &incoming.name);
    merge_type_section(&mut base.weapon_type, &incoming.weapon_type);
    merge_qualities_section(&mut base.qualities, &incoming.qualities);
    merge_lore_section(&mut base.lore, &incoming.lore);
    merge_visuals_section(&mut base.visuals, &incoming.visuals);
}

/// merge the optional `name` section.
fn merge_name_section(base: &mut Option<WeaponNameSection>, incoming: &Option<WeaponNameSection>) {
    if let Some(incoming) = incoming {
        let section = base.get_or_insert_with(|| WeaponNameSection {
            prefix: None,
            suffix: None,
            primitives: None,
        });
        merge_option_vec(&mut section.prefix, &incoming.prefix);
        merge_option_vec(&mut section.suffix, &incoming.suffix);
        merge_option_vec(&mut section.primitives, &incoming.primitives);
    }
}

/// merge the optional `type` section.
fn merge_type_section(base: &mut Option<WeaponTypeSection>, incoming: &Option<WeaponTypeSection>) {
    if let Some(incoming) = incoming {
        let section = base.get_or_insert_with(|| WeaponTypeSection { types: None });
        merge_option_vec(&mut section.types, &incoming.types);
    }
}

/// merge the optional `qualities` section.
fn merge_qualities_section(
    base: &mut Option<WeaponQualitiesSection>,
    incoming: &Option<WeaponQualitiesSection>,
) {
    if let Some(incoming) = incoming {
        let section = base.get_or_insert_with(|| WeaponQualitiesSection {
            rarity: None,
            condition: None,
        });
        merge_option_vec(&mut section.rarity, &incoming.rarity);
        merge_option_vec(&mut section.condition, &incoming.condition);
    }
}

/// merge the optional `lore` section.
fn merge_lore_section(base: &mut Option<WeaponLoreSection>, incoming: &Option<WeaponLoreSection>) {
    if let Some(incoming) = incoming {
        let section = base.get_or_insert_with(|| WeaponLoreSection {
            creators: None,
            deeds: None,
            quirks: None,
            templates: None,
        });
        merge_option_vec(&mut section.creators, &incoming.creators);
        merge_option_vec(&mut section.deeds, &incoming.deeds);
        merge_option_vec(&mut section.quirks, &incoming.quirks);
        merge_option_vec(&mut section.templates, &incoming.templates);
    }
}

/// merge the optional `visuals` section.
fn merge_visuals_section(
    base: &mut Option<WeaponVisualSection>,
    incoming: &Option<WeaponVisualSection>,
) {
    if let Some(incoming) = incoming {
        let section = base.get_or_insert_with(|| WeaponVisualSection {
            materials: None,
            colours: None,
            accents: None,
            features: None,
            templates: None,
        });
        merge_option_vec(&mut section.materials, &incoming.materials);
        merge_option_vec(&mut section.colours, &incoming.colours);
        merge_option_vec(&mut section.accents, &incoming.accents);
        merge_option_vec(&mut section.features, &incoming.features);
        merge_option_vec(&mut section.templates, &incoming.templates);
    }
}

/// merge two optional string lists with first-seen-order deduplication.
fn merge_option_vec(base: &mut Option<Vec<String>>, incoming: &Option<Vec<String>>) {
    if let Some(incoming_values) = incoming {
        let values = base.get_or_insert_with(Vec::new);
        for value in incoming_values {
            if !values.contains(value) {
                values.push(value.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weapon_loader_deserializes_data_sections() {
        let loaded = WeaponLoader::from_file("data/weapon_test_data.toml").unwrap();

        assert_eq!(loaded.header.target, Target::Weapon);
        assert!(!loaded.data.name.unwrap().prefix.unwrap().is_empty());
        assert!(!loaded.data.weapon_type.unwrap().types.unwrap().is_empty());
        assert!(!loaded.data.lore.unwrap().templates.unwrap().is_empty());
        assert!(!loaded.data.visuals.unwrap().templates.unwrap().is_empty());
    }

    #[test]
    fn test_weapon_loader_allows_missing_body_sections() {
        let content = r#"
[header]
name = "partial weapon set"
target = "weapon"
"#;

        let loaded = toml::from_str::<AethelDoc<WeaponLoader>>(content);

        assert!(loaded.is_ok());

        let loaded = loaded.unwrap();
        assert!(loaded.data.name.is_none());
        assert!(loaded.data.weapon_type.is_none());
        assert!(loaded.data.qualities.is_none());
        assert!(loaded.data.lore.is_none());
        assert!(loaded.data.visuals.is_none());
    }

    #[test]
    fn test_weapon_loader_allows_missing_fields_in_existing_section() {
        let content = r#"
[header]
name = "partial fields"
target = "weapon"

[name]
prefix = ["iron"]
"#;

        let loaded = toml::from_str::<AethelDoc<WeaponLoader>>(content);

        assert!(loaded.is_ok());

        let loaded = loaded.unwrap();
        let name = loaded.data.name.unwrap();
        assert_eq!(name.prefix.unwrap(), vec!["iron".to_string()]);
        assert!(name.suffix.is_none());
        assert!(name.primitives.is_none());
    }

    #[test]
    fn test_weapon_loader_merges_split_fixtures_and_deduplicates_values() {
        let paths = [
            "data/weapon_merge_part_1.toml",
            "data/weapon_merge_part_2.toml",
            "data/weapon_merge_part_3.toml",
            "data/weapon_merge_part_4.toml",
        ];

        let loaded = WeaponLoader::merge_from_files(&paths).unwrap();

        assert_eq!(loaded.header.target, Target::Weapon);

        let name = loaded.data.name.unwrap();
        assert_eq!(name.prefix.unwrap(), vec!["Iron", "Steel"]);
        assert_eq!(name.suffix.unwrap(), vec!["of the Dawn", "of the Dusk"]);
        assert_eq!(name.primitives.unwrap(), vec!["ka", "li"]);

        let weapon_type = loaded.data.weapon_type.unwrap();
        assert_eq!(weapon_type.types.unwrap(), vec!["longsword", "rapier"]);

        let qualities = loaded.data.qualities.unwrap();
        assert_eq!(qualities.rarity.unwrap(), vec!["Common", "Rare"]);
        assert_eq!(qualities.condition.unwrap(), vec!["Worn", "Pristine"]);

        let lore = loaded.data.lore.unwrap();
        assert_eq!(lore.templates.unwrap().len(), 1);

        let visuals = loaded.data.visuals.unwrap();
        assert_eq!(visuals.materials.unwrap(), vec!["brushed steel", "oxidized copper"]);
    }
}
