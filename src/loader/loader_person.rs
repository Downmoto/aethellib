//! person-target loader schema and parsing helpers.

use serde::Deserialize;

use crate::loader::{Target, TargetedLoader};

#[derive(Deserialize, Debug, Clone)]
/// person body sections loaded from a person-target toml document.
pub struct PersonLoader {
    /// optional name generation section.
    pub name: Option<PersonNameSection>,
}

#[derive(Deserialize, Debug, Clone)]
/// primitive fragments used to compose generated person names.
pub struct PersonNameSection {
    /// optional first-name primitives.
    pub first: Option<Vec<String>>,
    /// optional middle-name primitives.
    pub middle: Option<Vec<String>>,
    /// optional last-name primitives.
    pub last: Option<Vec<String>>,
}

impl TargetedLoader for PersonLoader {
    const TARGET: Target = Target::Person;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::AethelDoc;

    #[test]
    fn test_person_loader_deserializes_data_sections() {
        let loaded = PersonLoader::from_file("data/person_test_data.toml").unwrap();

        assert_eq!(loaded.header.target, Target::Person);
        assert!(!loaded.data.name.unwrap().first.unwrap().is_empty());
    }

    #[test]
    fn test_person_loader_allows_missing_body_sections() {
        let content = r#"
[header]
name = "partial person set"
target = "person"
"#;

        let loaded = toml::from_str::<AethelDoc<PersonLoader>>(content);

        assert!(loaded.is_ok());

        let loaded = loaded.unwrap();
        assert!(loaded.data.name.is_none());
    }

    #[test]
    fn test_person_loader_allows_missing_fields_in_existing_section() {
        let content = r#"
[header]
name = "partial person fields"
target = "person"

[name]
first = ["al"]
"#;

        let loaded = toml::from_str::<AethelDoc<PersonLoader>>(content);

        assert!(loaded.is_ok());

        let loaded = loaded.unwrap();
        let name = loaded.data.name.unwrap();
        assert_eq!(name.first.unwrap(), vec!["al".to_string()]);
        assert!(name.middle.is_none());
        assert!(name.last.is_none());
    }
}