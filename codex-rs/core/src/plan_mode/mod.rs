//! Core data structures used to power Plan Mode.

mod allow_list;
mod artifact;
mod capability;
mod config;
mod entry;
mod session;
mod telemetry;

/// System message inserted when Plan Mode is active to steer the model
/// toward planning instead of execution.
pub const PLAN_MODE_SYSTEM_PROMPT: &str = "Plan Mode is active. Assume every user request requires a detailed implementation plan. Before calling `update_plan`, investigate the codebase with read-only tools, inspect relevant files, and search the web or docs as needed to collect concrete facts. Use that research to draft a step-by-step plan with specific file changes, commands, risks, validation, and follow-up work. Capture the finished plan with `update_plan` only after research is complete, and do not execute or apply changes until the plan is explicitly approved.";

pub use allow_list::PlanModeAllowList;
pub use artifact::PlanArtifact;
pub use artifact::PlanArtifactMetadata;
pub use capability::ToolCapability;
pub use capability::ToolMode;
pub use config::PlanModeConfig;
pub use entry::PlanEntry;
pub use entry::PlanEntryType;
pub use session::PlanModeSession;
pub use session::PlanModeState;
pub use telemetry::PlanModeEvent;
pub use telemetry::PlanTelemetry;
