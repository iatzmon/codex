use serde::Deserialize;
use serde::Serialize;

/// Describes whether a tool is read-only or capable of mutating state. Plan
/// Mode uses this information to expose only safe capabilities during the
/// planning phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolMode {
    ReadOnly,
    Write,
    Execute,
}

impl ToolMode {
    /// Convenience helper to determine if the capability should be considered
    /// safe while operating in Plan Mode.
    pub fn is_read_only(self) -> bool {
        matches!(self, ToolMode::ReadOnly)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolCapability {
    pub id: String,
    pub mode: ToolMode,
    pub requires_network: bool,
}

impl ToolCapability {
    pub fn new(id: impl Into<String>, mode: ToolMode) -> Self {
        Self {
            id: id.into(),
            mode,
            requires_network: false,
        }
    }

    pub fn with_network_requirement(mut self, requires_network: bool) -> Self {
        self.requires_network = requires_network;
        self
    }

    /// Helper to determine whether the capability can be surfaced in Plan Mode
    /// given the current network policy enforcement.
    #[allow(dead_code)]
    pub fn is_allowed_in_plan_mode(&self, network_enabled: bool) -> bool {
        self.mode.is_read_only() && (!self.requires_network || network_enabled)
    }
}
