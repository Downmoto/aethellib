//! prelude exports for common aethellib workflows.

pub use crate::loader::error::{LoaderError, LoaderErrorKind};
pub use crate::loader::{CorpusBuilder, LoadOptions, LoadValidator, load_files, load_files_with_validator, load_str};
pub use crate::{Corpus, Document, DocumentMetadata, Field, Rule, Section, Target};
