use codex_slash_commands::CommandRegistry;
use codex_slash_commands::SlashCommandConfig;
use codex_slash_commands::SlashCommandError;
use codex_slash_commands::models::scope::CommandScope;
use pretty_assertions::assert_eq;
use std::fs;

#[tokio::test]
async fn loads_commands_from_user_and_project_directories() -> Result<(), SlashCommandError> {
    let project_dir = tempfile::tempdir().expect("tempdir");
    let user_dir = tempfile::tempdir().expect("tempdir");

    fs::create_dir_all(project_dir.path().join("web")).expect("create project namespace");
    fs::write(
        project_dir.path().join("web/deploy.md"),
        "---\ndescription: Deploy web\n---\nDeploy $ARGUMENTS",
    )
    .expect("write project command");

    fs::write(
        user_dir.path().join("notes.md"),
        "---\ndescription: Notes\n---\nRemember $1",
    )
    .expect("write user command");

    let registry = CommandRegistry::load(&SlashCommandConfig {
        project_dir: Some(project_dir.path().to_path_buf()),
        user_dir: Some(user_dir.path().to_path_buf()),
    })
    .await?;

    let project = match registry.lookup("project:web:deploy") {
        codex_slash_commands::CommandLookup::Command(cmd) => cmd,
        other => panic!("unexpected lookup result: {other:?}"),
    };
    assert_eq!(project.scope, CommandScope::Project);
    assert_eq!(project.namespace, vec!["web".to_string()]);

    let user = match registry.lookup("user:notes") {
        codex_slash_commands::CommandLookup::Command(cmd) => cmd,
        other => panic!("unexpected lookup result: {other:?}"),
    };
    assert_eq!(user.scope, CommandScope::User);

    match registry.lookup("notes") {
        codex_slash_commands::CommandLookup::Command(cmd) => {
            assert_eq!(cmd.name, "notes");
            assert_eq!(cmd.full_name(), "user:notes");
        }
        other => panic!("unexpected lookup result: {other:?}"),
    }

    Ok(())
}
