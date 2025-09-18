use codex_core::plan_mode::PlanModeConfig;
use codex_core::plan_mode::PlanModeSession;
use codex_core::plan_mode::PlanModeState;
use codex_core::plan_mode::ToolCapability;
use codex_core::plan_mode::ToolMode;
use codex_core::protocol::AskForApproval;
use pretty_assertions::assert_eq;
use uuid::Uuid;

#[test]
fn apply_plan_injects_artifact_and_overrides_mode() {
    let mut session = PlanModeSession::new(
        Uuid::new_v4(),
        AskForApproval::OnRequest,
        vec![ToolCapability::new("fs.read", ToolMode::ReadOnly)],
        &PlanModeConfig::default(),
    );

    session.begin_apply(Some(AskForApproval::OnFailure));
    assert_eq!(session.state, PlanModeState::Applying);
    assert_eq!(session.pending_exit, Some(AskForApproval::OnFailure));
}
