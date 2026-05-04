//! weapon-target loader schema and parsing helpers.

use serde::Deserialize;

use crate::loader::{AethelDoc, LoaderError, Target, TargetedLoader};

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
}

impl WeaponLoader {
    /// load a weapon toml file and validate its target header.
    pub fn from_file(path: &str) -> Result<AethelDoc<Self>, LoaderError> {
        <Self as TargetedLoader>::from_file(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::merge::{merge_from_files, MergedAethelDoc};

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

        let merged_docs = merge_from_files(&paths).unwrap();
        assert_eq!(merged_docs.len(), 1);
        let MergedAethelDoc::Weapon(loaded) = &merged_docs[0];

        assert_eq!(loaded.target, Target::Weapon);
        assert_eq!(loaded.documents.len(), 4);
        assert_eq!(loaded.documents[0].header.name, "weapon merge fixture part 1");
        assert_eq!(loaded.documents[1].header.name, "weapon merge fixture part 2");
        assert_eq!(loaded.documents[2].header.name, "weapon merge fixture part 3");
        assert_eq!(loaded.documents[3].header.name, "weapon merge fixture part 4");

        assert!(loaded
            .documents
            .iter()
            .all(|document| document.header.target == Target::Weapon));
    }
}
