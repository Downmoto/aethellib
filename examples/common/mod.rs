//! shared helpers for self-contained example fixtures.
//! this keeps examples independent from repository data files.

#![allow(dead_code)]

use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// temporary toml file removed automatically when dropped.
pub struct TempTomlFile {
    path: PathBuf,
}

impl TempTomlFile {
    /// creates a new temporary toml file with provided content.
    pub fn new(content: &str) -> Self {
        let sequence = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let file_name = format!("aethellib_example_{nanos}_{sequence}.toml");
        let path = std::env::temp_dir().join(file_name);

        fs::write(&path, content).expect("failed to write temporary toml example file");

        Self { path }
    }

    /// returns the temporary file path as a utf-8 string.
    pub fn path_str(&self) -> &str {
        self.path
            .to_str()
            .expect("temporary example file path must be valid utf-8")
    }
}

impl Drop for TempTomlFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// builds a minimal toml document with a header and optional body.
pub fn toml_document(name: &str, target: &str, body: &str) -> String {
    let trimmed_body = body.trim();

    if trimmed_body.is_empty() {
        format!("[header]\nname = \"{name}\"\ntarget = \"{target}\"\n")
    } else {
        format!(
            "[header]\nname = \"{name}\"\ntarget = \"{target}\"\n\n{trimmed_body}\n"
        )
    }
}

/// builds a minimal weapon document body from the given toml sections.
pub fn weapon_document(name: &str, body: &str) -> String {
    toml_document(name, "weapon", body)
}

/// builds a minimal person document body from the given toml sections.
pub fn person_document(name: &str, body: &str) -> String {
    toml_document(name, "person", body)
}
