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
