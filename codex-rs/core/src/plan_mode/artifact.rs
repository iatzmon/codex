use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

use super::PlanEntry;
use super::PlanEntryType;
use codex_protocol::plan_mode::PlanArtifactMetadataPayload;
use codex_protocol::plan_mode::PlanArtifactPayload;
use codex_protocol::plan_mode::PlanEntryPayload;

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
    pub approach: Vec<String>,
    pub steps: Vec<PlanEntry>,
    pub affected_areas: Vec<String>,
    pub risks: Vec<String>,
    pub alternatives: Vec<String>,
    pub next_actions: Vec<String>,
    pub tests: Vec<String>,
    pub rollback: Vec<String>,
    pub success_criteria: Vec<String>,
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

    /// Render the artifact into a human-readable Markdown summary that can be
    /// injected into the conversation transcript after `/apply-plan`.
    pub fn to_summary_markdown(&self) -> String {
        fn push_section(lines: &mut Vec<String>, label: &str, items: &[String]) {
            if items.is_empty() {
                return;
            }
            lines.push(format!("{label}:"));
            for item in items {
                if item.is_empty() {
                    continue;
                }
                lines.push(format!("- {item}"));
            }
            lines.push(String::new());
        }

        let mut lines: Vec<String> = Vec::new();

        if self.title.is_empty() {
            lines.push("Plan Ready for Execution".to_string());
        } else {
            lines.push(format!("Plan Ready for Execution: {}", self.title));
        }
        lines.push(String::new());

        push_section(&mut lines, "Objectives", &self.objectives);
        push_section(&mut lines, "Constraints", &self.constraints);
        push_section(&mut lines, "Assumptions", &self.assumptions);
        push_section(&mut lines, "Approach", &self.approach);

        if !self.steps.is_empty() {
            lines.push("Steps:".to_string());
            for (index, entry) in self.steps.iter().enumerate() {
                let number = index + 1;
                let summary = entry.summary.trim();
                lines.push(format!("{number}. {summary}"));
                if let Some(details) = &entry.details {
                    for detail_line in details.lines() {
                        if detail_line.trim().is_empty() {
                            continue;
                        }
                        lines.push(format!("    - {detail_line}"));
                    }
                }
            }
            lines.push(String::new());
        }

        push_section(&mut lines, "Affected Areas", &self.affected_areas);
        push_section(&mut lines, "Risks", &self.risks);
        push_section(&mut lines, "Alternatives", &self.alternatives);
        push_section(&mut lines, "Rollback / Mitigations", &self.rollback);
        push_section(&mut lines, "Success Criteria", &self.success_criteria);
        push_section(&mut lines, "Tests", &self.tests);
        push_section(&mut lines, "Next Actions", &self.next_actions);

        lines.push(
            "Reminder: Save this plan with `/save-plan <path>` before leaving the planning context.".to_string(),
        );

        // Remove trailing blank lines for cleaner output.
        while matches!(lines.last(), Some(line) if line.trim().is_empty()) {
            lines.pop();
        }

        lines.join("\n")
    }
}

impl From<&PlanArtifact> for PlanArtifactPayload {
    fn from(artifact: &PlanArtifact) -> Self {
        let metadata = if artifact.metadata.template.is_none()
            && artifact.metadata.model.is_none()
            && artifact.metadata.updated_at.is_none()
        {
            None
        } else {
            Some(PlanArtifactMetadataPayload {
                template: artifact.metadata.template.clone(),
                model: artifact.metadata.model.clone(),
                updated_at: artifact
                    .metadata
                    .updated_at
                    .map(|timestamp| timestamp.to_rfc3339()),
            })
        };

        Self {
            title: artifact.title.clone(),
            objectives: artifact.objectives.clone(),
            constraints: artifact.constraints.clone(),
            assumptions: artifact.assumptions.clone(),
            approach: artifact.approach.clone(),
            steps: artifact.steps.iter().map(PlanEntryPayload::from).collect(),
            affected_areas: artifact.affected_areas.clone(),
            risks: artifact.risks.clone(),
            alternatives: artifact.alternatives.clone(),
            next_actions: artifact.next_actions.clone(),
            tests: artifact.tests.clone(),
            rollback: artifact.rollback.clone(),
            success_criteria: artifact.success_criteria.clone(),
            metadata,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan_mode::PlanEntryType;

    #[test]
    fn summary_markdown_includes_required_sections() {
        let mut artifact = PlanArtifact {
            title: "Ship Plan Mode".to_string(),
            objectives: vec!["Keep workspace read-only".to_string()],
            constraints: vec!["No shell access".to_string()],
            assumptions: vec!["Sandbox enforced".to_string()],
            approach: vec!["Collect requirements".to_string()],
            steps: Vec::new(),
            affected_areas: vec!["core/session".to_string()],
            risks: vec!["User confusion".to_string()],
            alternatives: vec!["Allow writes with confirmations".to_string()],
            next_actions: vec!["Exit Plan Mode".to_string()],
            tests: vec!["Run plan_mode suite".to_string()],
            rollback: vec!["Restore from git".to_string()],
            success_criteria: vec!["All specs satisfied".to_string()],
            metadata: PlanArtifactMetadata::default(),
        };
        artifact.push_summary_entry(PlanEntryType::Command, "Draft implementation");

        let summary = artifact.to_summary_markdown();
        assert!(summary.contains("Objectives:"));
        assert!(summary.contains("Constraints:"));
        assert!(summary.contains("Steps:"));
        assert!(summary.contains("Rollback / Mitigations:"));
        assert!(summary.contains("Reminder: Save this plan"));
    }
}
