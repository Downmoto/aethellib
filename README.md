# aethellib

work in progress.

aethellib is currently in active pre-1.0 refactor mode.
the api may change between minor releases while architecture settles.

## status

- current crate version: 0.8.1
- release stability target: 1.0.0
- this readme is intentionally minimal until 1.0

## project direction

aethellib is now focused on reusable primitives for loading target-specific corpora and running composable text generation rules:

- typed document loading and target validation
- single-target corpus merging with source provenance metadata
- generation engine with composable combinators

it does not aim to be a mixed-target orchestration framework.
if you need mixed-target handling, do that grouping and dispatch in your own binary or library layer.

## pre-1.0 compatibility policy

before 1.0, the following may change without long deprecation windows:

- public api naming and module layout
- trait bounds and helper constructors
- example structure and fixture strategy

## installation

```toml
[dependencies]
aethellib = { git = "https://github.com/Downmoto/aethellib", branch = "master" }
```

## current quick start

load a corpus from one or more TOML files:

```rust
use aethellib::prelude::*;

let corpus = Corpus::from_files(
	&["data/weapon_test_data.toml"],
	"weapon",
	None,
	None,
)?;
```

run generation rules with the engine:

```rust
use aethellib::prelude::*;
use aethellib::engine::combinators::{chance, concat, lit, pick, weighted_choice};

let ctx = Engine::new(&corpus, rand::rng())
	.with_rule(concat(
		"weapon_name",
		pick("px", "name".to_string(), "prefix".to_string()),
		pick("ty", "type".to_string(), "type".to_string()),
	))
	.with_rule(chance(
		"optional_suffix",
		0.85,
		pick("sx", "name".to_string(), "suffix".to_string()),
	))
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
	.generate()?;

let name = ctx.get_previous("weapon_name").unwrap().value.as_str();
let element = ctx.get_previous("element").unwrap().value.as_str();
println!("{} {}", element, name);
```

## living reference

for up-to-date usage, run and read:

- [examples/main.rs](examples/main.rs)

```bash
cargo run --example main
```

## development

```bash
cargo test
cargo run --example main
```

## 1.0 readme plan

the full readme will be finalized at 1.0 and will include:

- stable api guarantees
- full data format specification
- complete examples and integration patterns
- docs and api reference via docs pages