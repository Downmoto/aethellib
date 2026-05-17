use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RuleType {
    // -- control and structure --
    /// executes an ordered sequence of sub-rules on the target field.
    Pipeline,

    // -- Composition --
    /// selects item(s) from value pools.
    Pick,
    /// assembles strings using static templates and placeholders.
    Pattern,
    /// combines exactly one random choice from multiple specified pools in order.
    Join,
    /// selects a randomized amount of items from candidate pools, joining them.
    Scramble,

    // -- Mutation --
    /// modifies text formatting or casing.
    Transform,
    /// attaches static strings to the boundaries.
    Affix,
    /// performs simple string substitution.
    Replace,

}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Condition {
    pub field: String,
    pub matches: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
/// a rule defined by the TOML file author inside a section.
pub struct Rule {
    /// output field this rule applies to.
    #[serde(rename = "for")]
    pub for_field: String,
    /// rule kind identifier (e.g. [`RuleType::Scramble`]).
    pub rule: RuleType,
    /// additional rule-specific parameters keyed by name.
    
    pub chance: Option<f32>,
    pub condition: Option<Condition>,

    #[serde(flatten)]
    pub params: HashMap<String, toml::Value>,
}

impl Rule {
    pub fn has_params(&self) -> bool {
        match self.rule {
            RuleType::Pipeline => true,
            RuleType::Pick => true,
            RuleType::Pattern => true,
            RuleType::Join => true,
            RuleType::Scramble => true,
            RuleType::Transform => true,
            RuleType::Affix => true,
            RuleType::Replace => true,
        }
    }
}