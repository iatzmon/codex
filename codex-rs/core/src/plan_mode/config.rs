use serde::Deserialize;
use serde::Serialize;

use super::PlanModeAllowList;

/// User-configurable overrides that control Plan Mode behaviour.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanModeConfig {
    #[serde(default)]
    pub plan_enabled: bool,
    #[serde(default)]
    pub allowed_read_only_tools: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub planning_model: Option<String>,
    #[serde(default)]
    pub apply_requires_confirmation: bool,
}

impl Default for PlanModeConfig {
    fn default() -> Self {
        Self {
            plan_enabled: false,
            allowed_read_only_tools: Vec::new(),
            planning_model: None,
            apply_requires_confirmation: true,
        }
    }
}

impl PlanModeConfig {
    pub fn is_enabled(&self) -> bool {
        self.plan_enabled
    }

    pub fn allow_list(&self) -> PlanModeAllowList {
        PlanModeAllowList::new(&self.allowed_read_only_tools)
    }
}
