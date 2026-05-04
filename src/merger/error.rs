use std::fmt;

use crate::{loader::error::LoaderError, merger::merger_options::MergerOptionError};

#[derive(Debug)]
/// merge errors returned by the module-level merge entrypoint.
pub enum MergerError {
    /// wraps loader-level parse/read/target errors.
    Loader(LoaderError),
    /// wraps merger option errors.
    MergerOption(MergerOptionError),
    /// reports invalid merge arguments, such as an empty file list.
    InvalidInput(String),
}

impl fmt::Display for MergerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MergerError::Loader(err) => write!(f, "{err}"),
            MergerError::MergerOption(err) => write!(f, "{err}"),
            MergerError::InvalidInput(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for MergerError {}

impl From<LoaderError> for MergerError {
    fn from(value: LoaderError) -> Self {
        MergerError::Loader(value)
    }
}

impl From<MergerOptionError> for MergerError {
    fn from(value: MergerOptionError) -> Self {
        MergerError::MergerOption(value)
    }
}