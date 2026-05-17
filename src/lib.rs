//! aethellib provides loaders and corpus builders for aethel datasets.

/// loader module entrypoint.
pub mod loader;
/// rules module entrypoint.
pub mod rules;
/// corpus module entrypoint.
pub mod corpus;


/// prelude module for common imports.
pub mod prelude {
    //! prelude exports for common aethellib workflows.

    pub use crate::loader::error::{LoaderError, LoaderErrorKind};
    pub use crate::loader::{LoadOptions, LoadValidator, load_files, load_files_with_validator};
    pub use crate::corpus::{Corpus, CorpusBuilder, PooledValue, ValuePool, ValueProvenance};
    pub use crate::corpus::types::{Document, DocumentMetadata, Field, Section, Target};
    pub use crate::rules::Rule;

}