//! prelude exports for common aethellib workflows.

pub use crate::generator::{GeneratedField, Generation, GenerationError, SourceRef};
pub use crate::loader::TargetedLoader;
pub use crate::merger::merge_files;
pub use crate::merger::merger_options::MergeOptions;
pub use crate::{AethelCorpus, AethelDoc, AethelDocHeader, SourceAethelDoc};
