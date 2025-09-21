use std::cmp::Ordering;

use crate::app::HookRegistryState;
use chrono::SecondsFormat;
use codex_protocol::hooks::{
    HookDefinition, HookEvent, HookLayerSummary, HookMatcher, HookMatchers, HookRegistrySnapshot,
    HookScope, HookSkipReason, SkippedHook,
};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};

pub(crate) fn build_hook_panel_lines(state: &HookRegistryState) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    lines.push(Line::from(vec!["Hook Inspector".bold()]));
    lines.push(Line::from(""));

    if let Some(err) = state.last_error() {
        lines.push(Line::from(vec![
            format!("Error loading hooks: {err}").red().bold(),
        ]));
        lines.push(Line::from(""));
    }

    let snapshot = state.snapshot();
    let total_hooks = total_hook_count(snapshot);
    let event_count = snapshot.events.len();
    let layer_count = snapshot.layers.len();

    if state.is_loading() && total_hooks == 0 && layer_count == 0 {
        lines.push(Line::from(vec!["Loading hook registry…".italic()]));
        return lines;
    }

    if state.is_loading() {
        lines.push(Line::from(vec!["Refreshing hook registry…".dim().italic()]));
        lines.push(Line::from(""));
    }

    let last_loaded = snapshot
        .last_loaded
        .map(|ts| ts.to_rfc3339_opts(SecondsFormat::Millis, true))
        .unwrap_or_else(|| "never".to_string());
    lines.push(Line::from(vec![
        format!("Last loaded: {last_loaded}").dim(),
    ]));
    lines.push(Line::from(vec![
        format!("Events: {event_count}").dim(),
        "  ".into(),
        format!("Hooks: {total_hooks}").dim(),
        "  ".into(),
        format!("Layers: {layer_count}").dim(),
    ]));

    if total_hooks == 0 && layer_count == 0 {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            "No hooks are currently configured.".italic().dim(),
        ]));
        return lines;
    }

    if !snapshot.layers.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec!["Layers".bold()]));
        let mut layers: Vec<&HookLayerSummary> = snapshot.layers.iter().collect();
        layers.sort_by(|a, b| scope_order(&a.scope).cmp(&scope_order(&b.scope)));
        for layer in layers {
            lines.push(render_layer_summary(layer));
            for skipped in &layer.skipped_hooks {
                lines.push(render_skipped_hook(skipped));
            }
        }
    }

    if !snapshot.events.is_empty() {
        lines.push(Line::from(""));
    }

    let mut events: Vec<(&HookEvent, &Vec<HookDefinition>)> = snapshot.events.iter().collect();
    events.sort_by(|(a, _), (b, _)| {
        event_order(a)
            .cmp(&event_order(b))
            .then_with(|| format_event(a).cmp(&format_event(b)))
    });

    for (idx, (event, hooks)) in events.iter().enumerate() {
        if idx > 0 {
            lines.push(Line::from(""));
        }
        lines.push(render_event_header(event, hooks.len()));
        let mut ordered: Vec<&HookDefinition> = hooks.iter().collect();
        ordered.sort_by(|a, b| compare_definitions(a, b));
        for definition in ordered {
            lines.push(render_hook_definition(definition));
            if let Some(notes) = &definition.notes {
                lines.push(render_notes_line(notes));
            }
            if let Some(filters) = render_matchers_line(&definition.matchers) {
                lines.push(filters);
            }
        }
    }

    lines
}

fn total_hook_count(snapshot: &HookRegistrySnapshot) -> usize {
    snapshot.events.values().map(|bucket| bucket.len()).sum()
}

fn render_layer_summary(layer: &HookLayerSummary) -> Line<'static> {
    let bullet: Span<'static> = "  • ".dim();
    let scope = scope_badge(&layer.scope);
    let path = layer.path.display().to_string();
    let skipped = layer.skipped_hooks.len();

    let mut spans = vec![bullet, scope, " ".into(), path.cyan()];
    spans.push("  (".dim());
    spans.push(format!("{} loaded", layer.loaded_hooks).dim());
    if skipped > 0 {
        spans.push(", ".dim());
        spans.push(format!("{} skipped", skipped).yellow().dim());
    }
    spans.push(")".dim());
    Line::from(spans)
}

fn render_skipped_hook(skipped: &SkippedHook) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = vec!["        ↳ skipped ".dim()];
    if let Some(id) = &skipped.hook_id {
        spans.push(id.clone().dim());
        spans.push(" ".into());
    }
    spans.push(
        format!("({})", format_skip_reason(&skipped.reason))
            .yellow()
            .dim(),
    );
    if let Some(details) = &skipped.details {
        spans.push(" – ".dim());
        spans.push(details.clone().dim());
    }
    Line::from(spans)
}

