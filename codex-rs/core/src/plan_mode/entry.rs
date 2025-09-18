use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

/// Categories of plan entries that describe the type of proposal captured
/// while operating in Plan Mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanEntryType {
    Command,
    FileChange,
    Research,
    Decision,
}

impl Default for PlanEntryType {
    fn default() -> Self {
        Self::Research
    }
}

/// Individual item captured in the plan artifact instead of executing a
/// command or mutating the workspace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanEntry {
    pub sequence: u16,
    pub entry_type: PlanEntryType,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Default for PlanEntry {
    fn default() -> Self {
        Self::new(PlanEntryType::Research, String::new())
    }
}

impl PlanEntry {
    /// Construct a new plan entry with the current timestamp and no details.
    pub fn new(entry_type: PlanEntryType, summary: impl Into<String>) -> Self {
        Self {
            sequence: 0,
            entry_type,
            summary: summary.into(),
            details: None,
            created_at: Utc::now(),
        }
    }

    /// Attach additional detail to the entry.
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}
