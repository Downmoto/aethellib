//! demonstrates loader errors for read, parse, and target mismatch failures.
//! this is useful for robust caller-side error handling.

#[path = "common/mod.rs"]
mod support;

use aethellib::{
    AethelDoc,
    loader::{TARGET_PERSON, TARGET_WEAPON, TargetedLoader, error::LoaderError},
};
use serde::Deserialize;
use std::error::Error;
use support::TempTomlFile;

#[derive(Deserialize, Debug)]
struct EmptyLoader;

impl TargetedLoader for EmptyLoader {
    const TARGET: &'static str = TARGET_WEAPON;
}

fn main() -> Result<(), Box<dyn Error>> {
    // 1) read error: file is missing, so filesystem load fails before parsing.
    let missing_path = std::env::temp_dir().join("aethellib_missing_example_input.toml");
    match EmptyLoader::from_file(&missing_path) {
        Err(LoaderError::ReadError { .. }) => println!("read error handled"),
        _ => unreachable!("expected read error"),
    }

    // 2) parse error: file exists, but toml syntax is invalid.
    let malformed = TempTomlFile::new("not valid toml = [");
    match EmptyLoader::from_file(malformed.path_str()) {
        Err(LoaderError::ParseError { .. }) => println!("parse error handled"),
        _ => unreachable!("expected parse error"),
    }

    // 3) target mismatch: parsed header.target does not match loader TARGET.
    let mismatch = TempTomlFile::new("[header]\nname = \"wrong target\"\ntarget = \"person\"\n");
    match EmptyLoader::from_file(mismatch.path_str()) {
        Err(LoaderError::TargetMismatch { expected, found }) => {
            assert_eq!(expected, TARGET_WEAPON);
            assert_eq!(found, TARGET_PERSON);
            println!("target mismatch handled: expected {expected}, found {found}");
        }
        _ => unreachable!("expected target mismatch"),
    }

    // direct parsing into AethelDoc is useful when you want to inspect raw
    // structured content without invoking TargetedLoader validation.
    let parsed: AethelDoc<toml::Table> =
        toml::from_str("[header]\nname = \"demo\"\ntarget = \"weapon\"\n")?;
    println!("parsed header via aetheldoc: {}", parsed.header.name);

    Ok(())
}
