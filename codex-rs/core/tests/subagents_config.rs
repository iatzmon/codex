use std::path::PathBuf;

use codex_core::subagents::config::{SubagentConfig, SubagentDiscoveryMode};
use codex_core::subagents::definition::{SubagentDefinition, SubagentScope};
use codex_core::subagents::invocation::InvocationSession;
use codex_core::subagents::runner::{SubagentInvocationError, SubagentRunner};
use codex_core::subagents::{SubagentBuilder, SubagentInventory};
use pretty_assertions::assert_eq;

fn definition(scope: SubagentScope) -> SubagentDefinition {
    SubagentDefinition {
        raw_name: "code-reviewer".into(),
        name: "code-reviewer".into(),
        description: "Reviews staged diffs".into(),
        tools: vec!["git_diff".into()],
        model: None,
        instructions: "Review the staged diffs for regressions.".into(),
        scope,
        source_path: PathBuf::from("/workspace/.codex/agents/code-reviewer.md"),
        validation_errors: Vec::new(),
    }
}

#[test]
fn inventory_respects_disabled_flag() {
    let config = SubagentConfig::disabled();
    let inventory =
        SubagentInventory::from_definitions(&config, [definition(SubagentScope::Project)]);

    assert!(
        inventory.subagents.is_empty(),
        "disabled config should skip discovery"
    );
    assert!(
        inventory
            .discovery_events
            .iter()
            .any(|event| event.message.contains("disabled")),
        "disabled config should emit a discovery hint"
    );
}

#[test]
fn runner_uses_default_model_when_definition_missing() {
    let config = SubagentConfig::enabled(SubagentDiscoveryMode::Auto)
        .with_default_model(Some("gpt-4.1-mini".into()));
    let inventory = SubagentBuilder::new(config.clone())
        .with_definition(definition(SubagentScope::Project))
        .build();

    let session = InvocationSession::new("code-reviewer").confirmed();
    let prepared = SubagentRunner::new(&config, &inventory)
        .invoke(session)
        .expect("invocation should succeed when enabled");

    assert_eq!(
        prepared.session.resolved_model.as_deref(),
        Some("gpt-4.1-mini"),
        "runner should fall back to default model"
    );
}

#[test]
fn runner_fails_when_feature_disabled() {
    let config = SubagentConfig::disabled();
    let inventory = SubagentBuilder::new(config.clone())
        .with_definition(definition(SubagentScope::Project))
        .build();

    let session = InvocationSession::new("code-reviewer").confirmed();
    let err = SubagentRunner::new(&config, &inventory)
        .invoke(session)
        .expect_err("disabled config should reject invocations");

    assert!(matches!(err, SubagentInvocationError::FeatureDisabled));
}
