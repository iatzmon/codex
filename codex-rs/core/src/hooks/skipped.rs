//! Records reasons for skipping hook definitions during load.

use serde::{Deserialize, Serialize};

/// Reason a hook could not be loaded from configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum HookSkipReason {
    InvalidSchema,
    UnsupportedVersion,
    DuplicateId,
    MissingExecutable,
    InvalidMatcher,
}

/// Record describing a skipped hook entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SkippedHook {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hook_id: Option<String>,
    pub reason: HookSkipReason,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl SkippedHook {
    pub fn new(reason: HookSkipReason) -> Self {
        Self {
            hook_id: None,
            reason,
            details: None,
        }
    }

    pub fn with_hook_id(mut self, id: impl Into<String>) -> Self {
        self.hook_id = Some(id.into());
        self
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}
