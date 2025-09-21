use std::path::PathBuf;

use codex_core::hooks::executor::{HookExecutor, PreToolUsePayload};
use codex_core::hooks::{
    HookEvent, HookExecutionRecord, HookLogWriter, HookOutcome, HookRegistry, HookScope,
};
use tempfile::tempdir;

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
async fn logs_pretool_decision_to_jsonl() {
    let temp = tempdir().unwrap();
    let log_path = temp.path().join("hooks.jsonl");
    let executor = HookExecutor::with_runtime(
        HookRegistry::default(),
        HookLogWriter::new(log_path.clone()),
        HookScope::LocalUser {
            codex_home: PathBuf::new(),
        },
    );

    let payload = PreToolUsePayload {
        tool_name: "shell".to_string(),
        command: "echo hi".to_string(),
    };

    executor
        .evaluate_pre_tool_use(&payload)
        .await
        .expect("pre-tool hook should evaluate");

    let contents = tokio::fs::read_to_string(&log_path)
        .await
        .expect("log file should exist");
    let line = contents
        .lines()
        .next()
        .expect("log file should contain a record");
    let record: HookExecutionRecord = serde_json::from_str(line).expect("valid log record");
    assert_eq!(record.event, HookEvent::PreToolUse);
    assert_eq!(record.decision.exit_code, 0);
    assert_eq!(record.trigger_id, payload.command);
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
