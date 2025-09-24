use std::collections::BTreeMap;

use crate::subagents::config::SubagentConfig;
use crate::subagents::definition::{SubagentDefinition, SubagentScope};
use crate::subagents::record::{SubagentRecord, SubagentStatus};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubagentConflict {
    pub name: String,
    pub losing_scope: SubagentScope,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscoveryEvent {
    pub message: String,
}

#[derive(Clone, Debug, Default)]
pub struct SubagentInventory {
    pub subagents: BTreeMap<String, SubagentRecord>,
    pub conflicts: Vec<SubagentConflict>,
    pub discovery_events: Vec<DiscoveryEvent>,
    invalid_records: Vec<SubagentRecord>,
}

impl SubagentInventory {
    pub fn invalid(&self) -> Vec<&SubagentRecord> {
        self.invalid_records.iter().collect()
    }

    pub fn from_definitions<I>(config: &SubagentConfig, definitions: I) -> Self
    where
        I: IntoIterator<Item = SubagentDefinition>,
    {
        let mut inventory = SubagentInventory::default();
        if !config.enabled {
            inventory.discovery_events.push(DiscoveryEvent {
                message: "subagents feature disabled via configuration".to_string(),
            });
            return inventory;
        }

        let mut grouped: BTreeMap<String, Vec<SubagentDefinition>> = BTreeMap::new();
        for definition in definitions {
            let key = if definition.name.is_empty() {
                SubagentDefinition::normalize_name(&definition.raw_name)
            } else {
                definition.name.clone()
            };
            grouped.entry(key).or_default().push(definition);
        }

        for (name, mut defs) in grouped {
            defs.sort_by(|a, b| scope_precedence(b.scope).cmp(&scope_precedence(a.scope)));

            let records: Vec<SubagentRecord> = defs
                .into_iter()
                .map(|definition| SubagentRecord::from_definition(definition, config))
                .collect();

            let chosen_idx = records
                .iter()
                .position(|record| matches!(record.status, SubagentStatus::Active));

            if let Some(idx) = chosen_idx {
                inventory
                    .subagents
                    .insert(name.clone(), records[idx].clone());
            }

            let chosen_scope = chosen_idx.map(|idx| records[idx].definition.scope);

            for (idx, record) in records.iter().enumerate() {
                match record.status {
                    SubagentStatus::Invalid => inventory.invalid_records.push(record.clone()),
                    SubagentStatus::Disabled => inventory.discovery_events.push(DiscoveryEvent {
                        message: format!(
                            "subagent '{}' skipped because feature is disabled",
                            record.definition.name
                        ),
                    }),
                    SubagentStatus::Active => {}
                }

                if Some(idx) == chosen_idx {
                    continue;
                }

                let reason = if record.is_invalid() {
                    "invalid definition".to_string()
                } else if matches!(record.status, SubagentStatus::Disabled) {
                    "disabled subagent".to_string()
                } else if let Some(chosen_scope) = chosen_scope {
                    if chosen_scope != record.definition.scope {
                        "project override".to_string()
                    } else {
                        "duplicate definition".to_string()
                    }
                } else {
                    "no active definition available".to_string()
                };

                inventory.conflicts.push(SubagentConflict {
                    name: record.definition.name.clone(),
                    losing_scope: record.definition.scope,
                    reason,
                });
            }
        }

        inventory
    }
}

fn scope_precedence(scope: SubagentScope) -> u8 {
    match scope {
        SubagentScope::Project => 2,
        SubagentScope::User => 1,
    }
}
