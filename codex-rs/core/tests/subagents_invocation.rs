use std::path::PathBuf;

use codex_core::subagents::SubagentBuilder;
use codex_core::subagents::config::{SubagentConfig, SubagentDiscoveryMode};
use codex_core::subagents::definition::{SubagentDefinition, SubagentScope};
use codex_core::subagents::invocation::InvocationSession;
use codex_core::subagents::runner::{SubagentInvocationError, SubagentRunner};

fn definition(scope: SubagentScope) -> SubagentDefinition {
    SubagentDefinition {
        raw_name: "code-reviewer".into(),
        name: "code-reviewer".into(),
        description: "Reviews staged diffs".into(),
        tools: vec!["git_diff".into()],
        model: Some("gpt-4.1-mini".into()),
        instructions: "Review the staged diffs for regressions.".into(),
        scope,
        source_path: PathBuf::from("/workspace/.codex/agents/code-reviewer.md"),
        validation_errors: Vec::new(),
    }
}

#[test]
fn denies_disallowed_tool() {
    let config = SubagentConfig::enabled(SubagentDiscoveryMode::Auto);
    let inventory = SubagentBuilder::new(config.clone())
        .with_definition(definition(SubagentScope::Project))
        .build();

    let mut session = InvocationSession::new("code-reviewer").confirmed();
    session.requested_tools = vec!["filesystem".into()];

    let err = SubagentRunner::new(&config, &inventory)
        .invoke(session)
        .expect_err("tool outside allowlist should be rejected");

    assert!(matches!(
        err,
        SubagentInvocationError::ToolNotAllowed { tool, .. } if tool == "filesystem"
    ));
}

#[test]
fn defers_detail_artifact_population_to_executor() {
    let config = SubagentConfig::enabled(SubagentDiscoveryMode::Auto);
    let inventory = SubagentBuilder::new(config.clone())
        .with_definition(definition(SubagentScope::Project))
        .build();

    let session = InvocationSession::new("code-reviewer").confirmed();
    let prepared = SubagentRunner::new(&config, &inventory)
        .invoke(session)
        .expect("invocation should succeed with default detail artifact");

    assert!(
        prepared.session.summary.is_none(),
        "runner should defer summary population to the executor"
    );
    assert!(
        prepared.session.detail_artifacts.is_empty(),
        "detail artifacts should be populated after execution"
    );
}
