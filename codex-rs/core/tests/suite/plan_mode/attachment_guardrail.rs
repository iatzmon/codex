use codex_core::plan_mode::PlanModeConfig;
use codex_core::plan_mode::PlanModeSession;
use codex_core::plan_mode::ToolCapability;
use codex_core::plan_mode::ToolMode;
use codex_core::protocol::AskForApproval;
use pretty_assertions::assert_eq;
use uuid::Uuid;

const DEFAULT_SHELL_ENTRIES: &[&str] = &[
    "shell(bash -lc cat *)",
    "shell(bash -lc find *)",
    "shell(bash -lc grep *)",
    "shell(bash -lc ls *)",
    "shell(bash -lc tree *)",
    "shell(bash -lc head *)",
    "shell(bash -lc tail *)",
    "shell(bash -lc stat *)",
    "shell(bash -lc pwd *)",
    "shell(bash -lc pwd)",
    "shell(bash -lc git status)",
    "shell(bash -lc git diff --stat)",
];

fn session_with_config(config: PlanModeConfig) -> PlanModeSession {
    let tools = vec![
        ToolCapability::new("attachments.read", ToolMode::ReadOnly).with_network_requirement(true),
    ];
    PlanModeSession::new(
        Uuid::new_v4(),
        AskForApproval::OnRequest,
        tools,
        &config,
        true,
    )
}

#[test]
fn external_attachments_are_blocked_with_guidance() {
    let restricted = session_with_config(PlanModeConfig::default());
    let tools: Vec<&str> = restricted.allowed_tool_ids().collect();
    assert_eq!(tools, DEFAULT_SHELL_ENTRIES);

    let config = PlanModeConfig {
        plan_enabled: true,
        allowed_read_only_tools: vec!["attachments.read".to_string()],
        planning_model: None,
        apply_requires_confirmation: true,
    };
    let allowed = session_with_config(config);
    let tools: Vec<&str> = allowed.allowed_tool_ids().collect();
    let mut expected = DEFAULT_SHELL_ENTRIES.to_vec();
    expected.push("attachments.read");
    assert_eq!(tools, expected);
}
