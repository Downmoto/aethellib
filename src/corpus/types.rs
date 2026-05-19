//! core document model types shared across loaders, corpus, and rules modules.

use serde::{Deserialize, Serialize};

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
