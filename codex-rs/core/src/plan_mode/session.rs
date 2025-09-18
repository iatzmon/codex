use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use crate::protocol::AskForApproval;

use super::PlanArtifact;
use super::PlanArtifactMetadata;
use super::PlanEntry;
use super::PlanEntryType;
use super::PlanModeConfig;
use super::PlanModeEvent;
use super::PlanTelemetry;
use super::ToolCapability;
use super::ToolMode;

/// High-level state of a session that is currently operating in Plan Mode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanModeSession {
    pub session_id: Uuid,
    pub entered_from: AskForApproval,
    pub state: PlanModeState,
    pub allowed_tools: Vec<ToolCapability>,
    pub plan_artifact: PlanArtifact,
    pub entered_at: DateTime<Utc>,
    pub pending_exit: Option<AskForApproval>,
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
    ) -> Self {
        let allowed_tools = filter_allowed_tools(tool_capabilities, config);

        Self {
            session_id,
            entered_from,
            state: PlanModeState::Active,
            allowed_tools,
            plan_artifact: PlanArtifact::default(),
            entered_at: Utc::now(),
            pending_exit: None,
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

    /// Determine whether Plan Mode can only expose read-only capabilities with
    /// the current network policy.
    pub fn allowed_tool_ids(&self) -> impl Iterator<Item = &str> {
        self.allowed_tools.iter().map(|cap| cap.id.as_str())
    }

    /// Snapshot telemetry for entering Plan Mode.
    pub fn entered_telemetry(&self) -> PlanTelemetry {
        PlanTelemetry::new(
            PlanModeEvent::Entered,
            self.entered_from,
            self.plan_artifact.entry_count(),
        )
    }

    #[allow(dead_code)]
    pub fn plan_artifact(&self) -> &PlanArtifact {
        &self.plan_artifact
    }
}

fn filter_allowed_tools(
    capabilities: Vec<ToolCapability>,
    config: &PlanModeConfig,
) -> Vec<ToolCapability> {
    let has_explicit_allow_list = !config.allowed_read_only_tools.is_empty();
    capabilities
        .into_iter()
        .filter(|capability| capability.mode == ToolMode::ReadOnly)
        .filter(|capability| {
            if !capability.requires_network {
                return true;
            }
            has_explicit_allow_list
                && config
                    .allowed_read_only_tools
                    .iter()
                    .any(|allowed| allowed == &capability.id)
        })
        .filter(|capability| {
            if !has_explicit_allow_list {
                return true;
            }
            config
                .allowed_read_only_tools
                .iter()
                .any(|allowed| allowed == &capability.id)
        })
        .collect()
}

/// Lifecycle states for the Plan Mode session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanModeState {
    Active,
    Applying,
    Exited,
}
