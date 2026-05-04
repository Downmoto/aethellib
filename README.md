# Aethillib

## breaking api update

the merge pipeline is now corpus-based. instead of flattening all input files into a single data blob with one header, merge output now retains every source document and its own metadata.

## what changed

1. `merge::merge_from_files` returns corpus variants that contain `documents`.
2. each source document now has `source_id`, `source_hash`, `source_path`, `header`, and `data`.
3. source ids are derived from canonicalized document content plus target.
4. identical content still gets unique per-corpus ids using a deterministic suffix.
5. `generators::generator_weapon::WeaponGenerator` now consumes corpus docs and emits provenance-rich output.
6. `GeneratedWeapon` fields are now `GeneratedField<T>` values, so every generated field can expose source references.
7. `MergedAethelDoc` now includes `as_weapon()` and `into_weapon()` accessors for cleaner variant extraction.
8. generators now expose deterministic generation via `generate_with_rng(...)`.

## metadata behavior

1. `header.version` remains file-specific metadata for downstream consumers.
2. merge compatibility is target-only.
3. different source versions can be merged together.
4. duplicate header names are allowed.

## migration notes

old usage:

```rust
let docs = merge::merge_from_files(&paths)?;
let weapon_doc = match &docs[0] {
	merge::MergedAethelDoc::Weapon(doc) => doc,
};
let generator = generators::generator_weapon::WeaponGenerator::new(weapon_doc.clone());
```

new usage:

```rust
let docs = merge::merge_from_files(&paths)?;
let weapon_corpus = docs.into_iter().next().unwrap().into_weapon().unwrap();

let generator = generators::generator_weapon::WeaponGenerator::new(weapon_corpus);
let generated = generator.generate();

println!("name: {}", generated.name.value);
if let Some(rarity) = &generated.rarity {
	for source in &rarity.source_refs {
		println!("rarity from {} ({})", source.source_name, source.source_id);
	}
}
```

## deterministic generation

use `generate_with_rng(...)` whenever you need reproducible generation during tests or debugging.

```rust
use rand::rngs::StdRng;
use rand::SeedableRng;

let mut rng = StdRng::seed_from_u64(42);
let generated = generator.generate_with_rng(&mut rng);
```

## error diagnostics

loader and merge read/parse errors now include file path context in error messages.

example shape:

```text
unable to parse toml file 'data/weapon_bad.toml': expected a right bracket, found eof
```
