use codex_slash_commands::CommandRegistry;
use codex_slash_commands::SlashCommandError;
use codex_slash_commands::models::command::Command;
use codex_slash_commands::models::metadata::FrontmatterMetadata;
use codex_slash_commands::models::scope::CommandScope;
use codex_slash_commands::registry::CommandLookup;
use pretty_assertions::assert_eq;
use std::path::PathBuf;

fn make_command(scope: CommandScope, namespace: &[&str], name: &str) -> Command {
    Command {
        scope,
        namespace: namespace.iter().map(|s| s.to_string()).collect(),
        name: name.to_string(),
        metadata: FrontmatterMetadata {
            description: Some(format!("{scope:?}:{name}")),
            argument_hint: None,
            model: None,
            allowed_tools: None,
        },
        body: format!("{name} body"),
        path: PathBuf::from(format!("/{scope:?}/{name}.md")),
    }
}

#[test]
fn inserts_and_looks_up_by_full_name() {
    let mut registry = CommandRegistry::new();
    let command = make_command(CommandScope::Project, &["web"], "deploy");
    registry
        .insert(command.clone())
        .expect("expected insert to succeed");

    match registry.lookup("project:web:deploy") {
        CommandLookup::Command(found) => {
            assert_eq!(found, command);
            assert_eq!(found.full_name(), "project:web:deploy");
        }
        other => panic!("unexpected lookup result: {other:?}"),
    }
}

#[test]
fn detects_ambiguous_unqualified_names() {
    let mut registry = CommandRegistry::new();
    registry
        .insert(make_command(CommandScope::Project, &[], "deploy"))
        .expect("expected insert to succeed");
    registry
        .insert(make_command(CommandScope::User, &[], "deploy"))
        .expect("expected insert to succeed");

    match registry.lookup("deploy") {
        CommandLookup::Ambiguous { matches: names } => {
            assert_eq!(names.len(), 2);
            assert!(names.contains(&"project:deploy".to_string()));
            assert!(names.contains(&"user:deploy".to_string()));
        }
        other => panic!("unexpected lookup result: {other:?}"),
    }
}

#[test]
fn returns_suggestions_for_missing_command() {
    let mut registry = CommandRegistry::new();
    registry
        .insert(make_command(CommandScope::Project, &["ops"], "deploy-prod"))
        .expect("expected insert");

    match registry.lookup("project:ops:deploy") {
        CommandLookup::NotFound { suggestions } => {
            assert!(suggestions.iter().any(|s| s.contains("deploy")));
        }
        other => panic!("unexpected lookup result: {other:?}"),
    }
}

#[test]
fn rejects_duplicate_fully_qualified_commands() {
    let mut registry = CommandRegistry::new();
    let first = make_command(CommandScope::Project, &[], "status");
    registry
        .insert(first.clone())
        .expect("expected insert to succeed");
    let err = registry
        .insert(first)
        .expect_err("expected duplicate insert to fail");
    assert!(matches!(err, SlashCommandError::DuplicateCommand { .. }));
}
