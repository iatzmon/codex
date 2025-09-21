use std::path::PathBuf;

use codex_core::hooks::{
    HookDecision, HookEvent, HookExecutionRecord, HookLogWriter, HookOutcome, HookScope,
};
use serde_json::Value;
use tempfile::tempdir;

#[tokio::test]
async fn writes_jsonl_record_with_newline() {
    let temp = tempdir().unwrap();
    let log_path = temp.path().join("hooks.jsonl");
    let writer = HookLogWriter::new(log_path.clone());

    let mut record = HookExecutionRecord::new(
        HookEvent::Notification,
        HookScope::LocalUser {
            codex_home: PathBuf::from("/home/user/.codex"),
        },
        "local.notify",
    );

    record.decision = HookDecision {
        decision: HookOutcome::Allow,
        message: Some("ok".to_string()),
        system_message: None,
        stop_reason: None,
        extra: Value::Null,
        exit_code: 0,
    };

    writer.append(&record).await.unwrap();

    let contents = tokio::fs::read_to_string(&log_path).await.unwrap();
    assert!(contents.ends_with('\n'));
    let line = contents.trim_end();
    let parsed: HookExecutionRecord = serde_json::from_str(line).unwrap();
    assert_eq!(parsed.hook_id, "local.notify");
    assert_eq!(parsed.decision.message.as_deref(), Some("ok"));
}
