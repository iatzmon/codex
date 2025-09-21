use codex_core::hooks::HookOutcome;
use codex_core::hooks::executor::{HookExecutor, PreToolUsePayload};

#[tokio::test]
async fn pretool_guard_blocks_dangerous_shell_command() {
    let executor = HookExecutor::default();
    let payload = PreToolUsePayload {
        tool_name: "shell".to_string(),
        command: "rm -rf /var/www".to_string(),
    };

    let decision = executor
        .evaluate_pre_tool_use(&payload)
        .await
        .expect("pre-tool guard should produce a decision");

    assert_eq!(decision.decision, HookOutcome::Deny);
    assert_eq!(decision.exit_code, 2);
}

#[tokio::test]
async fn pretool_guard_allows_safe_command() {
    let executor = HookExecutor::default();
    let payload = PreToolUsePayload {
        tool_name: "shell".to_string(),
        command: "echo hello".to_string(),
    };

    let decision = executor
        .evaluate_pre_tool_use(&payload)
        .await
        .expect("pre-tool guard should produce a decision");

    assert_eq!(decision.decision, HookOutcome::Allow);
    assert_eq!(decision.exit_code, 0);
}
