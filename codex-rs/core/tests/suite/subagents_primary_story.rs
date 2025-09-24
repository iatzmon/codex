use std::path::PathBuf;

use codex_core::subagents::{
    builder::SubagentBuilder,
    config::{SubagentConfig, SubagentDiscoveryMode},
    definition::{SubagentDefinition, SubagentScope},
    invocation::InvocationSession,
    runner::SubagentRunner,
};
use pretty_assertions::assert_eq;

fn project_definition() -> SubagentDefinition {
    SubagentDefinition {
        raw_name: "code-reviewer".into(),
        name: "code-reviewer".into(),
        description: "Reviews staged diffs for safety regressions".into(),
        tools: vec!["git_diff".into(), "tests".into()],
        model: Some("gpt-4.1-mini".into()),
        instructions: "Review staged diffs and execute tests as needed.".into(),
        scope: SubagentScope::Project,
        source_path: PathBuf::from("/home/iatzmon/workspace/codex/.codex/agents/code-reviewer.md"),
        validation_errors: Vec::new(),
    }
}

fn user_definition() -> SubagentDefinition {
    SubagentDefinition {
        raw_name: "code-reviewer".into(),
        name: "code-reviewer".into(),
        description: "User fallback reviewer".into(),
        tools: vec!["git_diff".into()],
        model: Some("gpt-4o-mini".into()),
        instructions: "Fallback reviewer instructions.".into(),
        scope: SubagentScope::User,
        source_path: PathBuf::from("/home/user/.codex/agents/code-reviewer.md"),
        validation_errors: Vec::new(),
    }
}

#[test]
fn subagents_primary_story_project_override_and_summary() {
    let mut config = SubagentConfig::enabled(SubagentDiscoveryMode::Auto);
    config.default_model = Some("gpt-4.1-mini".into());

    let inventory = SubagentBuilder::new(config.clone())
        .with_definition(user_definition())
        .with_definition(project_definition())
        .build();

    assert_eq!(
        inventory.subagents.len(),
        1,
        "project should override user definition"
    );

    let session = InvocationSession {
        parent_session_id: Some("root".into()),
        subagent_name: "code-reviewer".into(),
        requested_tools: vec!["git_diff".into(), "tests".into()],
        execution_log: Vec::new(),
        summary: None,
        detail_artifacts: Vec::new(),
        confirmed: true,
        resolved_model: None,
        extra_instructions: None,
    };

    let prepared = SubagentRunner::new(&config, &inventory)
        .invoke(session)
        .expect("invocation should succeed when confirmed");

    assert_eq!(prepared.session.subagent_name, "code-reviewer");
    assert_eq!(
        prepared.session.resolved_model.as_deref(),
        Some("gpt-4.1-mini")
    );
    assert!(
        prepared.session.summary.is_none(),
        "summary should be captured during execution"
    );
    assert!(
        prepared.session.detail_artifacts.is_empty(),
        "detail artifacts are assigned after execution"
    );
}
