//! Hook executor responsible for running lifecycle hooks.

use std::path::PathBuf;
use std::sync::Arc;

use super::{
    HookDecision, HookEvent, HookExecutionRecord, HookLogWriter, HookOutcome, HookRegistry,
    HookScope,
};
use hex;
use sha1::{Digest, Sha1};
use thiserror::Error;
use tracing::warn;

/// Coordinates hook evaluation and logging.
#[derive(Debug, Clone)]
pub struct HookExecutor {
    inner: Arc<HookExecutorInner>,
}

#[derive(Debug)]
struct HookExecutorInner {
    registry: HookRegistry,
    log_writer: Option<HookLogWriter>,
    default_scope: HookScope,
}

impl Default for HookExecutor {
    fn default() -> Self {
        Self {
            inner: Arc::new(HookExecutorInner {
                registry: HookRegistry::default(),
                log_writer: None,
                default_scope: HookScope::LocalUser {
                    codex_home: PathBuf::new(),
                },
            }),
        }
    }
}

impl HookExecutor {
    /// Create an executor with a specific registry snapshot.
    pub fn with_registry(registry: HookRegistry) -> Self {
        Self {
            inner: Arc::new(HookExecutorInner {
                registry,
                log_writer: None,
                default_scope: HookScope::LocalUser {
                    codex_home: PathBuf::new(),
                },
            }),
        }
    }

    /// Create an executor with logging capabilities and a default scope for
    /// synthesized hooks.
    pub fn with_runtime(
        registry: HookRegistry,
        log_writer: HookLogWriter,
        default_scope: HookScope,
    ) -> Self {
        Self {
            inner: Arc::new(HookExecutorInner {
                registry,
                log_writer: Some(log_writer),
                default_scope,
            }),
        }
    }

    /// Returns a clone of the current registry.
    pub fn registry(&self) -> HookRegistry {
        self.inner.registry.clone()
    }

    /// Whether the executor currently has no hooks registered.
    pub fn is_empty(&self) -> bool {
        self.inner.registry.is_empty()
    }

    /// Evaluate pre-tool-use hooks. For now this relies on a placeholder
    /// decision until richer hook execution lands in later tasks.
    pub async fn evaluate_pre_tool_use(
        &self,
        payload: &PreToolUsePayload,
    ) -> Result<HookDecision, HookExecutionError> {
        let decision = if payload.command.contains("rm -rf /var/www") {
            HookDecision {
                decision: HookOutcome::Deny,
                message: Some("Blocking destructive command".to_string()),
                system_message: None,
                stop_reason: Some("dangerous_command".to_string()),
                extra: serde_json::Value::Null,
                exit_code: 2,
            }
        } else {
            HookDecision {
                decision: HookOutcome::Allow,
                message: None,
                system_message: None,
                stop_reason: None,
                extra: serde_json::Value::Null,
                exit_code: 0,
            }
        };

        self.log_decision(
            HookEvent::PreToolUse,
            "codex.pretool.guard",
            &payload.command,
            &decision,
        )
        .await;

        Ok(decision)
    }

    /// Log a synthetic SessionStart hook invocation.
    pub async fn notify_session_start(&self) {
        self.log_allow_event(HookEvent::SessionStart, "codex.session.start", "session")
            .await;
    }

    /// Log a synthetic SessionEnd hook invocation.
    pub async fn notify_session_end(&self) {
        self.log_allow_event(HookEvent::SessionEnd, "codex.session.end", "session")
            .await;
    }

    /// Log a synthetic UserPromptSubmit hook invocation.
    pub async fn notify_user_prompt(&self) {
        self.log_allow_event(
            HookEvent::UserPromptSubmit,
            "codex.user_prompt.submit",
            "user_prompt",
        )
        .await;
    }

    /// Log a synthetic PostToolUse hook invocation to capture execution results.
    pub async fn record_post_tool_use(&self, command: &str, exit_code: i32) {
        let mut decision = HookDecision::default();
        decision.exit_code = exit_code;
        self.log_decision(
            HookEvent::PostToolUse,
            "codex.posttool.audit",
            command,
            &decision,
        )
        .await;
    }

    async fn log_allow_event(&self, event: HookEvent, hook_id: &str, trigger: &str) {
        if self.inner.log_writer.is_none() {
            return;
        }
        self.log_decision(event, hook_id, trigger, &HookDecision::default())
            .await;
    }

    async fn log_decision(
        &self,
        event: HookEvent,
        hook_id: &str,
        trigger: &str,
        decision: &HookDecision,
    ) {
        let Some(writer) = &self.inner.log_writer else {
            return;
        };

        let mut record = HookExecutionRecord::new(event, self.inner.default_scope.clone(), hook_id);
        record.decision = decision.clone();
        record.payload_hash = hash_payload(trigger);
        record.trigger_id = trigger.to_string();

        if let Err(err) = writer.append(&record).await {
            warn!("failed to append hook execution record: {err}");
        }
    }
}

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

fn hash_payload(input: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}
