use codex_core::plan_mode::PlanModeConfig;
use codex_core::plan_mode::PlanModeSession;
use codex_core::plan_mode::PlanModeState;
use codex_core::plan_mode::ToolCapability;
use codex_core::plan_mode::ToolMode;
use codex_core::protocol::AskForApproval;
use pretty_assertions::assert_eq;
use uuid::Uuid;

#[test]
fn exit_plan_restores_prior_mode() {
    let mut session = PlanModeSession::new(
        Uuid::new_v4(),
        AskForApproval::OnFailure,
        vec![ToolCapability::new("fs.read", ToolMode::ReadOnly)],
        &PlanModeConfig::default(),
    );

    session.exit_plan_mode();
    assert_eq!(session.state, PlanModeState::Exited);
    assert!(session.pending_exit.is_none());
}
