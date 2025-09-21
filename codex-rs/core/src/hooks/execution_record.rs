//! Structured record emitted for every hook execution.

use std::collections::VecDeque;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{HookDecision, HookEvent, HookScope};

/// Structured audit log entry for each hook invocation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HookExecutionRecord {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event: HookEvent,
    pub scope: HookScope,
    pub hook_id: String,
    pub decision: HookDecision,
    pub duration_ms: u128,
    pub stdout: VecDeque<String>,
    pub stderr: VecDeque<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub precedence_rank: u8,
    pub payload_hash: String,
    pub trigger_id: String,
}

impl HookExecutionRecord {
    pub fn new(event: HookEvent, scope: HookScope, hook_id: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event,
            scope,
            hook_id: hook_id.into(),
            decision: HookDecision::default(),
            duration_ms: 0,
            stdout: VecDeque::new(),
            stderr: VecDeque::new(),
            error: None,
            precedence_rank: 0,
            payload_hash: String::new(),
            trigger_id: String::new(),
        }
    }
}
