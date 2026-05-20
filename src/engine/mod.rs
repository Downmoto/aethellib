pub mod combinators;
pub mod error;

use std::collections::HashMap;

use rand::Rng;

use crate::corpus::{Corpus, ValueProvenance};
pub use error::AethelError;

// ─── ComposedValue ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
/// the final or intermediate result of a generation step.
pub struct ComposedValue {
    pub value: String,
    pub provenance: Vec<ValueProvenance>,
}

impl ComposedValue {
    /// merges two composed values, appending the string and extending the provenance vector.
    pub fn merge(mut self, other: ComposedValue) -> Self {
        self.value.push_str(&other.value);
        self.provenance.extend(other.provenance);
        self
    }
}

// ─── GenerationContext ────────────────────────────────────────────────────────

/// the shared state blackboard passed to every generation rule during execution.
pub struct GenerationContext<'a> {
    /// the corpus available to all rules.
    pub corpus: &'a Corpus,
    /// results of previously executed rules keyed by rule name.
    pub history: HashMap<String, ComposedValue>,
}

impl<'a> GenerationContext<'a> {
    pub fn new(corpus: &'a Corpus) -> Self {
        Self {
            corpus,
            history: HashMap::new(),
        }
    }

    /// returns the result of a previously executed rule, if present.
    pub fn get_previous(&self, key: &str) -> Option<&ComposedValue> {
        self.history.get(key)
    }
}

// ─── Rule ─────────────────────────────────────────────────────────────────────

/// the base interface for all generation logic.
pub trait Rule {
    /// the unique identifier for what this rule generates.
    fn name(&self) -> &str;

    /// executes the generation logic against the current context and rng.
    fn execute<'a>(
        &self,
        ctx: &GenerationContext<'a>,
        rng: &mut dyn Rng,
    ) -> Result<ComposedValue, AethelError>;
}

// ─── CustomRule ───────────────────────────────────────────────────────────────

/// wraps a closure as a [`Rule`], allowing inline rule definitions without
/// implementing the trait directly.
pub struct InlineRule<F> {
    name: String,
    logic: F,
}

impl<F> InlineRule<F> {
    pub fn new(name: impl Into<String>, logic: F) -> Self {
        Self {
            name: name.into(),
            logic,
        }
    }
}

impl<F> Rule for InlineRule<F>
where
    F: for<'a> Fn(&GenerationContext<'a>, &mut dyn Rng) -> Result<ComposedValue, AethelError>,
{
    fn name(&self) -> &str {
        &self.name
    }

    fn execute<'a>(
        &self,
        ctx: &GenerationContext<'a>,
        rng: &mut dyn Rng,
    ) -> Result<ComposedValue, AethelError> {
        (self.logic)(ctx, rng)
    }
}

// ─── Engine ───────────────────────────────────────────────────────────────────

/// orchestrates an ordered sequence of [`Rule`]s against a [`Corpus`] using a
/// seeded RNG to ensure deterministic outputs.
pub struct Engine<'a, R: Rng> {
    corpus: &'a Corpus,
    rng: R,
    rules: Vec<Box<dyn Rule>>,
}

impl<'a, R: Rng> Engine<'a, R> {
    pub fn new(corpus: &'a Corpus, rng: R) -> Self {
        Self {
            corpus,
            rng,
            rules: Vec::new(),
        }
    }

    /// appends a rule to the end of the pipeline.
    pub fn with_rule(mut self, rule: impl Rule + 'static) -> Self {
        self.rules.push(Box::new(rule));
        self
    }

    /// runs every rule in pipeline order, storing each result in the context
    /// history, and returns the populated context.
    pub fn generate(mut self) -> Result<GenerationContext<'a>, AethelError> {
        let mut ctx = GenerationContext::new(self.corpus);

        for rule in self.rules {
            let name = rule.name().to_string();
            let result = rule.execute(&ctx, &mut self.rng)?;
            ctx.history.insert(name, result);
        }

        Ok(ctx)
    }
}
