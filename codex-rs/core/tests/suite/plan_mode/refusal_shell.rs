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
        true,
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

#[test]
fn shell_commands_can_be_allowed_via_pattern() {
    let config = PlanModeConfig {
        allowed_read_only_tools: vec!["shell(cat *)".to_string()],
        ..Default::default()
    };

    let session = PlanModeSession::new(
        Uuid::new_v4(),
        AskForApproval::OnRequest,
        Vec::new(),
        &config,
        true,
    );

    assert!(session.is_shell_allowed("cat README.md"));
    assert!(!session.is_shell_allowed("npm run test:unit"));
}

#[test]
fn default_shell_commands_are_allowed() {
    let session = session();

    for (command, label) in [
        ("bash -lc cat README.md", "cat"),
        ("bash -lc find .", "find"),
        ("bash -lc grep pattern", "grep"),
        ("bash -lc ls -a", "ls"),
        ("bash -lc tree .", "tree"),
        ("bash -lc head README.md", "head"),
        ("bash -lc tail README.md", "tail"),
        ("bash -lc stat README.md", "stat"),
        ("bash -lc pwd", "pwd"),
        ("bash -lc git status", "git status"),
        ("bash -lc git diff --stat", "git diff --stat"),
    ] {
        assert!(
            session.is_shell_allowed(command),
            "expected default {label} command to be allowed"
        );
    }
}

#[test]
fn default_shell_commands_are_preserved_with_custom_allowlist() {
    let config = PlanModeConfig {
        allowed_read_only_tools: vec!["shell(bash -lc echo *)".to_string()],
        ..Default::default()
    };

    let session = PlanModeSession::new(
        Uuid::new_v4(),
        AskForApproval::OnRequest,
        vec![ToolCapability::new("fs.read", ToolMode::ReadOnly)],
        &config,
        true,
    );

    assert!(session.is_shell_allowed("bash -lc cat README.md"));
    assert!(session.is_shell_allowed("bash -lc git status"));
    assert!(session.is_shell_allowed("bash -lc git diff --stat"));
    assert!(session.is_shell_allowed("bash -lc pwd"));
    assert!(session.is_shell_allowed("bash -lc echo hello"));
}
