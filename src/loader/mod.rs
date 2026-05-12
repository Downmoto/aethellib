//! loader primitives for parsing and validating aethel source documents.

pub mod error;

use serde::de::DeserializeOwned;
use std::fs;
use std::path::Path;

use crate::AethelDoc;
use crate::loader::error::LoaderError;

pub trait TargetedLoader: Sized + DeserializeOwned {
    /// expected target for this loader implementation.
    const TARGET: &'static str;

    /// load, parse, and target-validate a single toml file.
    fn from_file(path: impl AsRef<Path>) -> Result<AethelDoc<Self>, LoaderError> {
        let path_ref = path.as_ref();
        let raw = fs::read_to_string(path_ref)
            .map_err(|source| LoaderError::read_for_path(path_ref, source))?;
        let parsed: AethelDoc<Self> =
            toml::from_str(&raw).map_err(|source| LoaderError::parse_for_path(path_ref, source))?;

        if parsed.header.target != Self::TARGET {
            return Err(LoaderError::TargetMismatch {
                expected: Self::TARGET.to_string(),
                found: parsed.header.target.clone(),
            });
        }

        Ok(parsed)
    }
}