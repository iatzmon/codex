//! Aggregates hook definitions grouped by event and precedence.

use std::collections::HashMap;

use chrono::{DateTime, Utc};

use super::layer_summary::HookLayerSummary;
use super::{HookDefinition, HookEvent, HookScope};

/// Primary runtime view of configured hooks.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct HookRegistry {
    pub events: HashMap<HookEvent, Vec<HookDefinition>>,
    pub last_loaded: Option<DateTime<Utc>>,
    pub source_layers: Vec<HookLayerSummary>,
}

impl HookRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a registry from definitions and layer summaries captured at a
    /// specific load time.
    pub fn with_layers(
        definitions: Vec<HookDefinition>,
        layer_summaries: Vec<HookLayerSummary>,
        loaded_at: DateTime<Utc>,
    ) -> Self {
        let mut registry = Self {
            events: HashMap::new(),
            last_loaded: Some(loaded_at),
            source_layers: layer_summaries,
        };
        registry.insert(definitions);
        registry
    }

    /// Merge additional hook definitions into the registry.
    pub fn insert(&mut self, definitions: Vec<HookDefinition>) {
        for definition in definitions {
            let event = definition.event.clone();
            self.events.entry(event).or_default().push(definition);
        }

        for bucket in self.events.values_mut() {
            bucket.sort_by(|a, b| {
                precedence_rank(&a.scope)
                    .cmp(&precedence_rank(&b.scope))
                    .then_with(|| a.id.cmp(&b.id))
            });
        }
    }

    /// Retrieve hooks for the given event ordered by precedence.
    pub fn hooks_for_event(&self, event: &HookEvent) -> &[HookDefinition] {
        static EMPTY: [HookDefinition; 0] = [];
        self.events
            .get(event)
            .map(|bucket| bucket.as_slice())
            .unwrap_or(&EMPTY)
    }

    /// Total number of hooks contained in the registry.
    pub fn len(&self) -> usize {
        self.events.values().map(|bucket| bucket.len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

fn precedence_rank(scope: &HookScope) -> u8 {
    match scope {
        HookScope::ManagedPolicy { .. } => 0,
        HookScope::Project { .. } => 1,
        HookScope::LocalUser { .. } => 2,
    }
}
