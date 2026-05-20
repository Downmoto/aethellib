//! standard combinator functions that each return a [`Rule`].

use rand::Rng;

use super::{AethelError, ComposedValue, CustomRule, GenerationContext, Rule};

/// selects a random value from the pool identified by `section` and `field`.
pub fn pick(
    name: impl Into<String>,
    section: String,
    field: String,
) -> impl Rule + 'static {
    CustomRule::new(name, move |ctx: &GenerationContext<'_>, rng: &mut dyn Rng| {
        let values = ctx
            .corpus
            .pooled_values_for_field_section(&field, &section)
            .ok_or_else(|| AethelError::PoolNotFound {
                section: section.clone(),
                field: field.clone(),
            })?;

        if values.is_empty() {
            return Err(AethelError::Custom("pool is empty".to_string()));
        }

        let index = (rng.next_u32() as usize) % values.len();
        let selected = &values[index];

        Ok(ComposedValue {
            value: selected.value.clone(),
            provenance: selected.provenance.clone(),
        })
    })
}

/// executes `rule_a` and `rule_b` sequentially and merges their results into one [`ComposedValue`].
pub fn concat(
    name: impl Into<String>,
    rule_a: impl Rule + 'static,
    rule_b: impl Rule + 'static,
) -> impl Rule + 'static {
    CustomRule::new(name, move |ctx: &GenerationContext<'_>, rng: &mut dyn Rng| {
        let val_a = rule_a.execute(ctx, rng)?;
        let val_b = rule_b.execute(ctx, rng)?;
        Ok(val_a.merge(val_b))
    })
}

/// tries `primary`; on any error executes `secondary` instead.
pub fn fallback(
    name: impl Into<String>,
    primary: impl Rule + 'static,
    secondary: impl Rule + 'static,
) -> impl Rule + 'static {
    CustomRule::new(name, move |ctx: &GenerationContext<'_>, rng: &mut dyn Rng| {
        match primary.execute(ctx, rng) {
            Ok(val) => Ok(val),
            Err(_) => secondary.execute(ctx, rng),
        }
    })
}

/// performs a single weighted RNG roll to select and execute one rule from `choices`.
///
/// returns [`AethelError::Custom`] if the total weight of all choices is zero.
pub fn weighted_choice(
    name: impl Into<String>,
    choices: Vec<(u32, Box<dyn Rule>)>,
) -> impl Rule + 'static {
    CustomRule::new(name, move |ctx: &GenerationContext<'_>, rng: &mut dyn Rng| {
        let total_weight: u32 = choices.iter().map(|(w, _)| w).sum();

        if total_weight == 0 {
            return Err(AethelError::Custom(
                "weighted choice has a total weight of 0".to_string(),
            ));
        }

        let mut roll = rng.next_u32() % total_weight;

        for (weight, rule) in &choices {
            if roll < *weight {
                return rule.execute(ctx, rng);
            }
            roll -= weight;
        }

        Err(AethelError::Custom(
            "mathematical error in weighted choice".to_string(),
        ))
    })
}

/// evaluates `probability` (0.0–1.0) against a single RNG roll.
/// on success executes `rule`; on failure returns an empty [`ComposedValue`].
pub fn chance(
    name: impl Into<String>,
    probability: f64,
    rule: impl Rule + 'static,
) -> impl Rule + 'static {
    CustomRule::new(name, move |ctx: &GenerationContext<'_>, rng: &mut dyn Rng| {
        let roll = (rng.next_u32() as f64) / (u32::MAX as f64);

        if roll <= probability {
            rule.execute(ctx, rng)
        } else {
            Ok(ComposedValue {
                value: String::new(),
                provenance: Vec::new(),
            })
        }
    })
}
