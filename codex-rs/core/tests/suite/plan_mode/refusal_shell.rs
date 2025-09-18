use codex_core::plan_mode::PlanEntryType;
use codex_core::plan_mode::PlanModeConfig;
use codex_core::plan_mode::PlanModeSession;
use codex_core::plan_mode::ToolCapability;
use codex_core::plan_mode::ToolMode;
use codex_core::protocol::AskForApproval;
use pretty_assertions::assert_eq;
use uuid::Uuid;

fn session() -> PlanModeSession {
    PlanModeSession::new(
        Uuid::new_v4(),
        AskForApproval::OnRequest,
        vec![ToolCapability::new("fs.read", ToolMode::ReadOnly)],
        &PlanModeConfig::default(),
    )
}

#[test]
fn shell_commands_are_blocked_and_recorded() {
    let mut session = session();
    let telemetry = session.record_refusal(PlanEntryType::Command, "rm -rf /tmp", None);

    assert_eq!(
        telemetry.event,
        codex_core::plan_mode::PlanModeEvent::RefusalCaptured
    );
    let artifact = session.plan_artifact();
    assert_eq!(artifact.entry_count(), 1);
    let entry = artifact.steps.first().expect("entry should exist");
    assert_eq!(entry.entry_type, PlanEntryType::Command);
    assert_eq!(entry.summary, "rm -rf /tmp");
}
