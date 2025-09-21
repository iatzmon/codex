//! Aggregates hook definitions grouped by event and precedence.

use std::collections::HashMap;

use super::{HookDefinition, HookEvent};
use chrono::{DateTime, Utc};

use super::layer_summary::HookLayerSummary;

/// Primary runtime view of configured hooks.
#[derive(Debug, Default)]
pub struct HookRegistry {
    pub events: HashMap<HookEvent, Vec<HookDefinition>>,
    pub last_loaded: Option<DateTime<Utc>>,
    pub source_layers: Vec<HookLayerSummary>,
}

impl HookRegistry {
    pub fn new() -> Self {
        Self::default()
    }
}
