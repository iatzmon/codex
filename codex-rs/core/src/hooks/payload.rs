//! Hook event payload serialized to JSON and delivered to hook scripts.

use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Lifecycle events that may trigger hooks.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "PascalCase")]
pub enum HookEvent {
    PreToolUse,
    PostToolUse,
    UserPromptSubmit,
    Notification,
    Stop,
    SubagentStop,
    PreCompact,
    SessionStart,
    SessionEnd,
}

/// Runtime sandbox metadata surfaced to hooks.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SandboxContext {
    pub mode: SandboxMode,
    pub network_access: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub writable_roots: Vec<PathBuf>,
}

/// Supported sandbox modes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SandboxMode {
    WorkspaceWrite,
    DangerFullAccess,
    ReadOnly,
}

/// Hook payload delivered over stdin to external processes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HookEventPayload {
    pub schema_version: String,
    pub event: HookEvent,
    pub session_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub workspace_root: PathBuf,
    #[serde(alias = "currentWorkingDirectory")]
    pub cwd: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcript_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox: Option<SandboxContext>,
    #[serde(default)]
    pub event_context: Value,
}

impl HookEventPayload {
    pub fn with_event(event: HookEvent) -> Self {
        Self {
            schema_version: "1.0".to_string(),
            event,
            session_id: Uuid::nil(),
            timestamp: Utc::now(),
            workspace_root: PathBuf::new(),
            cwd: PathBuf::new(),
            transcript_path: None,
            schema: None,
            sandbox: None,
            event_context: Value::Null,
        }
    }
}
