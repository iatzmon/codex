use std::path::PathBuf;

use codex_core::subagents::{
    builder::SubagentBuilder,
    config::{SubagentConfig, SubagentDiscoveryMode},
    definition::{SubagentDefinition, SubagentScope},
    invocation::InvocationSession,
    runner::{SubagentInvocationError, SubagentRunner},
};

fn restricted_definition() -> SubagentDefinition {
    SubagentDefinition {
        raw_name: "shell-guard".into(),
        name: "shell-guard".into(),
        description: "Runs shell commands with a limited allowlist".into(),
        tools: vec!["git_diff".into()],
        model: None,
        instructions: "Review staged diffs without running arbitrary shell commands.".into(),
        scope: SubagentScope::Project,
        source_path: PathBuf::from("/home/iatzmon/workspace/codex/.codex/agents/shell-guard.md"),
        validation_errors: Vec::new(),
    }
}

#[test]
fn subagents_denies_restricted_tool_usage() {
    let config = SubagentConfig::enabled(SubagentDiscoveryMode::Auto);
    let inventory = SubagentBuilder::new(config.clone())
        .with_definition(restricted_definition())
        .build();
    let runner = SubagentRunner::new(&config, &inventory);

    let session = InvocationSession {
        parent_session_id: Some("root".into()),
        subagent_name: "shell-guard".into(),
        requested_tools: vec!["filesystem".into()],
        execution_log: Vec::new(),
        summary: None,
        detail_artifacts: Vec::new(),
        confirmed: true,
        resolved_model: None,
        extra_instructions: None,
    };

    let result = runner.invoke(session);
    assert!(matches!(
        result,
        Err(SubagentInvocationError::ToolNotAllowed { tool, .. }) if tool == "filesystem"
    ));
}
