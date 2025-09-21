use codex_core::hooks::config_loader::HookConfigLoader;

#[test]
fn legacy_notify_configuration_synthesizes_notification_hook() {
    let legacy_config = r#"
        [notifications]
        notify = ["notify-send", "Codex"]
    "#;

    HookConfigLoader::synthesize_legacy_notify(legacy_config)
        .expect("legacy notify synthesis should produce Notification hook");
}
