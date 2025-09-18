use codex_core::plan_mode::PlanModeConfig;
use codex_core::plan_mode::PlanModeSession;
use codex_core::plan_mode::PlanModeState;
use codex_core::plan_mode::PlanTelemetry;
use codex_core::plan_mode::ToolCapability;
use codex_core::plan_mode::ToolMode;
use codex_core::protocol::AskForApproval;
use pretty_assertions::assert_eq;
use uuid::Uuid;

fn sample_tools() -> Vec<ToolCapability> {
    vec![
        ToolCapability::new("fs.read", ToolMode::ReadOnly),
        ToolCapability::new("shell", ToolMode::Execute),
    ]
}

#[test]
fn activation_flow_enters_plan_state() {
    let config = PlanModeConfig {
        plan_enabled: true,
        allowed_read_only_tools: vec!["fs.read".to_string()],
        planning_model: None,
        apply_requires_confirmation: true,
    };
    let session_id = Uuid::new_v4();
    let session = PlanModeSession::new(
        session_id,
        AskForApproval::OnRequest,
        sample_tools(),
        &config,
    );

    assert!(
        session.is_active(),
        "session should be active after activation"
    );
    assert_eq!(session.state, PlanModeState::Active);
    assert_eq!(session.entered_from, AskForApproval::OnRequest);

    let telemetry: PlanTelemetry = session.entered_telemetry();
    assert_eq!(
        telemetry.event,
        codex_core::plan_mode::PlanModeEvent::Entered
    );
    assert_eq!(telemetry.plan_entry_count, 0);
}
