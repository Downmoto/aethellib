//! aethellib provides loaders and corpus builders for aethel datasets.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// loader module entrypoint.
pub mod loader;
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

#[derive(Deserialize, Serialize, Debug, Clone)]
/// a rule defined by the TOML file author inside a section.
pub struct Rule {
    /// output field this rule applies to.
    #[serde(rename = "for")]
    pub for_field: String,
    /// rule kind identifier (e.g. `"scramble"`).
    pub rule: String,
    /// additional rule-specific parameters keyed by name.
    #[serde(flatten)]
    pub params: HashMap<String, toml::Value>,
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
    /// output field names produced by this section.
    pub outputs: Vec<String>,
    /// rules authored inside this section.
    pub rules: Vec<Rule>,
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

#[derive(Debug, Clone)]
/// a corpus of source documents for one target.
pub struct Corpus {
    /// target represented by all source documents.
    pub target: Target,
    /// source documents in first-seen order.
    pub documents: Vec<Document>,
}

impl Corpus {
    /// returns a new corpus builder for the given target.
    pub fn builder(target: impl Into<String>) -> crate::loader::CorpusBuilder {
        crate::loader::CorpusBuilder::new(target.into())
    }

    /// returns the target represented by this corpus.
    pub fn target(&self) -> &str {
        self.target.as_str()
    }

    /// combines two corpora with the same target into one, reassigning duplicate source ids.
    pub fn combine(self, other: Corpus) -> Self {
        let Corpus {
            target,
            mut documents,
        } = self;
        let Corpus {
            target: other_target,
            documents: other_documents,
        } = other;

        assert_eq!(
            target, other_target,
            "cannot combine corpora with different targets: left='{}', right='{}'",
            target, other_target
        );

        documents.extend(other_documents);

        let mut seen_source_hashes: HashMap<String, usize> = HashMap::new();
        for document in &mut documents {
            let count = seen_source_hashes
                .entry(document.source_hash.clone())
                .or_insert(0);
            *count += 1;

            if *count == 1 {
                document.source_id = document.source_hash.clone();
            } else {
                document.source_id = format!("{}:{}", document.source_hash, count);
            }
        }

        Self { target, documents }
    }

    /// returns all source ids in corpus order.
    pub fn source_ids(&self) -> Vec<&str> {
        self.documents
            .iter()
            .map(|document| document.source_id.as_str())
            .collect()
    }

    /// returns all source paths in corpus order.
    pub fn source_paths(&self) -> Vec<&str> {
        self.documents
            .iter()
            .map(|document| document.source_path.as_str())
            .collect()
    }

    /// finds one source document by source id.
    pub fn find_source(&self, source_id: &str) -> Option<&Document> {
        self.documents
            .iter()
            .find(|document| document.source_id == source_id)
    }
}

#[cfg(test)]
mod tests {
    use super::{Corpus, Document, DocumentMetadata, Section};

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
        };

        let right = Corpus {
            target: "weapon".to_string(),
            documents: vec![
                doc("hash-a", "old-2", "weapon"),
                doc("hash-b", "old-3", "weapon"),
            ],
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
        };

        let right = Corpus {
            target: "person".to_string(),
            documents: vec![doc("hash-b", "old-2", "person")],
        };

        let _ = left.combine(right);
    }
}
