//! Hook runtime components for Codex.
//!
//! This module will load hook configurations, manage precedence across layers,
//! and execute lifecycle hooks with structured logging.

pub mod config_loader;
pub mod decision;
pub mod definition;
pub mod execution_record;
pub mod executor;
pub mod layer_summary;
pub mod log_writer;
pub mod matchers;
pub mod payload;
pub mod registry;
pub mod schema_registry;
pub mod scope;
pub mod skipped;

pub use decision::{HookDecision, HookOutcome};
pub use definition::HookDefinition;
pub use execution_record::HookExecutionRecord;
pub use layer_summary::HookLayerSummary;
pub use log_writer::HookLogWriter;
pub use matchers::{HookMatcher, HookMatchers};
pub use payload::{HookEvent, HookEventPayload, SandboxContext, SandboxMode};
pub use registry::HookRegistry;
pub use scope::HookScope;
