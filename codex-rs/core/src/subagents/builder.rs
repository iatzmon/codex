use crate::subagents::config::SubagentConfig;
use crate::subagents::definition::SubagentDefinition;
use crate::subagents::discovery::{
    DiscoveryOutcome, DiscoverySource, SubagentSourceTree, discover_from_source,
};
use crate::subagents::inventory::{DiscoveryEvent, SubagentInventory};

pub struct SubagentBuilder {
    config: SubagentConfig,
    definitions: Vec<SubagentDefinition>,
    events: Vec<DiscoveryEvent>,
}

impl SubagentBuilder {
    pub fn new(config: SubagentConfig) -> Self {
        Self {
            config,
            definitions: Vec::new(),
            events: Vec::new(),
        }
    }

    pub fn with_definition(mut self, definition: SubagentDefinition) -> Self {
        self.definitions.push(definition);
        self
    }

    pub fn with_definitions<I>(mut self, definitions: I) -> Self
    where
        I: IntoIterator<Item = SubagentDefinition>,
    {
        self.definitions.extend(definitions);
        self
    }

    pub fn with_event(mut self, event: DiscoveryEvent) -> Self {
        self.events.push(event);
        self
    }

    pub fn record_event(mut self, message: impl Into<String>) -> Self {
        self.events.push(DiscoveryEvent {
            message: message.into(),
        });
        self
    }

    pub fn discover_source(mut self, source: DiscoverySource) -> Self {
        let DiscoveryOutcome {
            definitions,
            events,
        } = discover_from_source(source);
        self.definitions.extend(definitions);
        self.events.extend(events);
        self
    }

    pub fn discover_tree(mut self, tree: &SubagentSourceTree) -> Self {
        for path in &tree.project {
            self = self.discover_source(DiscoverySource::Project(path.clone()));
        }
        for path in &tree.user {
            self = self.discover_source(DiscoverySource::User(path.clone()));
        }
        self
    }

    pub fn build(self) -> SubagentInventory {
        let mut inventory = SubagentInventory::from_definitions(&self.config, self.definitions);
        inventory.discovery_events.extend(self.events);
        inventory
    }
}