fn render_event_header(event: &HookEvent, count: usize) -> Line<'static> {
    let mut spans = vec![format_event(event).bold()];
    spans.push("  ".into());
    spans.push(format!("{count} hook{}", if count == 1 { "" } else { "s" }).dim());
    Line::from(spans)
}

fn render_hook_definition(definition: &HookDefinition) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = vec![
        "    • ".dim(),
        scope_badge(&definition.scope),
        " ".into(),
        definition.id.clone().bold(),
    ];

    if let Some(timeout) = definition.timeout_ms {
        spans.push("  ".into());
        spans.push(format!("timeout {timeout}ms").dim());
    }
    if definition.allow_parallel {
        spans.push("  parallel".dim());
    }
    if let Some(cmd) = summarize_command(&definition.command) {
        spans.push("  ".into());
        spans.push(cmd.dim());
    }
    Line::from(spans)
}

fn render_notes_line(notes: &str) -> Line<'static> {
    Line::from(vec!["        ↳ ".dim(), notes.to_string().italic().dim()])
}

fn render_matchers_line(matchers: &HookMatchers) -> Option<Line<'static>> {
    let mut parts: Vec<String> = Vec::new();
    if !matchers.tool_names.is_empty() {
        parts.push(format!(
            "tool={}",
            describe_matcher_list(&matchers.tool_names)
        ));
    }
    if !matchers.sources.is_empty() {
        parts.push(format!(
            "source={}",
            describe_matcher_list(&matchers.sources)
        ));
    }
    if !matchers.paths.is_empty() {
        parts.push(format!("path={}", describe_matcher_list(&matchers.paths)));
    }
    if !matchers.tags.is_empty() {
        let tags = matchers.tags.join(",");
        parts.push(format!("tags={tags}"));
    }
    if parts.is_empty() {
        return None;
    }
    let text = parts.join(" · ");
    Some(Line::from(vec!["        ↳ filters ".dim(), text.dim()]))
}

fn describe_matcher_list(matchers: &[HookMatcher]) -> String {
    matchers
        .iter()
        .map(describe_matcher)
        .collect::<Vec<_>>()
        .join(",")
}

fn describe_matcher(matcher: &HookMatcher) -> String {
    match matcher {
        HookMatcher::Exact { value } => format!("=\"{value}\""),
        HookMatcher::Glob { value } => format!("glob:{value}"),
        HookMatcher::Regex { value } => format!("re:{value}"),
    }
}

fn summarize_command(command: &[String]) -> Option<String> {
    if command.is_empty() {
        return None;
    }
    let joined = command.join(" ");
    Some(ellipsize(&joined, 60))
}

fn ellipsize(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let take = max_chars.saturating_sub(1);
    let truncated: String = text.chars().take(take).collect();
    format!("{truncated}…")
}

fn scope_badge(scope: &HookScope) -> Span<'static> {
    match scope {
        HookScope::ManagedPolicy { name } => format!("managed:{name}").cyan().bold(),
        HookScope::Project { .. } => "project".magenta().bold(),
        HookScope::LocalUser { .. } => "local".yellow().bold(),
    }
}

fn format_skip_reason(reason: &HookSkipReason) -> String {
    match reason {
        HookSkipReason::InvalidSchema => "invalid-schema".to_string(),
        HookSkipReason::UnsupportedVersion => "unsupported-version".to_string(),
        HookSkipReason::DuplicateId => "duplicate-id".to_string(),
        HookSkipReason::MissingExecutable => "missing-executable".to_string(),
        HookSkipReason::InvalidMatcher => "invalid-matcher".to_string(),
    }
}

fn compare_definitions(a: &HookDefinition, b: &HookDefinition) -> Ordering {
    scope_order(&a.scope)
        .cmp(&scope_order(&b.scope))
        .then_with(|| a.id.cmp(&b.id))
}

fn scope_order(scope: &HookScope) -> u8 {
    match scope {
        HookScope::ManagedPolicy { .. } => 0,
        HookScope::Project { .. } => 1,
        HookScope::LocalUser { .. } => 2,
    }
}

fn event_order(event: &HookEvent) -> usize {
    match event {
        HookEvent::PreToolUse => 0,
        HookEvent::PostToolUse => 1,
        HookEvent::UserPromptSubmit => 2,
        HookEvent::Notification => 3,
        HookEvent::Stop => 4,
        HookEvent::SubagentStop => 5,
        HookEvent::PreCompact => 6,
        HookEvent::SessionStart => 7,
        HookEvent::SessionEnd => 8,
    }
}

fn format_event(event: &HookEvent) -> String {
    format!("{event:?}")
}
