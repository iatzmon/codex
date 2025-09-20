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

fn vec_cmd(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| s.to_string()).collect()
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
fn shell_allowlist_is_shell_agnostic() {
    let config = PlanModeConfig {
        allowed_read_only_tools: vec!["shell(git status)".to_string()],
        ..Default::default()
    };

    let session = PlanModeSession::new(
        Uuid::new_v4(),
        AskForApproval::OnRequest,
        Vec::new(),
        &config,
        true,
    );

    let bash = vec_cmd(&["/bin/bash", "-lc", "git status"]);
    assert!(session.is_shell_allowed(&bash));

    let zsh = vec_cmd(&["zsh", "-lc", "git status"]);
    assert!(session.is_shell_allowed(&zsh));

    let direct = vec_cmd(&["git", "status"]);
    assert!(session.is_shell_allowed(&direct));

    let disallowed = vec_cmd(&["git", "diff"]);
    assert!(!session.is_shell_allowed(&disallowed));
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

    let cat = vec_cmd(&["cat", "README.md"]);
    assert!(session.is_shell_allowed(&cat));
    let npm = vec_cmd(&["npm", "run", "test:unit"]);
    assert!(!session.is_shell_allowed(&npm));
}

#[test]
fn default_shell_commands_are_allowed() {
    let session = session();

    let cases: Vec<(Vec<String>, &str)> = vec![
        (vec_cmd(&["bash", "-lc", "cat README.md"]), "cat"),
        (vec_cmd(&["bash", "-lc", "find ."]), "find"),
        (vec_cmd(&["bash", "-lc", "grep pattern"]), "grep"),
        (vec_cmd(&["bash", "-lc", "ls -a"]), "ls"),
        (vec_cmd(&["bash", "-lc", "tree ."]), "tree"),
        (vec_cmd(&["bash", "-lc", "head README.md"]), "head"),
        (vec_cmd(&["bash", "-lc", "tail README.md"]), "tail"),
        (vec_cmd(&["bash", "-lc", "stat README.md"]), "stat"),
        (vec_cmd(&["bash", "-lc", "pwd"]), "pwd"),
        (vec_cmd(&["bash", "-lc", "git status"]), "git status"),
        (
            vec_cmd(&["bash", "-lc", "git diff --stat"]),
            "git diff --stat",
        ),
    ];

    for (command, label) in cases {
        assert!(
            session.is_shell_allowed(&command),
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

    let cat = vec_cmd(&["bash", "-lc", "cat README.md"]);
    assert!(session.is_shell_allowed(&cat));
    let status = vec_cmd(&["bash", "-lc", "git status"]);
    assert!(session.is_shell_allowed(&status));
    let diff = vec_cmd(&["bash", "-lc", "git diff --stat"]);
    assert!(session.is_shell_allowed(&diff));
    let pwd = vec_cmd(&["bash", "-lc", "pwd"]);
    assert!(session.is_shell_allowed(&pwd));
    let echo = vec_cmd(&["bash", "-lc", "echo hello"]);
    assert!(session.is_shell_allowed(&echo));
}
