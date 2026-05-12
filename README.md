# aethellib

work in progress.

aethellib is currently in active pre-1.0 refactor mode.
the api may change between minor releases while architecture settles.

## status

- current crate version: 0.3.x
- release stability target: 1.0.0
- this readme is intentionally minimal until 1.0

## project direction

aethellib is now focused on reusable primitives for building your own target loaders and generators:

- typed document loading and target validation
- single-target corpus merging with source provenance metadata
- generator trait surface for runtime generation workflows

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

implement a loader:

```rust
use serde::Deserialize;
use aethellib::loader::TargetedLoader;

#[derive(Debug, Deserialize)]
struct MyLoader {
	// your target-specific sections here
}

impl TargetedLoader for MyLoader {
	const TARGET: &'static str = "my-target";
}
```

merge source files into a typed corpus:

```rust
use aethellib::merger::merge_files;

let paths = vec!["data/a.toml", "data/b.toml"];
let corpus = merge_files::<MyLoader>(&paths, None)?;
```

## living reference

for up-to-date usage, run and read:

- [examples/test.rs](examples/test.rs)

```bash
cargo run --example test
```

## development

```bash
cargo test
cargo run --example test
```

## 1.0 readme plan

the full readme will be finalized at 1.0 and will include:

- stable api guarantees
- full data format specification
- migration guidance from pre-1.0 releases
- complete examples and integration patterns
- docs and api reference via Starlight docs pages