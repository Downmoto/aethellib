//! aethellib provides loaders and corpus builders for aethel datasets.

/// corpus module entrypoint.
pub mod corpus;
/// loader module entrypoint.
pub mod loader;
/// rules module entrypoint.
pub mod rules;

pub mod prelude {
    //! prelude exports for common aethellib workflows.

    pub use crate::corpus::types::{Document, DocumentMetadata, Field, Section, Target};
    pub use crate::corpus::{Corpus, CorpusBuilder, PooledValue, ValuePool, ValueProvenance};
    pub use crate::loader::error::{LoaderError, LoaderErrorKind};
    pub use crate::loader::{LoadOptions, LoadValidator, load_files, load_files_with_validator};
}
