//! Shared types for the Codex hook system.

pub mod decision;
pub mod definition;
pub mod execution_record;
pub mod matchers;
pub mod payload;
pub mod scope;

// Re-export the most common types for convenience.
pub use decision::{HookDecision, HookOutcome};
pub use definition::HookDefinition;
pub use execution_record::HookExecutionRecord;
pub use matchers::{HookMatcher, HookMatchers};
pub use payload::{HookEvent, HookEventPayload, SandboxContext, SandboxMode};
pub use scope::HookScope;
