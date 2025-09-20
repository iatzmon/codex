use serde::Deserialize;
use serde::Serialize;
use ts_rs::TS;
use uuid::Uuid;

use crate::protocol::AskForApproval;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum PlanEntryTypePayload {
    Command,
    FileChange,
    Research,
    Decision,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct PlanEntryPayload {
    pub sequence: u16,
    pub entry_type: PlanEntryTypePayload,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct PlanArtifactPayload {
    pub title: String,
    pub objectives: Vec<String>,
    pub constraints: Vec<String>,
    pub assumptions: Vec<String>,
    pub approach: Vec<String>,
    pub steps: Vec<PlanEntryPayload>,
    pub affected_areas: Vec<String>,
    pub risks: Vec<String>,
    pub next_actions: Vec<String>,
    pub tests: Vec<String>,
    pub alternatives: Vec<String>,
    pub rollback: Vec<String>,
    pub success_criteria: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct PlanModeSessionPayload {
    pub session_id: Uuid,
    pub entered_from: AskForApproval,
    pub allowed_tools: Vec<String>,
    pub plan_artifact: PlanArtifactPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct PlanModeActivatedEvent {
    pub session: PlanModeSessionPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct PlanModeUpdatedEvent {
    pub session: PlanModeSessionPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct PlanModeExitedEvent {
    pub previous_mode: AskForApproval,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct PlanModeAppliedEvent {
    pub target_mode: AskForApproval,
    pub plan_entries: usize,
}
