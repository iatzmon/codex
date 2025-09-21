//! HookDefinition describes lifecycle hook configuration entries.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::{HookEvent, HookMatchers, HookScope};

/// Declarative hook definition combining matchers and execution metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HookDefinition {
    pub id: String,
    pub event: HookEvent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(default)]
    pub command: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    #[serde(default)]
    pub allow_parallel: bool,
    pub schema_versions: Vec<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub matchers: HookMatchers,
    #[serde(skip)]
    pub scope: HookScope,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub source_path: Option<PathBuf>,
}

impl HookDefinition {
    pub fn new(id: impl Into<String>, event: HookEvent, scope: HookScope) -> Self {
        Self {
            id: id.into(),
            event,
            notes: None,
            command: Vec::new(),
            working_dir: None,
            timeout_ms: None,
            allow_parallel: false,
            schema_versions: vec!["1.0".to_string()],
            env: HashMap::new(),
            matchers: HookMatchers::default(),
            scope,
            source_path: None,
        }
    }
}

impl Default for HookDefinition {
    fn default() -> Self {
        Self {
            id: String::new(),
            event: HookEvent::PreToolUse,
            notes: None,
            command: Vec::new(),
            working_dir: None,
            timeout_ms: None,
            allow_parallel: false,
            schema_versions: Vec::new(),
            env: HashMap::new(),
            matchers: HookMatchers::default(),
            scope: HookScope::default(),
            source_path: None,
        }
    }
}
