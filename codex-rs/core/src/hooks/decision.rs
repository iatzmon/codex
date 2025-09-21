//! Decision types returned by hook executions.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Normalized decision returned from a hook execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HookDecision {
    pub decision: HookOutcome,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Value::is_null")]
    pub extra: Value,
    pub exit_code: i32,
}

impl Default for HookDecision {
    fn default() -> Self {
        Self {
            decision: HookOutcome::Allow,
            message: None,
            system_message: None,
            stop_reason: None,
            extra: Value::Null,
            exit_code: 0,
        }
    }
}

/// Possible outcomes from a hook.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum HookOutcome {
    Allow,
    Ask,
    Deny,
    Block,
    Continue,
}
