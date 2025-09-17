use codex_slash_commands::CommandRegistry;
use codex_slash_commands::InterpolationContext;
use codex_slash_commands::SlashCommandConfig;
use codex_slash_commands::interpolate_template;
use codex_slash_commands::models::scope::CommandScope;
use pretty_assertions::assert_eq;
use std::fs;

#[tokio::test]
async fn end_to_end_invocation_applies_model_override() {
    let project_dir = tempfile::tempdir().expect("tempdir");
    let command_body = r#"---
model: gpt-4o
argument_hint: "<branch>"
---
Deploy $1 to production using $ARGUMENTS
"#;
    fs::create_dir_all(project_dir.path().join("ops")).expect("create namespace");
    fs::write(project_dir.path().join("ops/deploy.md"), command_body).expect("write command");

    let registry = CommandRegistry::load(&SlashCommandConfig {
        project_dir: Some(project_dir.path().to_path_buf()),
        user_dir: None,
    })
    .await
    .expect("expected registry to load");

    let command = match registry.lookup("project:ops:deploy") {
        codex_slash_commands::CommandLookup::Command(cmd) => cmd,
        other => panic!("unexpected lookup result: {other:?}"),
    };

    assert_eq!(command.scope, CommandScope::Project);
    assert_eq!(command.metadata.model.as_deref(), Some("gpt-4o"));
    assert_eq!(command.metadata.argument_hint.as_deref(), Some("<branch>"));

    let prompt = interpolate_template(
        &command.body,
        &InterpolationContext::new(vec!["feature/login".to_string(), "extra".to_string()]),
    )
    .expect("expected interpolation to succeed");

    assert_eq!(
        prompt.trim(),
        "Deploy feature/login to production using feature/login extra"
    );
}
