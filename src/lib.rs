//! aethellib provides loaders, merge helpers, and generators for aethel datasets.

/// generation module entrypoint.
pub mod generator;
/// loader module entrypoint.
pub mod loader;
/// merge module entrypoint.
pub mod merger;

#[cfg(test)]
/// shared test helpers for inline fixtures and temp files.
pub(crate) mod test_support;
