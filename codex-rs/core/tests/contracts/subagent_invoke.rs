use std::path::PathBuf;

use codex_core::subagents::{
    builder::SubagentBuilder,
    config::{SubagentConfig, SubagentDiscoveryMode},
    definition::{SubagentDefinition, SubagentScope},
    inventory::SubagentInventory,
    invocation::InvocationSession,
    runner::{SubagentInvocationError, SubagentRunner},
};
use pretty_assertions::assert_eq;

fn make_definition(name: &str, scope: SubagentScope) -> SubagentDefinition {
    SubagentDefinition {
        raw_name: name.to_string(),
        name: name.to_string(),
        description: format!("{name} description"),
        tools: vec!["git_diff".into()],
        model: Some("gpt-4.1-mini".into()),
        instructions: format!("Run the {name} playbook."),
        scope,
        source_path: PathBuf::from(format!(
            "/home/iatzmon/workspace/codex/.codex/agents/{name}.md"
        )),
        validation_errors: Vec::new(),
    }
}

fn inventory_from_definition(
    config: &SubagentConfig,
    definition: SubagentDefinition,
) -> SubagentInventory {
    SubagentBuilder::new(config.clone())
        .with_definition(definition)
        .build()
}

#[test]
fn invoke_applies_tool_allowlist() {
    let config = SubagentConfig::enabled(SubagentDiscoveryMode::Auto);
    let definition = make_definition("code-reviewer", SubagentScope::Project);
    let inventory = inventory_from_definition(&config, definition.clone());
    let runner = SubagentRunner::new(&config, &inventory);

    let session = InvocationSession {
        parent_session_id: Some("root".into()),
        subagent_name: "code-reviewer".into(),
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

#[test]
fn invoke_uses_configured_model() {
    let mut config = SubagentConfig::enabled(SubagentDiscoveryMode::Auto);
    config.default_model = Some("gpt-4o-mini".into());
    let definition = SubagentDefinition {
        model: None,
        ..make_definition("code-reviewer", SubagentScope::Project)
    };
    let inventory = inventory_from_definition(&config, definition.clone());
    let runner = SubagentRunner::new(&config, &inventory);

    let session = InvocationSession {
        parent_session_id: Some("root".into()),
        subagent_name: "code-reviewer".into(),
        requested_tools: vec!["git_diff".into()],
        execution_log: Vec::new(),
        summary: None,
        detail_artifacts: Vec::new(),
        confirmed: true,
        resolved_model: None,
        extra_instructions: None,
    };

    let result = runner
        .invoke(session)
        .expect("invocation should resolve model");
    assert_eq!(
        result.session.resolved_model.as_deref(),
        Some("gpt-4o-mini")
    );
}

#[test]
fn invoke_requires_confirmation_for_auto_suggest() {
    let config = SubagentConfig::enabled(SubagentDiscoveryMode::Auto);
    let definition = make_definition("code-reviewer", SubagentScope::Project);
    let inventory = inventory_from_definition(&config, definition.clone());
    let runner = SubagentRunner::new(&config, &inventory);

    let session = InvocationSession {
        parent_session_id: Some("root".into()),
        subagent_name: "code-reviewer".into(),
        requested_tools: vec!["git_diff".into()],
        execution_log: Vec::new(),
        summary: None,
        detail_artifacts: Vec::new(),
        confirmed: false,
        resolved_model: None,
        extra_instructions: None,
    };

    let result = runner.invoke(session);
    assert!(matches!(
        result,
        Err(SubagentInvocationError::ConfirmationRequired(name)) if name == "code-reviewer"
    ));
}

#[test]
fn invoke_preserves_extra_instructions() {
    let config = SubagentConfig::enabled(SubagentDiscoveryMode::Manual);
    let definition = make_definition("code-reviewer", SubagentScope::Project);
    let inventory = inventory_from_definition(&config, definition.clone());
    let runner = SubagentRunner::new(&config, &inventory);

    let mut session = InvocationSession::new("code-reviewer").confirmed();
    session.extra_instructions = Some("Focus on regression tests".into());

    let prepared = runner
        .invoke(session)
        .expect("invocation should carry extra instructions");

    assert_eq!(
        prepared.session.extra_instructions.as_deref(),
        Some("Focus on regression tests")
    );
}
