//! Metadata for each configuration layer contributing hooks.

use std::path::PathBuf;

use super::HookScope;

use super::skipped::SkippedHook;

/// Captures load metadata for a single configuration layer.
#[derive(Debug, Clone, PartialEq)]
pub struct HookLayerSummary {
    pub scope: HookScope,
    pub path: PathBuf,
    pub checksum: String,
    pub loaded_hooks: usize,
    pub skipped_hooks: Vec<SkippedHook>,
}

impl HookLayerSummary {
    pub fn new(scope: HookScope, path: PathBuf) -> Self {
        Self {
            scope,
            path,
            checksum: String::new(),
            loaded_hooks: 0,
            skipped_hooks: Vec::new(),
        }
    }
}
