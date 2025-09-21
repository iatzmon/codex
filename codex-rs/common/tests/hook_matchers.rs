use codex_core::hooks::registry::HookRegistry;
use codex_core::hooks::{HookDefinition, HookEvent, HookMatcher, HookMatchers, HookScope};
use serde_json::json;
use std::path::PathBuf;

fn definition_with_scope(id: &str, scope: HookScope) -> HookDefinition {
    let mut definition = HookDefinition::new(id.to_string(), HookEvent::PreToolUse, scope);
    definition.command = vec!["/bin/true".into()];
    definition
}

#[test]
fn matcher_serializes_with_type_tag() {
    let matcher = HookMatcher::Glob {
        value: "shell*".to_string(),
    };
    let serialized = serde_json::to_value(&matcher).expect("serialize matcher");
    assert_eq!(serialized, json!({"type": "glob", "value": "shell*"}));

    let matchers = HookMatchers {
        tool_names: vec![matcher],
        sources: Vec::new(),
        paths: Vec::new(),
        tags: Vec::new(),
    };
    let serialized_group = serde_json::to_value(&matchers).expect("serialize group");
    assert_eq!(
        serialized_group,
        json!({"toolNames": [{"type": "glob", "value": "shell*"}]})
    );
}

#[test]
fn registry_orders_hooks_by_scope_then_id() {
    let mut registry = HookRegistry::new();

    let managed = definition_with_scope(
        "managed.alpha",
        HookScope::ManagedPolicy {
            name: "org-policy".into(),
        },
    );
    let mut project = definition_with_scope(
        "project.alpha",
        HookScope::Project {
            project_root: PathBuf::from("/workspace/project"),
        },
    );
    project.id = "project.beta".into();
    let local = definition_with_scope(
        "local.alpha",
        HookScope::LocalUser {
            codex_home: PathBuf::from("/home/user/.codex"),
        },
    );

    registry.insert(vec![local.clone(), project.clone(), managed.clone()]);

    let hooks = registry.hooks_for_event(&HookEvent::PreToolUse);
    let ordered_ids: Vec<&str> = hooks.iter().map(|hook| hook.id.as_str()).collect();
    assert_eq!(
        ordered_ids,
        vec!["managed.alpha", "project.beta", "local.alpha"]
    );

    // Ensure that inserting another hook with the same scope respects id ordering.
    let mut another_managed = managed.clone();
    another_managed.id = "managed.beta".into();
    registry.insert(vec![another_managed]);
    let hooks = registry.hooks_for_event(&HookEvent::PreToolUse);
    let ordered_ids: Vec<&str> = hooks.iter().map(|hook| hook.id.as_str()).collect();
    assert_eq!(
        ordered_ids,
        vec![
            "managed.alpha",
            "managed.beta",
            "project.beta",
            "local.alpha"
        ]
    );
}
