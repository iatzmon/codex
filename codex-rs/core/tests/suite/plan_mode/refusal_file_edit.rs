use codex_core::plan_mode::PlanEntryType;
use codex_core::plan_mode::PlanModeConfig;
use codex_core::plan_mode::PlanModeSession;
use codex_core::plan_mode::ToolCapability;
use codex_core::plan_mode::ToolMode;
use codex_core::protocol::AskForApproval;
use pretty_assertions::assert_eq;
use uuid::Uuid;

fn session() -> PlanModeSession {
    let config = PlanModeConfig::default();
    PlanModeSession::new(
        Uuid::new_v4(),
        AskForApproval::OnRequest,
        vec![ToolCapability::new("fs.read", ToolMode::ReadOnly)],
        &config,
    )
}

#[test]
fn file_edit_requests_are_captured_as_plan_entries() {
    let mut session = session();
    let telemetry = session.record_refusal(
        PlanEntryType::FileChange,
        "edit src/lib.rs",
        Some("diff --git".to_string()),
    );

    assert_eq!(
        telemetry.event,
        codex_core::plan_mode::PlanModeEvent::RefusalCaptured
    );
    let artifact = session.plan_artifact();
    assert_eq!(artifact.entry_count(), 1);
    let entry = artifact.steps.first().expect("entry should exist");
    assert_eq!(entry.sequence, 1);
    assert_eq!(entry.entry_type, PlanEntryType::FileChange);
    assert_eq!(entry.summary, "edit src/lib.rs");
    assert_eq!(entry.details.as_deref(), Some("diff --git"));
}
