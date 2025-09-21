use std::collections::HashMap;
use std::path::PathBuf;

use chrono::{TimeZone, Utc};
use codex_protocol::hooks::{
    HookDefinition, HookEvent, HookExecLogRequest, HookExecLogResponse, HookListRequest,
    HookMatcher, HookMatchers, HookRegistrySnapshot, HookReloadResponse, HookScope,
    HookValidationStatus, HookValidationSummary,
};
use codex_protocol::protocol::{
    EventMsg, HookExecLogResponseEvent, HookRegistrySnapshotEvent, HookReloadResultEvent,
    HookValidationResultEvent, Op,
};
use serde_json::json;

#[test]
fn op_hook_list_serializes_with_filters() {
    let op = Op::HookList(HookListRequest {
        event: Some(HookEvent::PreToolUse),
        scope: None,
    });

    let value = serde_json::to_value(&op).unwrap();
    assert_eq!(value["type"], "hook_list");
    assert_eq!(value["event"], "PreToolUse");
}

#[test]
fn event_hook_list_response_serializes_snapshot() {
    let mut registry = HookRegistrySnapshot::default();
    registry.last_loaded = Some(Utc.with_ymd_and_hms(2025, 9, 20, 12, 0, 0).unwrap());
    registry.events.insert(
        HookEvent::PreToolUse,
        vec![HookDefinition {
            id: "managed.guard".to_string(),
            event: HookEvent::PreToolUse,
            notes: None,
            command: vec!["/usr/bin/check".to_string()],
            working_dir: Some(PathBuf::from("/etc/codex")),
            timeout_ms: Some(10_000),
            allow_parallel: false,
            schema_versions: vec!["1.0".to_string()],
            env: HashMap::new(),
            matchers: HookMatchers {
                tool_names: vec![HookMatcher::Glob {
                    value: "shell*".to_string(),
                }],
                ..HookMatchers::default()
            },
            scope: HookScope::ManagedPolicy {
                name: "default".to_string(),
            },
            source_path: Some(PathBuf::from("/etc/codex/hooks/policy.toml")),
        }],
    );

    let event = EventMsg::HookListResponse(HookRegistrySnapshotEvent { registry });
    let value = serde_json::to_value(event).unwrap();
    assert_eq!(value["type"], "hook_list_response");
    assert_eq!(value["registry"]["events"]["PreToolUse"].as_array().unwrap().len(), 1);
}

#[test]
fn event_hook_exec_log_response_serializes_records() {
    let response = HookExecLogResponseEvent {
        logs: HookExecLogResponse {
            records: vec![serde_json::from_value(json!({
                "id": "00000000-0000-0000-0000-000000000000",
                "timestamp": "2025-09-20T12:00:00Z",
                "event": "PreToolUse",
                "scope": {"type": "managedPolicy", "name": "default"},
                "hookId": "managed.guard",
                "decision": {
                    "decision": "Allow",
                    "exitCode": 0,
                    "extra": null
                },
                "durationMs": 42,
                "stdout": [],
                "stderr": [],
                "precedenceRank": 0,
                "payloadHash": "abc",
                "triggerId": "turn-1"
            })).unwrap()],
        },
    };

    let value = serde_json::to_value(EventMsg::HookExecLogResponse(response)).unwrap();
    assert_eq!(value["type"], "hook_exec_log_response");
    assert_eq!(value["logs"]["records"].as_array().unwrap().len(), 1);
}

#[test]
fn event_hook_validation_response_serializes_summary() {
    let summary = HookValidationSummary {
        status: HookValidationStatus::Warning,
        errors: vec![],
        warnings: vec!["managed policy missing schemaVersions".to_string()],
        layers: vec![],
    };
    let value = serde_json::to_value(EventMsg::HookValidationResult(
        HookValidationResultEvent { summary },
    ))
    .unwrap();
    assert_eq!(value["type"], "hook_validation_result");
    assert_eq!(value["summary"]["status"], "warning");
}

#[test]
fn event_hook_reload_response_serializes() {
    let event = EventMsg::HookReloadResult(HookReloadResultEvent {
        result: HookReloadResponse {
            reloaded: true,
            message: Some("Reloaded".to_string()),
        },
    });

    let value = serde_json::to_value(event).unwrap();
    assert_eq!(value["type"], "hook_reload_result");
    assert_eq!(value["result"]["reloaded"], true);
}

#[test]
fn op_hook_exec_log_request_serializes_filters() {
    let op = Op::HookExecLog(HookExecLogRequest {
        since: Some(Utc.with_ymd_and_hms(2025, 9, 20, 12, 0, 0).unwrap()),
        event: Some(HookEvent::Notification),
        hook_id: Some("managed.notify".to_string()),
        tail: Some(10),
    });

    let value = serde_json::to_value(op).unwrap();
    assert_eq!(value["type"], "hook_exec_log");
    assert_eq!(value["tail"], 10);
}
