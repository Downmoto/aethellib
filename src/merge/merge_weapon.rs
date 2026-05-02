use crate::loader::loader_weapon::{
    WeaponLoader, WeaponLoreSection, WeaponNameSection, WeaponQualitiesSection,
    WeaponTypeSection, WeaponVisualSection,
};
use crate::loader::{AethelDoc, LoaderError, Target, TargetedLoader};

pub fn merge_weapon_files(paths: &[&str]) -> Result<AethelDoc<WeaponLoader>, LoaderError> {
    let first_path = paths.first().ok_or_else(|| {
        LoaderError::ReadError(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "at least one path is required for merge",
        ))
    })?;

    let first_loaded = <WeaponLoader as TargetedLoader>::from_file(first_path)?;
    let mut merged = AethelDoc {
        header: first_loaded.header,
        data: WeaponLoader {
            name: None,
            weapon_type: None,
            qualities: None,
            lore: None,
            visuals: None,
        },
    };

    for path in paths {
        let next = <WeaponLoader as TargetedLoader>::from_file(path)?;
        merge_weapon_loader(&mut merged.data, &next.data);
    }

    merged.header.target = Target::Weapon;
    Ok(merged)
}

fn merge_weapon_loader(base: &mut WeaponLoader, incoming: &WeaponLoader) {
    merge_name_section(&mut base.name, &incoming.name);
    merge_type_section(&mut base.weapon_type, &incoming.weapon_type);
    merge_qualities_section(&mut base.qualities, &incoming.qualities);
    merge_lore_section(&mut base.lore, &incoming.lore);
    merge_visuals_section(&mut base.visuals, &incoming.visuals);
}

fn merge_name_section(base: &mut Option<WeaponNameSection>, incoming: &Option<WeaponNameSection>) {
    if let Some(incoming) = incoming {
        let section = base.get_or_insert_with(|| WeaponNameSection {
            prefix: None,
            suffix: None,
            primitives: None,
        });
        merge_option_vec(&mut section.prefix, &incoming.prefix);
        merge_option_vec(&mut section.suffix, &incoming.suffix);
        merge_option_vec(&mut section.primitives, &incoming.primitives);
    }
}

fn merge_type_section(base: &mut Option<WeaponTypeSection>, incoming: &Option<WeaponTypeSection>) {
    if let Some(incoming) = incoming {
        let section = base.get_or_insert_with(|| WeaponTypeSection { types: None });
        merge_option_vec(&mut section.types, &incoming.types);
    }
}

fn merge_qualities_section(
    base: &mut Option<WeaponQualitiesSection>,
    incoming: &Option<WeaponQualitiesSection>,
) {
    if let Some(incoming) = incoming {
        let section = base.get_or_insert_with(|| WeaponQualitiesSection {
            rarity: None,
            condition: None,
        });
        merge_option_vec(&mut section.rarity, &incoming.rarity);
        merge_option_vec(&mut section.condition, &incoming.condition);
    }
}

fn merge_lore_section(base: &mut Option<WeaponLoreSection>, incoming: &Option<WeaponLoreSection>) {
    if let Some(incoming) = incoming {
        let section = base.get_or_insert_with(|| WeaponLoreSection {
            creators: None,
            deeds: None,
            quirks: None,
            templates: None,
        });
        merge_option_vec(&mut section.creators, &incoming.creators);
        merge_option_vec(&mut section.deeds, &incoming.deeds);
        merge_option_vec(&mut section.quirks, &incoming.quirks);
        merge_option_vec(&mut section.templates, &incoming.templates);
    }
}

fn merge_visuals_section(
    base: &mut Option<WeaponVisualSection>,
    incoming: &Option<WeaponVisualSection>,
) {
    if let Some(incoming) = incoming {
        let section = base.get_or_insert_with(|| WeaponVisualSection {
            materials: None,
            colours: None,
            accents: None,
            features: None,
            templates: None,
        });
        merge_option_vec(&mut section.materials, &incoming.materials);
        merge_option_vec(&mut section.colours, &incoming.colours);
        merge_option_vec(&mut section.accents, &incoming.accents);
        merge_option_vec(&mut section.features, &incoming.features);
        merge_option_vec(&mut section.templates, &incoming.templates);
    }
}

fn merge_option_vec(base: &mut Option<Vec<String>>, incoming: &Option<Vec<String>>) {
    if let Some(incoming_values) = incoming {
        let values = base.get_or_insert_with(Vec::new);
        for value in incoming_values {
            if !values.contains(value) {
                values.push(value.clone());
            }
        }
    }
}