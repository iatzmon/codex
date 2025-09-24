use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvocationSession {
    pub parent_session_id: Option<String>,
    pub subagent_name: String,
    pub requested_tools: Vec<String>,
    pub execution_log: Vec<String>,
    pub summary: Option<String>,
    pub detail_artifacts: Vec<PathBuf>,
    pub confirmed: bool,
    pub resolved_model: Option<String>,
    pub extra_instructions: Option<String>,
}

impl InvocationSession {
    pub fn new(subagent_name: impl Into<String>) -> Self {
        Self {
            parent_session_id: None,
            subagent_name: subagent_name.into(),
            requested_tools: Vec::new(),
            execution_log: Vec::new(),
            summary: None,
            detail_artifacts: Vec::new(),
            confirmed: false,
            resolved_model: None,
            extra_instructions: None,
        }
    }

    pub fn confirmed(mut self) -> Self {
        self.confirmed = true;
        self
    }
}
