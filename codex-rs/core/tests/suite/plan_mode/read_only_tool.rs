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

#[test]
fn read_only_tools_remain_available() {
    let tools = vec![
        ToolCapability::new("fs.read", ToolMode::ReadOnly),
        ToolCapability::new("fs.write", ToolMode::Write),
        ToolCapability::new("shell", ToolMode::Execute),
    ];
    let config = PlanModeConfig::default();

    let session = PlanModeSession::new(
        Uuid::new_v4(),
        AskForApproval::OnRequest,
        tools,
        &config,
        true,
    );

    let collected: Vec<&str> = session.allowed_tool_ids().collect();
    let mut expected = DEFAULT_SHELL_ENTRIES.to_vec();
    expected.push("fs.read");
    assert_eq!(
        collected, expected,
        "default shell helpers and read-only tools should remain"
    );
}

#[test]
fn wildcard_tool_rules_match_expected_tools() {
    let config = PlanModeConfig {
        allowed_read_only_tools: vec!["n8n-mcp__list_*".to_string()],
        ..Default::default()
    };

    let session = PlanModeSession::new(
        Uuid::new_v4(),
        AskForApproval::OnRequest,
        Vec::new(),
        &config,
        true,
    );

    assert!(session.is_tool_allowed("n8n-mcp__list_nodes"));
    assert!(!session.is_tool_allowed("n8n-mcp__get_workflow"));

    let display: Vec<&str> = session.allowed_tool_ids().collect();
    let mut expected = DEFAULT_SHELL_ENTRIES.to_vec();
    expected.push("n8n-mcp__list_*");
    assert_eq!(display, expected);
}
