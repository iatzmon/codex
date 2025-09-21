use std::path::PathBuf;

use chrono::Utc;
use codex_core::hooks::{HookDefinition, HookEvent, HookLayerSummary, HookRegistry, HookScope};

fn layer(scope: HookScope) -> HookLayerSummary {
    HookLayerSummary::new(scope, PathBuf::from("/tmp/mock.toml"))
}

#[test]
fn registry_orders_hooks_by_scope_precedence() {
    let mut managed = HookDefinition::new(
        "managed.guard",
        HookEvent::PreToolUse,
        HookScope::ManagedPolicy {
            name: "policy".into(),
        },
    );
    managed.schema_versions = vec!["1.0".into()];

    let mut project = HookDefinition::new(
        "project.guard",
        HookEvent::PreToolUse,
        HookScope::Project {
            project_root: PathBuf::from("/workspace"),
        },
    );
    project.schema_versions = vec!["1.0".into()];

    let mut local = HookDefinition::new(
        "local.guard",
        HookEvent::PreToolUse,
        HookScope::LocalUser {
            codex_home: PathBuf::from("/home/user/.codex"),
        },
    );
    local.schema_versions = vec!["1.0".into()];

    let registry = HookRegistry::with_layers(
        vec![local, project, managed],
        vec![layer(HookScope::ManagedPolicy {
            name: "policy".into(),
        })],
        Utc::now(),
    );

    let hooks = registry.hooks_for_event(&HookEvent::PreToolUse);
    let ids: Vec<_> = hooks.iter().map(|hook| hook.id.as_str()).collect();
    assert_eq!(ids, vec!["managed.guard", "project.guard", "local.guard"]);
}
