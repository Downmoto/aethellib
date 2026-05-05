//! merges files for multiple targets and preserves first-seen target order.
//! this demonstrates merge_from_files and mergedaetheldoc::target.

#[path = "common/mod.rs"]
mod support;

use aethellib::merger::merge_from_files;
use std::error::Error;
use support::{TempTomlFile, toml_document};

fn main() -> Result<(), Box<dyn Error>> {
    // feed mixed targets in a deliberate order so we can demonstrate that
    // merge_from_files preserves first-seen target ordering.
    let alpha_one = TempTomlFile::new(&toml_document(
        "alpha one",
        "alpha",
        "[name]\nvalue = \"a\"",
    ));
    let beta_one = TempTomlFile::new(&toml_document("beta one", "beta", "[name]\nvalue = \"b\""));
    let alpha_two = TempTomlFile::new(&toml_document(
        "alpha two",
        "alpha",
        "[name]\nvalue = \"c\"",
    ));

    let paths = [
        alpha_one.path_str(),
        beta_one.path_str(),
        alpha_two.path_str(),
    ];

    // output is one MergedAethelDoc per target, not one per source file.
    let merged = merge_from_files(&paths, None)?;

    // first seen target is alpha, so alpha is index 0.
    assert_eq!(merged.len(), 2);
    assert_eq!(merged[0].target(), "alpha");
    assert_eq!(merged[1].target(), "beta");
    assert_eq!(merged[0].documents.len(), 2);
    assert_eq!(merged[1].documents.len(), 1);

    for doc in &merged {
        println!(
            "target {} has {} sources",
            doc.target(),
            doc.documents.len()
        );
    }

    Ok(())
}
