use std::path::PathBuf;

use codex_core::subagents::{
    builder::SubagentBuilder,
    config::{SubagentConfig, SubagentDiscoveryMode},
    definition::{SubagentDefinition, SubagentScope},
};
use pretty_assertions::assert_eq;

fn make_definition(name: &str, scope: SubagentScope, path: &str) -> SubagentDefinition {
    SubagentDefinition {
        raw_name: name.to_string(),
        name: name.to_string(),
        description: format!("{name} description"),
        tools: vec!["git_diff".into()],
        model: Some("gpt-4.1-mini".into()),
        instructions: format!("Run the {name} playbook."),
        scope,
        source_path: PathBuf::from(path),
        validation_errors: Vec::new(),
    }
}

#[test]
fn agents_list_returns_project_override_first() {
    let config = SubagentConfig::enabled(SubagentDiscoveryMode::Auto);
    let user_def = make_definition(
        "code-reviewer",
        SubagentScope::User,
        "/home/user/.codex/agents/code-reviewer.md",
    );
    let mut project_def = make_definition(
        "code-reviewer",
        SubagentScope::Project,
        "/home/iatzmon/workspace/codex/.codex/agents/code-reviewer.md",
    );
    project_def.description = "project override description".into();

    let inventory = SubagentBuilder::new(config)
        .with_definitions(vec![user_def.clone(), project_def.clone()])
        .build();

    assert_eq!(
        inventory.subagents.len(),
        1,
        "only one subagent should remain after precedence resolution"
    );
    let record = inventory
        .subagents
        .get("code-reviewer")
        .expect("project definition should win precedence");
    assert_eq!(record.definition.scope, SubagentScope::Project);
    assert_eq!(
        record.definition.description,
        "project override description"
    );

    assert_eq!(
        inventory.conflicts.len(),
        1,
        "user definition should be recorded as a conflict"
    );
    let conflict = &inventory.conflicts[0];
    assert_eq!(conflict.name, "code-reviewer");
    assert_eq!(conflict.losing_scope, SubagentScope::User);
    assert_eq!(conflict.reason, "project override");
}

#[test]
fn agents_list_filters_invalid_definitions() {
    let config = SubagentConfig::enabled(SubagentDiscoveryMode::Auto);
    let valid_def = make_definition(
        "code-reviewer",
        SubagentScope::Project,
        "/home/iatzmon/workspace/codex/.codex/agents/code-reviewer.md",
    );
    let mut invalid_def = make_definition(
        "broken-agent",
        SubagentScope::User,
        "/home/user/.codex/agents/broken-agent.md",
    );
    invalid_def
        .validation_errors
        .push("missing description".into());

    let inventory = SubagentBuilder::new(config)
        .with_definitions(vec![valid_def.clone(), invalid_def.clone()])
        .build();

    assert!(
        inventory.subagents.contains_key("code-reviewer"),
        "valid project agent should appear in list",
    );
    assert!(
        !inventory.subagents.contains_key("broken-agent"),
        "invalid agent should not show up by default",
    );

    let invalid: Vec<_> = inventory
        .invalid()
        .into_iter()
        .map(|record| record.definition.name.as_str())
        .collect();
    assert_eq!(invalid, vec!["broken-agent"]);
}

#[test]
fn project_invalid_falls_back_to_user_definition() {
    let config = SubagentConfig::enabled(SubagentDiscoveryMode::Auto);

    let mut project_def = make_definition(
        "code-reviewer",
        SubagentScope::Project,
        "/home/iatzmon/workspace/codex/.codex/agents/code-reviewer.md",
    );
    project_def
        .validation_errors
        .push("missing instructions".into());

    let user_def = make_definition(
        "code-reviewer",
        SubagentScope::User,
        "/home/user/.codex/agents/code-reviewer.md",
    );

    let inventory = SubagentBuilder::new(config)
        .with_definitions(vec![project_def.clone(), user_def.clone()])
        .build();

    let record = inventory
        .subagents
        .get("code-reviewer")
        .expect("user definition should be selected when project is invalid");
    assert_eq!(record.definition.scope, SubagentScope::User);

    let invalid: Vec<_> = inventory
        .invalid()
        .into_iter()
        .map(|record| record.definition.source_path.clone())
        .collect();
    assert_eq!(invalid, vec![project_def.source_path.clone()]);

    let conflict = inventory
        .conflicts
        .iter()
        .find(|conflict| conflict.losing_scope == SubagentScope::Project)
        .expect("invalid project definition should be captured as conflict");
    assert_eq!(conflict.reason, "invalid definition");
}
