use rand::seq::SliceRandom;
use rand::thread_rng;

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

		let name = build_name(self.data.name.as_ref(), weapon_type.clone(), &mut rng);
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
	weapon_type: Option<String>,
	rng: &mut rand::rngs::ThreadRng,
) -> String {
	let prefix = section.and_then(|s| choose_random(&s.prefix, rng));
	let primitive = section.and_then(|s| choose_random(&s.primitives, rng));
	let suffix = section.and_then(|s| choose_random(&s.suffix, rng));

	let core = weapon_type
		.or(primitive)
		.unwrap_or_else(|| "weapon".to_string());

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
	}
}