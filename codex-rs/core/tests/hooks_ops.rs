use std::sync::Arc;

use codex_core::protocol::{EventMsg, Op};
use codex_core::{CodexAuth, CodexConversation, ConversationManager};
use codex_protocol::hooks::{
    HookExecLogRequest, HookListRequest, HookScopeFilter, HookValidateRequest, HookValidationStatus,
};
use core_test_support::{load_default_config_for_test, wait_for_event};
use tempfile::TempDir;

async fn setup_conversation() -> (TempDir, Arc<CodexConversation>, ConversationManager) {
    let codex_home = TempDir::new().expect("create temp dir");
    let config = load_default_config_for_test(&codex_home);
    let manager = ConversationManager::with_auth(CodexAuth::from_api_key("test"));
    let conversation = manager
        .new_conversation(config)
        .await
        .expect("create conversation")
        .conversation;
    (codex_home, conversation, manager)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn hook_list_returns_empty_snapshot() {
    let (_home, codex, _manager) = setup_conversation().await;

    codex
        .submit(Op::HookList(HookListRequest {
            event: None,
            scope: None,
        }))
        .await
        .expect("submit hook list");

    let event = wait_for_event(&codex, |msg| matches!(msg, EventMsg::HookListResponse(_))).await;
    if let EventMsg::HookListResponse(payload) = event {
        assert!(payload.registry.events.is_empty());
    } else {
        panic!("unexpected event variant");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn hook_ops_return_placeholder_responses() {
    let (_home, codex, _manager) = setup_conversation().await;

    codex
        .submit(Op::HookExecLog(HookExecLogRequest {
            since: None,
            event: None,
            hook_id: None,
            tail: Some(5),
        }))
        .await
        .expect("submit hook exec log");
    let event = wait_for_event(&codex, |msg| {
        matches!(msg, EventMsg::HookExecLogResponse(_))
    })
    .await;
    if let EventMsg::HookExecLogResponse(payload) = event {
        assert!(payload.logs.records.is_empty());
    } else {
        panic!("unexpected event variant");
    }

    codex
        .submit(Op::HookValidate(HookValidateRequest {
            scope: Some(HookScopeFilter::Project),
        }))
        .await
        .expect("submit hook validate");
    let event = wait_for_event(&codex, |msg| {
        matches!(msg, EventMsg::HookValidationResult(_))
    })
    .await;
    if let EventMsg::HookValidationResult(payload) = event {
        assert_eq!(payload.summary.status, HookValidationStatus::Ok);
    } else {
        panic!("unexpected event variant");
    }

    codex
        .submit(Op::HookReload)
        .await
        .expect("submit hook reload");
    let event = wait_for_event(&codex, |msg| matches!(msg, EventMsg::HookReloadResult(_))).await;
    if let EventMsg::HookReloadResult(payload) = event {
        assert!(!payload.result.reloaded);
    } else {
        panic!("unexpected event variant");
    }
}
