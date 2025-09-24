#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubagentDiscoveryMode {
    Auto,
    Manual,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubagentConfig {
    pub enabled: bool,
    pub default_model: Option<String>,
    pub discovery: SubagentDiscoveryMode,
}

impl Default for SubagentConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            default_model: None,
            discovery: SubagentDiscoveryMode::Auto,
        }
    }
}

impl SubagentConfig {
    pub fn new(
        enabled: bool,
        default_model: Option<String>,
        discovery: SubagentDiscoveryMode,
    ) -> Self {
        Self {
            enabled,
            default_model,
            discovery,
        }
    }

    pub fn enabled(discovery: SubagentDiscoveryMode) -> Self {
        Self::new(true, None, discovery)
    }

    pub fn disabled() -> Self {
        Self::default()
    }

    pub fn with_default_model(mut self, default_model: Option<String>) -> Self {
        self.default_model = default_model;
        self
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}
