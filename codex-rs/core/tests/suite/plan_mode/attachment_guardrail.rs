use codex_core::plan_mode::PlanModeConfig;
use codex_core::plan_mode::PlanModeSession;
use codex_core::plan_mode::ToolCapability;
use codex_core::plan_mode::ToolMode;
use codex_core::protocol::AskForApproval;
use pretty_assertions::assert_eq;
use uuid::Uuid;

fn session_with_config(config: PlanModeConfig) -> PlanModeSession {
    let tools = vec![
        ToolCapability::new("attachments.read", ToolMode::ReadOnly).with_network_requirement(true),
    ];
    PlanModeSession::new(Uuid::new_v4(), AskForApproval::OnRequest, tools, &config)
}

#[test]
fn external_attachments_are_blocked_with_guidance() {
    let restricted = session_with_config(PlanModeConfig::default());
    assert!(restricted.allowed_tool_ids().next().is_none());

    let config = PlanModeConfig {
        plan_enabled: true,
        allowed_read_only_tools: vec!["attachments.read".to_string()],
        planning_model: None,
        apply_requires_confirmation: true,
    };
    let allowed = session_with_config(config);
    let tools: Vec<&str> = allowed.allowed_tool_ids().collect();
    assert_eq!(tools, vec!["attachments.read"]);
}
