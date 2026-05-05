//! converts merged untyped documents into typed corpora.
//! this demonstrates mergedaetheldoc::to_corpus and mergedaetheldoc::into_corpus.

#[path = "common/mod.rs"]
mod support;

use aethellib::loader::{TargetedLoader, error::LoaderError};
use aethellib::merger::{error::MergerError, merge_from_files};
use serde::Deserialize;
use std::error::Error;
use support::{TempTomlFile, toml_document};

#[derive(Debug, Deserialize, Clone)]
struct AlphaLoader {
    name: Option<AlphaNameSection>,
}

#[derive(Debug, Deserialize, Clone)]
struct AlphaNameSection {
    value: Option<String>,
}

impl TargetedLoader for AlphaLoader {
    const TARGET: &'static str = "alpha";
}

#[derive(Debug, Deserialize, Clone)]
struct BetaLoader;

impl TargetedLoader for BetaLoader {
    const TARGET: &'static str = "beta";
}

fn main() -> Result<(), Box<dyn Error>> {
    // start from a generic mixed-target merge output.
    let alpha = TempTomlFile::new(&toml_document(
        "alpha dataset",
        "alpha",
        "[name]\nvalue = \"alpha-name\"",
    ));
    let merged = merge_from_files(&[alpha.path_str()], None)?;

    // to_corpus clones the merged doc internally, so the original value remains
    // available for more conversions.
    let borrowed = merged[0].to_corpus::<AlphaLoader>()?;
    assert_eq!(borrowed.target, AlphaLoader::TARGET);
    assert_eq!(borrowed.documents.len(), 1);
    assert_eq!(
        borrowed.documents[0]
            .data
            .name
            .as_ref()
            .and_then(|section| section.value.as_deref()),
        Some("alpha-name")
    );

    // converting with the wrong loader target returns a typed mismatch error.
    match merged[0].to_corpus::<BetaLoader>() {
        Err(MergerError::Loader(LoaderError::TargetMismatch { .. })) => {
            println!("mismatch conversion rejected as expected")
        }
        _ => unreachable!("expected target mismatch for wrong loader"),
    }

    // into_corpus consumes the merged value (or a clone of it), which can be
    // preferable when you do not need the untyped version afterwards.
    let owned = merged[0].clone().into_corpus::<AlphaLoader>()?;
    println!(
        "into_corpus consumed clone with {} document",
        owned.documents.len()
    );

    Ok(())
}
