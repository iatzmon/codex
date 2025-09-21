use std::collections::HashMap;

use super::HookEvent as CoreHookEvent;
use super::definition::HookDefinition as CoreHookDefinition;
use super::layer_summary::HookLayerSummary as CoreHookLayerSummary;
use super::matchers::{HookMatcher as CoreHookMatcher, HookMatchers as CoreHookMatchers};
use super::registry::HookRegistry as CoreHookRegistry;
use super::scope::HookScope;
use super::skipped::{HookSkipReason as CoreHookSkipReason, SkippedHook as CoreSkippedHook};
use codex_protocol::hooks::{
    HookDefinition as ProtoHookDefinition, HookLayerSummary as ProtoLayerSummary, HookListRequest,
    HookMatcher as ProtoHookMatcher, HookMatchers as ProtoHookMatchers, HookRegistrySnapshot,
    HookScope as ProtoHookScope, HookScopeFilter, HookSkipReason as ProtoHookSkipReason,
    SkippedHook as ProtoSkippedHook,
};

pub fn build_hook_registry_snapshot(
    registry: &CoreHookRegistry,
    request: &HookListRequest,
) -> HookRegistrySnapshot {
    let event_filter = request.event.as_ref();
    let scope_filter = request.scope.as_ref();

    let mut events = HashMap::new();
    for (event, definitions) in registry.events.iter() {
        if let Some(filter_event) = event_filter {
            if convert_hook_event(event) != *filter_event {
                continue;
            }
        }

        let converted_defs: Vec<_> = definitions
            .iter()
            .filter(|definition| scope_matches(scope_filter, &definition.scope))
            .map(convert_hook_definition)
            .collect();

        if !converted_defs.is_empty() {
            events.insert(convert_hook_event(event), converted_defs);
        }
    }

    let layers = registry
        .source_layers
        .iter()
        .filter_map(|layer| convert_layer_summary(layer, scope_filter))
        .collect();

    HookRegistrySnapshot {
        events,
        layers,
        last_loaded: registry.last_loaded,
    }
}

fn convert_layer_summary(
    summary: &CoreHookLayerSummary,
    scope_filter: Option<&HookScopeFilter>,
) -> Option<ProtoLayerSummary> {
    if !scope_matches(scope_filter, &summary.scope) {
        return None;
    }

    Some(ProtoLayerSummary {
        scope: convert_hook_scope(&summary.scope),
        path: summary.path.clone(),
        checksum: summary.checksum.clone(),
        loaded_hooks: summary.loaded_hooks,
        skipped_hooks: summary
            .skipped_hooks
            .iter()
            .map(convert_skipped_hook)
            .collect(),
    })
}

fn convert_hook_definition(definition: &CoreHookDefinition) -> ProtoHookDefinition {
    ProtoHookDefinition {
        id: definition.id.clone(),
        event: convert_hook_event(&definition.event),
        notes: definition.notes.clone(),
        command: definition.command.clone(),
        working_dir: definition.working_dir.clone(),
        timeout_ms: definition.timeout_ms,
        allow_parallel: definition.allow_parallel,
        schema_versions: definition.schema_versions.clone(),
        env: definition.env.clone(),
        matchers: convert_hook_matchers(&definition.matchers),
        scope: convert_hook_scope(&definition.scope),
        source_path: definition.source_path.clone(),
    }
}

fn convert_hook_scope(scope: &HookScope) -> ProtoHookScope {
    match scope {
        HookScope::ManagedPolicy { name } => ProtoHookScope::ManagedPolicy { name: name.clone() },
        HookScope::Project { project_root } => ProtoHookScope::Project {
            project_root: project_root.clone(),
        },
        HookScope::LocalUser { codex_home } => ProtoHookScope::LocalUser {
            codex_home: codex_home.clone(),
        },
    }
}

fn convert_hook_event(event: &CoreHookEvent) -> codex_protocol::hooks::HookEvent {
    match event {
        CoreHookEvent::PreToolUse => codex_protocol::hooks::HookEvent::PreToolUse,
        CoreHookEvent::PostToolUse => codex_protocol::hooks::HookEvent::PostToolUse,
        CoreHookEvent::UserPromptSubmit => codex_protocol::hooks::HookEvent::UserPromptSubmit,
        CoreHookEvent::Notification => codex_protocol::hooks::HookEvent::Notification,
        CoreHookEvent::Stop => codex_protocol::hooks::HookEvent::Stop,
        CoreHookEvent::SubagentStop => codex_protocol::hooks::HookEvent::SubagentStop,
        CoreHookEvent::PreCompact => codex_protocol::hooks::HookEvent::PreCompact,
        CoreHookEvent::SessionStart => codex_protocol::hooks::HookEvent::SessionStart,
        CoreHookEvent::SessionEnd => codex_protocol::hooks::HookEvent::SessionEnd,
    }
}

fn convert_hook_matchers(matchers: &CoreHookMatchers) -> ProtoHookMatchers {
    ProtoHookMatchers {
        tool_names: matchers
            .tool_names
            .iter()
            .map(convert_hook_matcher)
            .collect(),
        sources: matchers.sources.iter().map(convert_hook_matcher).collect(),
        paths: matchers.paths.iter().map(convert_hook_matcher).collect(),
        tags: matchers.tags.clone(),
    }
}

fn convert_hook_matcher(matcher: &CoreHookMatcher) -> ProtoHookMatcher {
    match matcher {
        CoreHookMatcher::Exact { value } => ProtoHookMatcher::Exact {
            value: value.clone(),
        },
        CoreHookMatcher::Glob { value } => ProtoHookMatcher::Glob {
            value: value.clone(),
        },
        CoreHookMatcher::Regex { value } => ProtoHookMatcher::Regex {
            value: value.clone(),
        },
    }
}

fn convert_skipped_hook(skipped: &CoreSkippedHook) -> ProtoSkippedHook {
    ProtoSkippedHook {
        hook_id: skipped.hook_id.clone(),
        reason: convert_hook_skip_reason(&skipped.reason),
        details: skipped.details.clone(),
    }
}

fn convert_hook_skip_reason(reason: &CoreHookSkipReason) -> ProtoHookSkipReason {
    match reason {
        CoreHookSkipReason::InvalidSchema => ProtoHookSkipReason::InvalidSchema,
        CoreHookSkipReason::UnsupportedVersion => ProtoHookSkipReason::UnsupportedVersion,
        CoreHookSkipReason::DuplicateId => ProtoHookSkipReason::DuplicateId,
        CoreHookSkipReason::MissingExecutable => ProtoHookSkipReason::MissingExecutable,
        CoreHookSkipReason::InvalidMatcher => ProtoHookSkipReason::InvalidMatcher,
    }
}

fn scope_matches(filter: Option<&HookScopeFilter>, scope: &HookScope) -> bool {
    match filter {
        None => true,
        Some(HookScopeFilter::Managed) => matches!(scope, HookScope::ManagedPolicy { .. }),
        Some(HookScopeFilter::Project) => matches!(scope, HookScope::Project { .. }),
        Some(HookScopeFilter::Local) => matches!(scope, HookScope::LocalUser { .. }),
    }
}
