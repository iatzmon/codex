use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

use crate::protocol::AskForApproval;

/// Events reported while the session is operating in Plan Mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanModeEvent {
    Entered,
    RefusalCaptured,
    ApplySuccess,
    Exit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanTelemetry {
    pub event: PlanModeEvent,
    pub previous_mode: AskForApproval,
    pub plan_entry_count: usize,
    pub occurred_at: DateTime<Utc>,
}

impl PlanTelemetry {
    pub fn new(
        event: PlanModeEvent,
        previous_mode: AskForApproval,
        plan_entry_count: usize,
    ) -> Self {
        Self {
            event,
            previous_mode,
            plan_entry_count,
            occurred_at: Utc::now(),
        }
    }
}
