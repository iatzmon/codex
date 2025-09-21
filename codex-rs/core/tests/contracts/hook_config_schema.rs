use codex_core::hooks::config_loader::{HookConfigError, HookConfigLoader};
use pretty_assertions::assert_eq;

fn valid_config() -> &'static str {
    r#"
        schemaVersion = "1.0"

        [[hooks]]
        id = "project.shell.guard"
        event = "PreToolUse"
        command = ["./scripts/check.sh"]
        schemaVersions = ["1.0"]
    "#
}

fn invalid_config_missing_id() -> &'static str {
    r#"
        schemaVersion = "1.0"

        [[hooks]]
        event = "PreToolUse"
        command = ["./scripts/check.sh"]
        schemaVersions = ["1.0"]
    "#
}

#[test]
fn hook_config_allows_valid_minimal_document() {
    let result = HookConfigLoader::validate_document(valid_config());
    assert_eq!(result, Ok(()));
}

#[test]
fn hook_config_rejects_missing_id() {
    let result = HookConfigLoader::validate_document(invalid_config_missing_id());
    assert!(matches!(
        result,
        Err(HookConfigError::InvalidConfiguration(message)) if message.contains("id")
    ));
}
