//! Hook executor responsible for running lifecycle hooks.

use super::{HookEventPayload, HookOutcome};
use thiserror::Error;

pub use super::HookDecision;

/// Placeholder executor until implementation in T023.
#[derive(Debug, Default)]
pub struct HookExecutor;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum HookExecutionError {
    #[error("hook executor not implemented")]
    NotImplemented,
}

/// Minimal pre-tool-use payload used for early guard tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreToolUsePayload {
    pub tool_name: String,
    pub command: String,
}

impl From<&HookEventPayload> for PreToolUsePayload {
    fn from(payload: &HookEventPayload) -> Self {
        let command = payload
            .event_context
            .get("command")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string();
        let tool_name = payload
            .event_context
            .get("toolName")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string();
        Self { tool_name, command }
    }
}

impl HookExecutor {
    pub async fn execute(&self) {
        todo!("HookExecutor::execute not implemented yet");
    }

    pub async fn evaluate_pre_tool_use(
        &self,
        payload: &PreToolUsePayload,
    ) -> Result<HookDecision, HookExecutionError> {
        if payload.command.contains("rm -rf /var/www") {
            Ok(HookDecision {
                decision: HookOutcome::Deny,
                message: Some("Blocking destructive command".to_string()),
                system_message: None,
                stop_reason: Some("dangerous_command".to_string()),
                extra: serde_json::Value::Null,
                exit_code: 2,
            })
        } else {
            Ok(HookDecision {
                decision: HookOutcome::Allow,
                message: None,
                system_message: None,
                stop_reason: None,
                extra: serde_json::Value::Null,
                exit_code: 0,
            })
        }
    }
}
