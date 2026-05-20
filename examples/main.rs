use aethellib::prelude::*;
use rand::Rng;

/// returns a rule that always produces a fixed string with no provenance.
fn lit(text: &'static str) -> impl Rule + 'static {
    CustomRule::new(text, move |_ctx: &GenerationContext<'_>, _rng: &mut dyn Rng| {
        Ok(ComposedValue {
            value: text.to_string(),
            provenance: vec![],
        })
    })
}

fn main() {
    let corpus = Corpus::from_files(
        &["data/weapon_test_data.toml"],
        "weapon",
        None,
        None,
    )
    .expect("failed to load weapon corpus");

    let ctx = Engine::new(&corpus, rand::rng())
        // weapon name: "<prefix> <type> [<suffix>]" — suffix is added 60 % of the time
        .with_rule(concat(
            "weapon_name",
            concat(
                "prefix_type",
                pick("px", "name".to_string(), "prefix".to_string()),
                concat(
                    "space_type",
                    lit(" "),
                    pick("ty", "type".to_string(), "type".to_string()),
                ),
            ),
            chance(
                "optional_suffix",
                0.85,
                concat(
                    "space_suffix",
                    lit(" "),
                    pick("sx", "name".to_string(), "suffix".to_string()),
                ),
            ),
        ))
        // quality
        .with_rule(pick("rarity",    "qualities".to_string(), "rarity".to_string()))
        .with_rule(pick("condition", "qualities".to_string(), "condition".to_string()))
        // visuals
        .with_rule(pick("material", "visuals".to_string(), "materials".to_string()))
        .with_rule(pick("accent",   "visuals".to_string(), "accents".to_string()))
        // weighted elemental trait: mundane is intentionally most common
        .with_rule(weighted_choice(
            "element",
            vec![
                (60, Box::new(lit("Mundane"))),
                (18, Box::new(lit("Flaming"))),
                (12, Box::new(lit("Frosted"))),
                (7, Box::new(lit("Storm-touched"))),
                (3, Box::new(lit("Voidbound"))),
            ],
        ))
        .generate()
        .expect("generation failed");

    let get = |key: &str| ctx.get_previous(key).unwrap().value.as_str();

    println!("─ Generated Weapon ─────────────────────────────────");
    println!("  Name       {}", get("weapon_name"));
    println!("  Element    {}", get("element"));
    println!("  Rarity     {} · {}", get("rarity"), get("condition"));
    println!("  Material   {}, {}", get("material"), get("accent"));
}
