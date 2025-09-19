use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use crate::protocol::AskForApproval;
use codex_protocol::plan_mode::PlanArtifactPayload;
use codex_protocol::plan_mode::PlanModeSessionPayload;

use super::PlanArtifact;
use super::PlanArtifactMetadata;
use super::PlanEntry;
use super::PlanEntryType;
use super::PlanModeAllowList;
use super::PlanModeConfig;
use super::PlanModeEvent;
use super::PlanTelemetry;
use super::ToolCapability;
use super::ToolMode;

/// High-level state of a session that is currently operating in Plan Mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanModeSession {
    pub session_id: Uuid,
    pub entered_from: AskForApproval,
    pub state: PlanModeState,
    pub allowed_tools: Vec<String>,
    pub plan_artifact: PlanArtifact,
    pub entered_at: DateTime<Utc>,
    pub pending_exit: Option<AskForApproval>,
    #[serde(skip, default)]
    allow_list: PlanModeAllowList,
    #[serde(skip, default)]
    fallback_tool_ids: Vec<String>,
}

impl PlanModeSession {
    /// Construct a new Plan Mode session using the provided configuration and
    /// tool registry. The tool list is filtered to retain only read-only
    /// capabilities permitted by the overrides.
    pub fn new(
        session_id: Uuid,
        entered_from: AskForApproval,
        tool_capabilities: Vec<ToolCapability>,
        config: &PlanModeConfig,
        network_enabled: bool,
    ) -> Self {
        let allow_list = config.allow_list();
        let fallback_tool_ids: Vec<String> = tool_capabilities
            .into_iter()
            .filter(|capability| capability.mode == ToolMode::ReadOnly)
            .filter(|capability| {
                if !capability.requires_network {
                    return true;
                }
                network_enabled
                    && allow_list.has_tool_rules()
                    && allow_list.matches_tool(&capability.id)
            })
            .map(|capability| capability.id)
            .collect();

        let mut allowed_tools = allow_list.raw_entries().to_vec();
        if !allow_list.has_tool_rules() {
            allowed_tools.extend(fallback_tool_ids.clone());
        }

        Self {
            session_id,
            entered_from,
            state: PlanModeState::Active,
            allowed_tools,
            plan_artifact: PlanArtifact::default(),
            entered_at: Utc::now(),
            pending_exit: None,
            allow_list,
            fallback_tool_ids,
        }
    }

    /// Record a refusal by capturing the provided summary/details into the
    /// artifact and returning the updated telemetry payload.
    pub fn record_refusal(
        &mut self,
        entry_type: PlanEntryType,
        summary: impl Into<String>,
        details: Option<String>,
    ) -> PlanTelemetry {
        let mut entry = PlanEntry::new(entry_type, summary);
        if let Some(details) = details {
            entry = entry.with_details(details);
        }
        self.plan_artifact.add_entry(entry);
        PlanTelemetry::new(
            PlanModeEvent::RefusalCaptured,
            self.entered_from,
            self.plan_artifact.entry_count(),
        )
    }

    /// Update metadata for the plan artifact, typically after loading a
    /// template or switching models.
    pub fn set_artifact_metadata(&mut self, metadata: PlanArtifactMetadata) {
        self.plan_artifact.metadata = metadata;
    }

    /// Begin the apply flow. The state is marked as `Applying` and the desired
    /// approval mode is cached for later use when the transition completes.
    pub fn begin_apply(&mut self, target_mode: Option<AskForApproval>) {
        self.state = PlanModeState::Applying;
        self.pending_exit = target_mode;
    }

    /// Mark the session as fully exited and clear any pending overrides.
    pub fn exit_plan_mode(&mut self) {
        self.state = PlanModeState::Exited;
        self.pending_exit = None;
    }

    /// Convenience accessor to inspect whether the session is still active.
    pub fn is_active(&self) -> bool {
        matches!(self.state, PlanModeState::Active)
    }

    /// Raw allow-list entries for display purposes.
    pub fn allowed_tool_ids(&self) -> impl Iterator<Item = &str> {
        self.allowed_tools.iter().map(|entry| entry.as_str())
    }

    pub fn is_tool_allowed(&self, tool_id: &str) -> bool {
        if self.allow_list.has_tool_rules() {
            self.allow_list.matches_tool(tool_id)
        } else {
            self.fallback_tool_ids.iter().any(|id| id == tool_id)
        }
    }

    pub fn is_shell_allowed(&self, command: &str) -> bool {
        self.allow_list.matches_shell_command(command)
    }

    /// Snapshot telemetry for entering Plan Mode.
    pub fn entered_telemetry(&self) -> PlanTelemetry {
        PlanTelemetry::new(
            PlanModeEvent::Entered,
            self.entered_from,
            self.plan_artifact.entry_count(),
        )
    }

    pub fn to_payload(&self) -> PlanModeSessionPayload {
        PlanModeSessionPayload {
            session_id: self.session_id,
            entered_from: self.entered_from,
            allowed_tools: self.allowed_tools.clone(),
            plan_artifact: PlanArtifactPayload::from(&self.plan_artifact),
        }
    }

    #[allow(dead_code)]
    pub fn plan_artifact(&self) -> &PlanArtifact {
        &self.plan_artifact
    }
}

/// Lifecycle states for the Plan Mode session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanModeState {
    Active,
    Applying,
    Exited,
}
