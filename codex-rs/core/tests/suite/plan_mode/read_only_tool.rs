use codex_core::plan_mode::PlanModeConfig;
use codex_core::plan_mode::PlanModeSession;
use codex_core::plan_mode::ToolCapability;
use codex_core::plan_mode::ToolMode;
use codex_core::protocol::AskForApproval;
use pretty_assertions::assert_eq;
use uuid::Uuid;

#[test]
fn read_only_tools_remain_available() {
    let tools = vec![
        ToolCapability::new("fs.read", ToolMode::ReadOnly),
        ToolCapability::new("fs.write", ToolMode::Write),
        ToolCapability::new("shell", ToolMode::Execute),
    ];
    let config = PlanModeConfig::default();

    let session = PlanModeSession::new(Uuid::new_v4(), AskForApproval::OnRequest, tools, &config);

    let collected: Vec<&str> = session.allowed_tool_ids().collect();
    assert_eq!(
        collected,
        vec!["fs.read"],
        "only read-only tools should remain"
    );
}
