//! Matcher definitions used to filter hook execution.

use serde::{Deserialize, Serialize};

/// Supported matcher types for hook predicates.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum HookMatcher {
    Exact { value: String },
    Glob { value: String },
    Regex { value: String },
}

/// Aggregate matcher configuration grouped by fields.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HookMatchers {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_names: Vec<HookMatcher>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<HookMatcher>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub paths: Vec<HookMatcher>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}
