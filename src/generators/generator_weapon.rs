use std::fmt;

use rand::seq::SliceRandom;
use rand::thread_rng;
use rand::Rng;

use crate::loader::loader_weapon::{
	WeaponLoader, WeaponLoreSection, WeaponNameSection, WeaponVisualSection,
};
use crate::loader::{AethelDoc, LoaderError};

pub struct WeaponGenerator {
	data: WeaponLoader,
}

#[derive(Debug)]
pub struct GeneratedWeapon {
	pub name: String,
	pub weapon_type: Option<String>,
	pub rarity: Option<String>,
	pub condition: Option<String>,
	pub lore: Option<String>,
	pub visuals: Option<String>,
}

impl fmt::Display for GeneratedWeapon {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let weapon_type = self.weapon_type.as_deref().unwrap_or("unknown");
		let rarity = self.rarity.as_deref().unwrap_or("unspecified");
		let condition = self.condition.as_deref().unwrap_or("unspecified");
		let lore = self.lore.as_deref().unwrap_or("none");
		let visuals = self.visuals.as_deref().unwrap_or("none");

		writeln!(f, "generated weapon")?;
		writeln!(f, "----------------")?;
		writeln!(f, "name      : {}", self.name)?;
		writeln!(f, "type      : {weapon_type}")?;
		writeln!(f, "rarity    : {rarity}")?;
		writeln!(f, "condition : {condition}")?;
		writeln!(f, "lore      : {lore}")?;
		write!(f, "visuals   : {visuals}")
	}
}

impl WeaponGenerator {
	pub fn new(document: AethelDoc<WeaponLoader>) -> Self {
		Self {
			data: document.data,
		}
	}

	pub fn from_file(path: &str) -> Result<Self, LoaderError> {
		let document = WeaponLoader::from_file(path)?;
		Ok(Self::new(document))
	}

	pub fn generate(&self) -> GeneratedWeapon {
		let mut rng = thread_rng();

		let weapon_type = self
			.data
			.weapon_type
			.as_ref()
			.and_then(|section| choose_random(&section.types, &mut rng));

		let rarity = self
			.data
			.qualities
			.as_ref()
			.and_then(|section| choose_random(&section.rarity, &mut rng));

		let condition = self
			.data
			.qualities
			.as_ref()
			.and_then(|section| choose_random(&section.condition, &mut rng));

		let name = build_name(self.data.name.as_ref(), &mut rng);
		let lore = build_lore(self.data.lore.as_ref(), &mut rng);
		let visuals = build_visuals(self.data.visuals.as_ref(), &mut rng);

		GeneratedWeapon {
			name,
			weapon_type,
			rarity,
			condition,
			lore,
			visuals,
		}
	}
}

fn choose_random(list: &Option<Vec<String>>, rng: &mut rand::rngs::ThreadRng) -> Option<String> {
	list.as_ref().and_then(|values| values.choose(rng).cloned())
}

fn build_name(
	section: Option<&WeaponNameSection>,
	rng: &mut rand::rngs::ThreadRng,
) -> String {
	let prefix = section.and_then(|s| choose_random(&s.prefix, rng));
	let suffix = section.and_then(|s| choose_random(&s.suffix, rng));

	let core = build_primitive_core(section, rng).unwrap_or_else(|| "weapon".to_string());

	let mut parts = Vec::new();
	if let Some(prefix) = prefix {
		parts.push(prefix);
	}
	parts.push(core);
	if let Some(suffix) = suffix {
		parts.push(suffix);
	}

	parts.join(" ")
}

fn build_primitive_core(
	section: Option<&WeaponNameSection>,
	rng: &mut rand::rngs::ThreadRng,
) -> Option<String> {
	let primitives = section?.primitives.as_ref()?;
	if primitives.is_empty() {
		return None;
	}

	let part_count = rng.gen_range(3..=5);
	let mut core = String::new();

	for _ in 0..part_count {
		if let Some(part) = primitives.choose(rng) {
			core.push_str(part);
		}
	}

	if core.is_empty() {
		None
	} else {
		let mut chars = core.chars();
		if let Some(first) = chars.next() {
			let mut capitalized = String::new();
			capitalized.push(first.to_ascii_uppercase());
			capitalized.push_str(chars.as_str());
			Some(capitalized)
		} else {
			None
		}
	}
}

fn build_lore(section: Option<&WeaponLoreSection>, rng: &mut rand::rngs::ThreadRng) -> Option<String> {
	let section = section?;
	let template = choose_random(&section.templates, rng)?;
	let creator = choose_random(&section.creators, rng).unwrap_or_default();
	let deed = choose_random(&section.deeds, rng).unwrap_or_default();
	let quirk = choose_random(&section.quirks, rng).unwrap_or_default();

	Some(
		template
			.replace("{creator}", &creator)
			.replace("{deed}", &deed)
			.replace("{quirk}", &quirk),
	)
}

fn build_visuals(
	section: Option<&WeaponVisualSection>,
	rng: &mut rand::rngs::ThreadRng,
) -> Option<String> {
	let section = section?;
	let template = choose_random(&section.templates, rng)?;
	let material = choose_random(&section.materials, rng).unwrap_or_default();
	let colour = choose_random(&section.colours, rng).unwrap_or_default();
	let accent = choose_random(&section.accents, rng).unwrap_or_default();
	let feature = choose_random(&section.features, rng).unwrap_or_default();

	Some(
		template
			.replace("{material}", &material)
			.replace("{colour}", &colour)
			.replace("{accent}", &accent)
			.replace("{feature}", &feature),
	)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_dbg_random_weapon() {
		let generator = WeaponGenerator::from_file("data/weapon_test_data.toml").unwrap();
		let generated = generator.generate();

		assert!(!generated.name.is_empty());

		println!("{generated}");
	}

	#[test]
	fn test_generated_weapon_display_looks_nice() {
		let generated = GeneratedWeapon {
			name: "Steel Longsword of Dawn".to_string(),
			weapon_type: Some("longsword".to_string()),
			rarity: Some("rare".to_string()),
			condition: Some("pristine".to_string()),
			lore: Some("forged by old masters".to_string()),
			visuals: Some("silver blade with blue accents".to_string()),
		};

		let output = format!("{generated}");

		assert!(output.contains("generated weapon"));
		assert!(output.contains("name      : Steel Longsword of Dawn"));
		assert!(output.contains("type      : longsword"));
		assert!(output.contains("rarity    : rare"));
		assert!(output.contains("condition : pristine"));
		assert!(output.contains("lore      : forged by old masters"));
		assert!(output.contains("visuals   : silver blade with blue accents"));
	}

	#[test]
	fn test_build_primitive_core_uses_three_to_five_parts() {
		let section = WeaponNameSection {
			prefix: None,
			suffix: None,
			primitives: Some(vec!["a".to_string(), "b".to_string(), "c".to_string()]),
		};
		let mut rng = thread_rng();

		let core = build_primitive_core(Some(&section), &mut rng).unwrap();
		let lowercase_core = core.to_ascii_lowercase();

		assert!((3..=5).contains(&core.len()));
		assert!(lowercase_core.chars().all(|ch| matches!(ch, 'a' | 'b' | 'c')));
	}
}