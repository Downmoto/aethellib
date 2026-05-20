//! aethellib provides loaders and corpus builders for aethel datasets.

/// corpus module entrypoint.
pub mod corpus;
/// generation engine, rule trait, and combinators.
pub mod engine;

pub mod prelude {
    //! prelude exports for common aethellib workflows.

    pub use crate::corpus::types::{Document, DocumentMetadata, Field, Section, Target};
    pub use crate::corpus::{Corpus, CorpusBuilder, PooledValue, ValuePool, ValueProvenance};
    pub use crate::corpus::error::{CorpusLoaderError, CorpusLoaderErrorKind};
    pub use crate::corpus::utils::{CorpusLoaderOptions, LoadValidator};

    pub use crate::engine::{ComposedValue, CustomRule, Engine, GenerationContext, Rule};
    pub use crate::engine::error::AethelError;
    pub use crate::engine::combinators::{chance, concat, fallback, pick, weighted_choice};
}
