use crate::subagents::config::{SubagentConfig, SubagentDiscoveryMode};
use crate::subagents::inventory::SubagentInventory;
use crate::subagents::invocation::InvocationSession;
use crate::subagents::record::SubagentRecord;
use crate::subagents::record::SubagentStatus;

#[derive(Debug, thiserror::Error)]
pub enum SubagentInvocationError {
    #[error("subagents feature disabled")]
    FeatureDisabled,
    #[error("no subagent named '{0}'")]
    UnknownSubagent(String),
    #[error("subagent '{0}' is invalid")]
    InvalidSubagent(String),
    #[error("subagent '{0}' is disabled")]
    DisabledSubagent(String),
    #[error("tool '{tool}' is not allowed for subagent '{subagent}'")]
    ToolNotAllowed { subagent: String, tool: String },
    #[error("confirmation required before invoking subagent '{0}'")]
    ConfirmationRequired(String),
    #[error("subagent execution failed: {0}")]
    ExecutionFailed(String),
    #[error("subagent invocation requires authentication manager")]
    MissingAuthManager,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedSubagentInvocation {
    pub session: InvocationSession,
    pub record: SubagentRecord,
}

#[derive(Debug)]
pub struct SubagentRunner<'a> {
    pub config: &'a SubagentConfig,
    pub inventory: &'a SubagentInventory,
}

impl<'a> SubagentRunner<'a> {
    pub fn new(config: &'a SubagentConfig, inventory: &'a SubagentInventory) -> Self {
        Self { config, inventory }
    }

    pub fn invoke(
        &self,
        mut session: InvocationSession,
    ) -> Result<PreparedSubagentInvocation, SubagentInvocationError> {
        if !self.config.is_enabled() {
            return Err(SubagentInvocationError::FeatureDisabled);
        }

        let record = self
            .inventory
            .subagents
            .get(&session.subagent_name)
            .or_else(|| {
                self.inventory
                    .invalid()
                    .into_iter()
                    .find(|record| record.definition.name == session.subagent_name)
                    .map(|record| record)
            })
            .ok_or_else(|| {
                SubagentInvocationError::UnknownSubagent(session.subagent_name.clone())
            })?;

        match record.status {
            SubagentStatus::Invalid => {
                return Err(SubagentInvocationError::InvalidSubagent(
                    record.definition.name.clone(),
                ));
            }
            SubagentStatus::Disabled => {
                return Err(SubagentInvocationError::DisabledSubagent(
                    record.definition.name.clone(),
                ));
            }
            SubagentStatus::Active => {}
        }

        if matches!(self.config.discovery, SubagentDiscoveryMode::Auto) && !session.confirmed {
            return Err(SubagentInvocationError::ConfirmationRequired(
                record.definition.name.clone(),
            ));
        }

        for tool in &session.requested_tools {
            if !record.allows_tool(tool) {
                return Err(SubagentInvocationError::ToolNotAllowed {
                    subagent: record.definition.name.clone(),
                    tool: tool.clone(),
                });
            }
        }

        if session.resolved_model.is_none() {
            session.resolved_model = record
                .effective_model
                .clone()
                .or_else(|| self.config.default_model.clone());
        }

        if session.summary.is_none() {
            session.summary = None;
        }

        if session.detail_artifacts.is_empty() {
            session.detail_artifacts.clear();
        }

        Ok(PreparedSubagentInvocation {
            session,
            record: record.clone(),
        })
    }
}
