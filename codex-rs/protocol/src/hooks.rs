use std::collections::HashMap;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, TS)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum HookScope {
    ManagedPolicy {
        name: String,
    },
    Project {
        #[ts(type = "string")]
        project_root: PathBuf,
    },
    LocalUser {
        #[ts(type = "string")]
        codex_home: PathBuf,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum HookMatcher {
    Exact { value: String },
    Glob { value: String },
    Regex { value: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS, Default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[serde(rename_all = "camelCase")]
pub struct HookDefinition {
    pub id: String,
    pub event: HookEvent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(default)]
    pub command: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
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
    pub scope: HookScope,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub source_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
pub struct HookLayerSummary {
    pub scope: HookScope,
    #[ts(type = "string")]
    pub path: PathBuf,
    pub checksum: String,
    pub loaded_hooks: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skipped_hooks: Vec<SkippedHook>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
pub struct SkippedHook {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hook_id: Option<String>,
    pub reason: HookSkipReason,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
pub enum HookSkipReason {
    InvalidSchema,
    UnsupportedVersion,
    DuplicateId,
    MissingExecutable,
    InvalidMatcher,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, Default)]
#[serde(rename_all = "camelCase")]
pub struct HookRegistrySnapshot {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub events: HashMap<HookEvent, Vec<HookDefinition>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layers: Vec<HookLayerSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub last_loaded: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "PascalCase")]
pub enum HookOutcome {
    Allow,
    Ask,
    Deny,
    Block,
    Continue,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
pub struct HookExecutionRecord {
    pub id: Uuid,
    #[ts(type = "string")]
    pub timestamp: DateTime<Utc>,
    pub event: HookEvent,
    pub scope: HookScope,
    pub hook_id: String,
    pub decision: HookDecision,
    pub duration_ms: u128,
    pub stdout: Vec<String>,
    pub stderr: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub precedence_rank: u8,
    pub payload_hash: String,
    pub trigger_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
pub struct HookListRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<HookEvent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<HookScopeFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "lowercase")]
pub enum HookScopeFilter {
    Managed,
    Project,
    Local,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
pub struct HookExecLogRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub since: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<HookEvent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hook_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tail: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
pub struct HookValidateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<HookScopeFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
pub enum HookValidationStatus {
    Ok,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
pub struct HookValidationSummary {
    pub status: HookValidationStatus,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layers: Vec<HookLayerSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
pub struct HookReloadResponse {
    pub reloaded: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
pub struct HookExecLogResponse {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub records: Vec<HookExecutionRecord>,
}
