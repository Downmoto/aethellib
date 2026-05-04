//! demonstrates merge options and merger option errors.
//! this shows duplicate header name behaviour control.

#[path = "common/mod.rs"]
mod support;

use aethellib::merger::{error::MergerError, merge_from_files, merger_options::{MergeOptions, MergerOptionError}};
use std::error::Error;
use support::{TempTomlFile, toml_document};

fn main() -> Result<(), Box<dyn Error>> {
    let one = TempTomlFile::new(&toml_document(
        "duplicate header",
        "custom",
        "[body]\nvalue = \"a\"",
    ));
    let two = TempTomlFile::new(&toml_document(
        "duplicate header",
        "custom",
        "[body]\nvalue = \"b\"",
    ));
    let paths = [one.path_str(), two.path_str()];

    let default_ok = merge_from_files(&paths, None)?;
    assert_eq!(default_ok.len(), 1);

    let strict = merge_from_files(
        &paths,
        Some(MergeOptions {
            identical_names_allowed: false,
        }),
    );

    match strict {
        Err(MergerError::MergerOption(MergerOptionError::IdenticalNameAllowed { header })) => {
            assert_eq!(header, "duplicate header");
            println!("strict merge rejected duplicate header: {header}");
        }
        _ => unreachable!("expected merger option error"),
    }

    Ok(())
}
