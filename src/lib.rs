//! aethellib provides loaders and corpus builders for aethel datasets.

/// corpus module entrypoint.
pub mod corpus;
/// rules module entrypoint.
pub mod rules;

pub mod prelude {
    //! prelude exports for common aethellib workflows.

    pub use crate::corpus::types::{Document, DocumentMetadata, Field, Section, Target};
    pub use crate::corpus::{Corpus, CorpusBuilder, PooledValue, ValuePool, ValueProvenance};
    pub use crate::corpus::error::{CorpusLoaderError, CorpusLoaderErrorKind};
    pub use crate::corpus::utils::{CorpusLoaderOptions, LoadValidator};
}
