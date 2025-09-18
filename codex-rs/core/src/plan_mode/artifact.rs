use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

use super::PlanEntry;
use super::PlanEntryType;

/// Additional metadata associated with a planning artifact, including
/// information required to reconstruct the original template or model
/// selection.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanArtifactMetadata {
    /// Optional name of the template that seeded the plan.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    /// Optional identifier for the model used while gathering the plan.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Timestamp indicating when the plan metadata was last updated.
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
}

/// Structured document that captures the agent's plan while operating in
/// Plan Mode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PlanArtifact {
    pub title: String,
    pub objectives: Vec<String>,
    pub constraints: Vec<String>,
    pub assumptions: Vec<String>,
    pub steps: Vec<PlanEntry>,
    pub risks: Vec<String>,
    pub next_actions: Vec<String>,
    pub tests: Vec<String>,
    pub metadata: PlanArtifactMetadata,
}

impl PlanArtifact {
    /// Append a new entry to the plan, automatically assigning the next
    /// sequence number when one was not provided.
    pub fn add_entry(&mut self, mut entry: PlanEntry) {
        if entry.sequence == 0 {
            entry.sequence = self.next_sequence();
        }
        self.steps.push(entry);
    }

    /// Helper to add a simple entry based on free-form content.
    pub fn push_summary_entry(&mut self, entry_type: PlanEntryType, summary: impl Into<String>) {
        let mut entry = PlanEntry::new(entry_type, summary.into());
        entry.sequence = self.next_sequence();
        self.steps.push(entry);
    }

    /// Retrieve the number of plan steps recorded so far.
    pub fn entry_count(&self) -> usize {
        self.steps.len()
    }

    fn next_sequence(&self) -> u16 {
        self.steps
            .last()
            .map(|entry| entry.sequence.saturating_add(1))
            .unwrap_or(1)
    }
}
