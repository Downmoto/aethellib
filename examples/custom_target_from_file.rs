//! purpose: show a downstream-defined target loader and generator using `merge_target_files`.

use std::error::Error;

use aethellib::generator::Generator;
use aethellib::loader::TargetedLoader;
use aethellib::merger::{AethelCorpus, merge_target_files};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
struct SettlementLoader {
    settlement: Option<SettlementSection>,
}

#[derive(Debug, Deserialize, Clone)]
struct SettlementSection {
    names: Option<Vec<String>>,
}

impl TargetedLoader for SettlementLoader {
    const TARGET: &'static str = "settlement";
}

struct SettlementGenerator {
    names: Vec<String>,
}

impl Generator for SettlementGenerator {
    type Loader = SettlementLoader;
    type Output = String;

    fn new(corpus: AethelCorpus<Self::Loader>) -> Self {
        let mut names = Vec::new();

        for document in corpus.documents {
            if let Some(section) = document.data.settlement {
                if let Some(section_names) = section.names {
                    names.extend(section_names);
                }
            }
        }

        if names.is_empty() {
            names.push("new haven".to_string());
        }

        Self { names }
    }

    fn generate_with_rng<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Self::Output {
        use rand::seq::SliceRandom;

        self.names
            .choose(rng)
            .cloned()
            .unwrap_or_else(|| "new haven".to_string())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let temp_path = std::env::temp_dir().join("aethellib_custom_target_example.toml");

    let result = (|| -> Result<(), Box<dyn Error>> {
        std::fs::write(
            &temp_path,
            r#"
[header]
name = "settlement example"
target = "settlement"

[settlement]
names = ["oakrest", "sunford", "greymoor"]
"#,
        )?;

        let path = temp_path.to_string_lossy().to_string();
        let corpus = merge_target_files::<SettlementLoader>(&[path.as_str()], None)?;

        let generator = SettlementGenerator::new(corpus);
        let generated = generator.generate();

        println!("custom target from merge_target_files -> {generated}");
        Ok(())
    })();

    let _ = std::fs::remove_file(&temp_path);
    result
}
