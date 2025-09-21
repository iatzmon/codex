use codex_core::hooks::schema_registry::{HookPayloadValidationError, HookSchemaRegistry};
use pretty_assertions::assert_eq;

fn valid_payload() -> &'static str {
    r#"
    {
        "schemaVersion": "1.0",
        "event": "PreToolUse",
        "sessionId": "00000000-0000-0000-0000-000000000000",
        "timestamp": "2025-09-20T20:15:00Z",
        "workspaceRoot": "/workspace/project",
        "cwd": "/workspace/project",
        "eventContext": {
            "toolName": "shell",
            "arguments": {}
        }
    }
    "#
}

fn invalid_payload_missing_event() -> &'static str {
    r#"
    {
        "schemaVersion": "1.0",
        "sessionId": "00000000-0000-0000-0000-000000000000",
        "timestamp": "2025-09-20T20:15:00Z",
        "workspaceRoot": "/workspace/project",
        "cwd": "/workspace/project",
        "eventContext": {
            "toolName": "shell",
            "arguments": {}
        }
    }
    "#
}

#[test]
fn hook_payload_allows_valid_document() {
    let result = HookSchemaRegistry::validate_payload(valid_payload());
    assert_eq!(result, Ok(()));
}

#[test]
fn hook_payload_rejects_missing_event() {
    let result = HookSchemaRegistry::validate_payload(invalid_payload_missing_event());
    assert!(matches!(
        result,
        Err(HookPayloadValidationError::Invalid(message)) if message.contains("event")
    ));
}
