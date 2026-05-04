//! implements a custom target loader and generator.
//! this demonstrates extending the library without built-in target features.

#[path = "common/mod.rs"]
mod support;

use aethellib::generator::{GeneratedField, Generator};
use aethellib::loader::TargetedLoader;
use aethellib::merger::AethelCorpus;
use rand::{Rng, seq::SliceRandom};
use serde::Deserialize;
use std::error::Error;
use support::{TempTomlFile, toml_document};

#[derive(Debug, Deserialize, Clone)]
struct SettlementLoader {
    name: Option<SettlementNameSection>,
}

#[derive(Debug, Deserialize, Clone)]
struct SettlementNameSection {
    starts: Option<Vec<String>>,
    ends: Option<Vec<String>>,
}

impl TargetedLoader for SettlementLoader {
    const TARGET: &'static str = "settlement";
}

#[derive(Debug)]
struct GeneratedSettlement {
    name: GeneratedField<String>,
}

struct SettlementGenerator {
    starts: Vec<String>,
    ends: Vec<String>,
}

impl Generator for SettlementGenerator {
    type Loader = SettlementLoader;
    type Output = GeneratedSettlement;

    fn new(corpus: AethelCorpus<Self::Loader>) -> Self {
        let mut starts = Vec::new();
        let mut ends = Vec::new();

        for document in corpus.documents {
            if let Some(name) = document.data.name {
                if let Some(values) = name.starts {
                    starts.extend(values);
                }
                if let Some(values) = name.ends {
                    ends.extend(values);
                }
            }
        }

        Self { starts, ends }
    }

    fn generate_with_rng<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::Output {
        let start = self
            .starts
            .choose(rng)
            .cloned()
            .unwrap_or_else(|| "new".to_string());
        let end = self
            .ends
            .choose(rng)
            .cloned()
            .unwrap_or_else(|| "vale".to_string());

        GeneratedSettlement {
            name: GeneratedField {
                value: format!("{start}{end}"),
                source_refs: Vec::new(),
            },
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let fixture = TempTomlFile::new(&toml_document(
        "settlement data",
        "settlement",
        r#"
[name]
starts = ["green", "river", "oak"]
ends = ["ford", "mere", "hold"]
"#,
    ));

    let generator = SettlementGenerator::from_file(fixture.path_str())?;
    let generated = generator.generate();

    println!("generated settlement: {}", generated.name.value);
    Ok(())
}
