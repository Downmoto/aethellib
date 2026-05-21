use std::fmt;

#[derive(Debug)]
/// errors produced by the generation engine.
pub enum AethelError {
    /// no value pool exists for the given section and field pair.
    PoolNotFound { section: String, field: String },
    /// a rule required a previous result that is absent from the context history.
    MissingDependency(String),
    /// a user-defined or configuration error with a descriptive message.
    Custom(String),
}

impl fmt::Display for AethelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AethelError::PoolNotFound { section, field } => {
                write!(f, "pool not found for section '{section}', field '{field}'")
            }
            AethelError::MissingDependency(key) => {
                write!(f, "missing dependency in generation context: '{key}'")
            }
            AethelError::Custom(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for AethelError {}
