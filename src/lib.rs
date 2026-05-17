//! aethellib provides loaders and corpus builders for aethel datasets.


use serde::{Deserialize, Serialize};

/// loader module entrypoint.
pub mod loader;
/// rules module entrypoint.
pub mod rules;

pub mod corpus;
/// prelude module for common imports.
pub mod prelude;

/// open target identifier used by loaders and corpus builders.
pub type Target = String;

#[derive(Deserialize, Serialize, Debug, Clone)]
/// common metadata from the `[header]` table of each input file.
pub struct DocumentMetadata {
    /// dataset display name.
    pub title: String,
    /// target category used for loader validation.
    pub target: Target,
    /// optional dataset description.
    pub desc: Option<String>,
    /// optional dataset author.
    pub author: Option<String>,
    /// optional dataset version.
    pub version: Option<String>,
    /// optional schema identifier.
    pub schema: Option<String>,
}

#[derive(Debug, Clone)]
/// a named value pool inside a section.
pub struct Field {
    /// field name as defined by the TOML key.
    pub title: String,
    /// candidate string values for this field.
    pub values: Vec<String>,
}

#[derive(Debug, Clone)]
/// a top-level section inside a document (one TOML table, excluding `[header]`).
pub struct Section {
    /// section name derived from the TOML table key.
    pub title: String,
    /// named value pools defined within this section.
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone)]
/// a parsed and corpus-tracked source document.
pub struct Document {
    /// metadata from the `[header]` table.
    pub metadata: DocumentMetadata,
    /// sections parsed from the document body.
    pub sections: Vec<Section>,
    /// unique id within the owning corpus instance.
    pub source_id: String,
    /// deterministic hash derived from document content and target.
    pub source_hash: String,
    /// original source path or name supplied at load time.
    pub source_path: String,
}

impl Document {
    /// returns the section with the given title, if present.
    pub fn section(&self, title: &str) -> Option<&Section> {
        self.sections.iter().find(|s| s.title == title)
    }
}



#[cfg(test)]
mod tests {
    use std::vec;

use super::{Document, DocumentMetadata, Section};

    use crate::corpus::Corpus;

    fn doc(hash: &str, id: &str, target: &str) -> Document {
        Document {
            source_id: id.to_string(),
            source_hash: hash.to_string(),
            source_path: format!("/{id}.toml"),
            metadata: DocumentMetadata {
                title: id.to_string(),
                target: target.to_string(),
                desc: None,
                author: None,
                version: None,
                schema: None,
            },
            sections: Vec::<Section>::new(),
        }
    }

    #[test]
    fn combine_appends_documents_and_renumbers_duplicate_hash_ids() {
        let left = Corpus {
            target: "weapon".to_string(),
            documents: vec![doc("hash-a", "old-1", "weapon")],
            pools: vec![]
        };

        let right = Corpus {
            target: "weapon".to_string(),
            documents: vec![
                doc("hash-a", "old-2", "weapon"),
                doc("hash-b", "old-3", "weapon"),
            ],
            pools: vec![]
        };

        let combined = left.combine(right);

        assert_eq!(combined.target, "weapon");
        assert_eq!(combined.documents.len(), 3);
        assert_eq!(combined.documents[0].source_id, "hash-a");
        assert_eq!(combined.documents[1].source_id, "hash-a:2");
        assert_eq!(combined.documents[2].source_id, "hash-b");
    }

    #[test]
    #[should_panic(expected = "cannot combine corpora with different targets")]
    fn combine_panics_for_different_targets() {
        let left = Corpus {
            target: "weapon".to_string(),
            documents: vec![doc("hash-a", "old-1", "weapon")],
            pools: vec![]
        };

        let right = Corpus {
            target: "person".to_string(),
            documents: vec![doc("hash-b", "old-2", "person")],
            pools: vec![]
        };

        let _ = left.combine(right);
    }
}
