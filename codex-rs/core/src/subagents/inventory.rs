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

            let mut defs_iter = defs.into_iter();
            let chosen = defs_iter
                .next()
                .expect("grouped collection should always have at least one item");
            let record = SubagentRecord::from_definition(chosen.clone(), config);

            if record.is_invalid() {
                inventory.invalid_records.push(record);
            } else if matches!(record.status, SubagentStatus::Disabled) {
                inventory.discovery_events.push(DiscoveryEvent {
                    message: format!(
                        "subagent '{}' skipped because feature is disabled",
                        chosen.name
                    ),
                });
            } else {
                inventory.subagents.insert(name.clone(), record);
            }

            for losing in defs_iter {
                let reason = if chosen.scope != losing.scope {
                    "project override".to_string()
                } else {
                    "duplicate definition".to_string()
                };

                inventory.conflicts.push(SubagentConflict {
                    name: losing.name.clone(),
                    losing_scope: losing.scope,
                    reason,
                });

                let losing_record = SubagentRecord::from_definition(losing, config);
                if losing_record.is_invalid() {
                    inventory.invalid_records.push(losing_record);
                }
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
