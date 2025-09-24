use crate::subagents::config::SubagentConfig;
use crate::subagents::definition::SubagentDefinition;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SubagentStatus {
    Active,
    Invalid,
    Disabled,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubagentRecord {
    pub definition: SubagentDefinition,
    pub effective_tools: Vec<String>,
    pub effective_model: Option<String>,
    pub status: SubagentStatus,
    pub validation_errors: Vec<String>,
}

impl SubagentRecord {
    pub fn from_definition(definition: SubagentDefinition, config: &SubagentConfig) -> Self {
        let mut status = SubagentStatus::Active;
        if !definition.validation_errors.is_empty() {
            status = SubagentStatus::Invalid;
        } else if !config.enabled {
            status = SubagentStatus::Disabled;
        }

        let effective_model = if let Some(model) = &definition.model {
            Some(model.clone())
        } else {
            config.default_model.clone()
        };

        Self {
            effective_tools: definition.tools.clone(),
            validation_errors: definition.validation_errors.clone(),
            definition,
            effective_model,
            status,
        }
    }

    pub fn allows_tool(&self, tool: &str) -> bool {
        if self.effective_tools.is_empty() {
            return true;
        }
        self.effective_tools.iter().any(|allowed| allowed == tool)
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, SubagentStatus::Active)
    }

    pub fn is_invalid(&self) -> bool {
        matches!(self.status, SubagentStatus::Invalid)
    }
}
