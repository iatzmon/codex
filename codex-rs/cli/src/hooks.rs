use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use anyhow::{Context, Result, anyhow, bail};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand, ValueEnum};
use codex_common::CliConfigOverrides;
use codex_core::config::{Config, ConfigOverrides};
use codex_core::hooks::HookEvent as CoreHookEvent;
use codex_core::hooks::HookScope;
use codex_core::hooks::execution_record::HookExecutionRecord;
use codex_core::hooks::registry::HookRegistry;
use codex_core::hooks::skipped::HookSkipReason as CoreHookSkipReason;
use codex_core::hooks::snapshot::build_hook_registry_snapshot;
use codex_protocol::hooks::{
    HookEvent as ProtoHookEvent, HookListRequest, HookRegistrySnapshot,
    HookScope as ProtoHookScope, HookScopeFilter, HookSkipReason as ProtoHookSkipReason,
    HookValidationStatus,
};
use serde::Serialize;

#[derive(Debug, Parser)]
pub struct HooksCli {
    #[clap(subcommand)]
    pub command: HooksSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum HooksSubcommand {
    List(ListArgs),
    ExecLog(ExecLogArgs),
    Validate(ValidateArgs),
    Reload,
}

#[derive(Debug, Parser)]
pub struct ListArgs {
    /// Filter by lifecycle event (e.g. PreToolUse)
    #[arg(long = "event")]
    pub event: Option<String>,

    /// Filter by configuration layer scope
    #[arg(long = "scope")]
    pub scope: Option<ScopeFilterOption>,

    /// Emit JSON instead of a human-friendly table
    #[arg(long = "json")]
    pub json: bool,
}

#[derive(Debug, Parser)]
pub struct ExecLogArgs {
    /// Only return records at or after this ISO-8601 timestamp
    #[arg(long = "since")]
    pub since: Option<String>,

    /// Filter by lifecycle event name
    #[arg(long = "event")]
    pub event: Option<String>,

    /// Filter by hook identifier
    #[arg(long = "hook-id")]
    pub hook_id: Option<String>,

    /// Limit to the most recent N records
    #[arg(long = "tail")]
    pub tail: Option<u32>,

    /// Emit JSON output
    #[arg(long = "json")]
    pub json: bool,
}

#[derive(Debug, Parser)]
pub struct ValidateArgs {
    /// Filter validation summary to a particular scope
    #[arg(long = "scope")]
    pub scope: Option<ScopeFilterOption>,

    /// Emit JSON output
    #[arg(long = "json")]
    pub json: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ScopeFilterOption {
    Managed,
    Project,
    Local,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ListJsonLayer {
    scope: String,
    path: String,
    checksum: String,
    loaded_hooks: usize,
    skipped_hooks: Vec<SkippedHookJson>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SkippedHookJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    hook_id: Option<String>,
    reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ListJsonHook {
    id: String,
    scope: String,
    command: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    notes: Option<String>,
}

#[derive(Debug, Serialize)]
struct ListJsonEvent {
    event: String,
    hooks: Vec<ListJsonHook>,
}

#[derive(Debug, Serialize)]
struct ListJson {
    layers: Vec<ListJsonLayer>,
    events: Vec<ListJsonEvent>,
}

#[derive(Debug, Serialize)]
struct ExecLogJsonRecord {
    time: String,
    event: String,
    hook: String,
    decision: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ValidateLayerJson {
    scope: String,
    path: String,
    loaded_hooks: usize,
    checksum: String,
    skipped_hooks: Vec<SkippedHookJson>,
}

#[derive(Debug, Serialize)]
struct ValidateJson {
    status: String,
    layers: Vec<ValidateLayerJson>,
}

pub fn run_hooks_cli(cli: HooksCli, root_overrides: CliConfigOverrides) -> Result<()> {
    match cli.command {
        HooksSubcommand::List(args) => run_list(args, root_overrides),
        HooksSubcommand::ExecLog(args) => run_exec_log(args, root_overrides),
        HooksSubcommand::Validate(args) => run_validate(args, root_overrides),
        HooksSubcommand::Reload => run_reload(),
    }
}

fn run_list(args: ListArgs, root_overrides: CliConfigOverrides) -> Result<()> {
    let config = load_config(root_overrides)?;
    let request = HookListRequest {
        event: match args.event.as_deref() {
            Some(name) => Some(parse_hook_event(name)?),
            None => None,
        },
        scope: args.scope.map(|s| s.to_filter()),
    };

    let snapshot = build_hook_registry_snapshot(&config.hook_registry, &request);
    if args.json {
        let json = build_list_json(&snapshot);
        println!("{}", serde_json::to_string_pretty(&json)?);
    } else {
        render_list(&snapshot);
    }
    Ok(())
}

fn run_exec_log(args: ExecLogArgs, root_overrides: CliConfigOverrides) -> Result<()> {
    let config = load_config(root_overrides)?;
    let log_path = hooks_log_path(&config);
    let mut records = read_exec_log(&log_path)?;

    if let Some(event_name) = args.event.as_deref() {
        let event_label = format!("{:?}", parse_hook_event(event_name)?);
        records.retain(|record| format_core_event(&record.event) == event_label);
    }

    if let Some(ref hook_id) = args.hook_id {
        records.retain(|record| record.hook_id == *hook_id);
    }

    if let Some(ref since) = args.since {
        let since = parse_datetime(since)?;
        records.retain(|record| record.timestamp >= since);
    }

    if let Some(tail) = args.tail {
        if tail == 0 {
            records.clear();
        } else if records.len() > tail as usize {
            records = records.split_off(records.len() - tail as usize);
        }
    }

    if args.json {
        let json_records: Vec<_> = records.iter().map(exec_record_to_json).collect();
        println!("{}", serde_json::to_string_pretty(&json_records)?);
    } else {
        render_exec_log(&records);
    }

    Ok(())
}

fn run_validate(args: ValidateArgs, root_overrides: CliConfigOverrides) -> Result<()> {
    let config = load_config(root_overrides)?;
    let request = HookListRequest {
        event: None,
        scope: args.scope.map(|s| s.to_filter()),
    };
    let snapshot = build_hook_registry_snapshot(&config.hook_registry, &request);
    let (status, layers) = validation_summary(&config.hook_registry, args.scope);

    if args.json {
        let json = ValidateJson {
            status: status_label(&status).to_string(),
            layers,
        };
        println!("{}", serde_json::to_string_pretty(&json)?);
    } else {
        render_validation(&snapshot, &status);
    }

    match status {
        HookValidationStatus::Ok => Ok(()),
        HookValidationStatus::Warning => {
            std::process::exit(3);
        }
        HookValidationStatus::Error => {
            std::process::exit(2);
        }
    }
}

fn run_reload() -> Result<()> {
    println!("Hook reload requests are only supported inside an active Codex session.");
    Ok(())
}

fn load_config(cli_overrides: CliConfigOverrides) -> Result<Config> {
    let overrides = cli_overrides
        .parse_overrides()
        .map_err(|e| anyhow!("Error parsing -c overrides: {e}"))?;
    Config::load_with_cli_overrides(overrides, ConfigOverrides::default())
        .map_err(|e| anyhow!("Error loading configuration: {e}"))
}

fn parse_hook_event(value: &str) -> Result<ProtoHookEvent> {
    match value.to_ascii_lowercase().as_str() {
        "pretooluse" | "pre-tool-use" => Ok(ProtoHookEvent::PreToolUse),
        "posttooluse" | "post-tool-use" => Ok(ProtoHookEvent::PostToolUse),
        "userpromptsubmit" | "user-prompt-submit" => Ok(ProtoHookEvent::UserPromptSubmit),
        "notification" => Ok(ProtoHookEvent::Notification),
        "stop" => Ok(ProtoHookEvent::Stop),
        "subagentstop" | "subagent-stop" => Ok(ProtoHookEvent::SubagentStop),
        "precompact" | "pre-compact" => Ok(ProtoHookEvent::PreCompact),
        "sessionstart" | "session-start" => Ok(ProtoHookEvent::SessionStart),
        "sessionend" | "session-end" => Ok(ProtoHookEvent::SessionEnd),
        other => bail!("Unknown hook event: {other}"),
    }
}

impl ScopeFilterOption {
    fn to_filter(self) -> HookScopeFilter {
        match self {
            ScopeFilterOption::Managed => HookScopeFilter::Managed,
            ScopeFilterOption::Project => HookScopeFilter::Project,
            ScopeFilterOption::Local => HookScopeFilter::Local,
        }
    }
}

fn build_list_json(snapshot: &HookRegistrySnapshot) -> ListJson {
    let layers = snapshot
        .layers
        .iter()
        .map(|layer| ListJsonLayer {
            scope: format_proto_scope(&layer.scope),
            path: layer.path.display().to_string(),
            checksum: layer.checksum.clone(),
            loaded_hooks: layer.loaded_hooks,
            skipped_hooks: layer
                .skipped_hooks
                .iter()
                .map(|skipped| SkippedHookJson {
                    hook_id: skipped.hook_id.clone(),
                    reason: format_proto_skip_reason(&skipped.reason),
                    details: skipped.details.clone(),
                })
                .collect(),
        })
        .collect();

    let mut events: Vec<ListJsonEvent> = snapshot
        .events
        .iter()
        .map(|(event, hooks)| {
            let mut hooks_json: Vec<ListJsonHook> = hooks
                .iter()
                .map(|hook| ListJsonHook {
                    id: hook.id.clone(),
                    scope: format_proto_scope(&hook.scope),
                    command: hook.command.clone(),
                    notes: hook.notes.clone(),
                })
                .collect();
            hooks_json.sort_by(|a, b| a.id.cmp(&b.id));
            ListJsonEvent {
                event: format_proto_event(event),
                hooks: hooks_json,
            }
        })
        .collect();

    events.sort_by(|a, b| a.event.cmp(&b.event));

    ListJson { layers, events }
}

fn render_list(snapshot: &HookRegistrySnapshot) {
    let mut rows = Vec::new();
    for (event, hooks) in snapshot.events.iter() {
        let event_name = format_proto_event(event);
        for hook in hooks {
            rows.push((
                event_name.clone(),
                format_proto_scope(&hook.scope),
                hook.id.clone(),
                hook.command.join(" "),
            ));
        }
    }

    if rows.is_empty() {
        println!("No hooks are currently registered.");
        return;
    }

    rows.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.2.cmp(&b.2)));

    println!("{:<15} {:<8} {}", "Event", "Scope", "ID");
    for (event, scope, id, command) in rows {
        println!("{:<15} {:<8} {}", event, scope, id);
        if !command.is_empty() {
            println!("    Command: {}", command);
        }
    }
}

fn render_exec_log(records: &[HookExecutionRecord]) {
    if records.is_empty() {
        println!("No hook executions recorded yet.");
        return;
    }

    println!(
        "{:<25} {:<12} {:<24} {:<10}",
        "Timestamp", "Event", "Hook", "Decision"
    );
    for record in records {
        println!(
            "{:<25} {:<12} {:<24} {:<10}",
            record
                .timestamp
                .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            format_core_event(&record.event),
            record.hook_id,
            format_core_outcome(&record.decision.decision),
        );
        if let Some(message) = &record.decision.message {
            println!("    Message: {}", message);
        }
    }
}

fn exec_record_to_json(record: &HookExecutionRecord) -> ExecLogJsonRecord {
    ExecLogJsonRecord {
        time: record
            .timestamp
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        event: format_core_event(&record.event),
        hook: record.hook_id.clone(),
        decision: format_core_outcome(&record.decision.decision),
        message: record.decision.message.clone(),
    }
}

fn render_validation(snapshot: &HookRegistrySnapshot, status: &HookValidationStatus) {
    println!("Validation status: {}", status_label(status).to_uppercase());
    if snapshot.layers.is_empty() {
        println!("No hook layers evaluated.");
        return;
    }
    for layer in &snapshot.layers {
        println!(
            "- {} ({}) → hooks: {}",
            format_proto_scope(&layer.scope),
            layer.path.display(),
            layer.loaded_hooks,
        );
        for skipped in &layer.skipped_hooks {
            println!(
                "    skipped {}: {}",
                format_proto_skip_reason(&skipped.reason),
                skipped
                    .hook_id
                    .clone()
                    .unwrap_or_else(|| "(unknown)".to_string())
            );
        }
    }
}

fn validation_summary(
    registry: &HookRegistry,
    scope: Option<ScopeFilterOption>,
) -> (HookValidationStatus, Vec<ValidateLayerJson>) {
    let mut status = HookValidationStatus::Ok;
    let mut layers = Vec::new();

    for layer in &registry.source_layers {
        if !scope_matches_core(scope, &layer.scope) {
            continue;
        }

        let mut layer_status = HookValidationStatus::Ok;
        let skipped: Vec<_> = layer
            .skipped_hooks
            .iter()
            .map(|skipped| {
                let severity = match skipped.reason {
                    CoreHookSkipReason::InvalidSchema
                    | CoreHookSkipReason::MissingExecutable
                    | CoreHookSkipReason::UnsupportedVersion => HookValidationStatus::Error,
                    CoreHookSkipReason::DuplicateId | CoreHookSkipReason::InvalidMatcher => {
                        HookValidationStatus::Warning
                    }
                };
                if severity == HookValidationStatus::Error {
                    layer_status = HookValidationStatus::Error;
                } else if layer_status == HookValidationStatus::Ok {
                    layer_status = HookValidationStatus::Warning;
                }

                SkippedHookJson {
                    hook_id: skipped.hook_id.clone(),
                    reason: format_core_skip_reason(&skipped.reason),
                    details: skipped.details.clone(),
                }
            })
            .collect();

        if matches!(layer_status, HookValidationStatus::Error) {
            status = HookValidationStatus::Error;
        } else if matches!(layer_status, HookValidationStatus::Warning)
            && !matches!(status, HookValidationStatus::Error)
        {
            status = HookValidationStatus::Warning;
        }

        layers.push(ValidateLayerJson {
            scope: format_core_scope(&layer.scope),
            path: layer.path.display().to_string(),
            loaded_hooks: layer.loaded_hooks,
            checksum: layer.checksum.clone(),
            skipped_hooks: skipped,
        });
    }

    (status, layers)
}

fn scope_matches_core(filter: Option<ScopeFilterOption>, scope: &HookScope) -> bool {
    match filter {
        None => true,
        Some(ScopeFilterOption::Managed) => matches!(scope, HookScope::ManagedPolicy { .. }),
        Some(ScopeFilterOption::Project) => matches!(scope, HookScope::Project { .. }),
        Some(ScopeFilterOption::Local) => matches!(scope, HookScope::LocalUser { .. }),
    }
}

fn format_proto_scope(scope: &ProtoHookScope) -> String {
    match scope {
        ProtoHookScope::ManagedPolicy { .. } => "managed".to_string(),
        ProtoHookScope::Project { .. } => "project".to_string(),
        ProtoHookScope::LocalUser { .. } => "local".to_string(),
    }
}

fn format_core_scope(scope: &HookScope) -> String {
    match scope {
        HookScope::ManagedPolicy { .. } => "managed".to_string(),
        HookScope::Project { .. } => "project".to_string(),
        HookScope::LocalUser { .. } => "local".to_string(),
    }
}

fn format_proto_skip_reason(reason: &ProtoHookSkipReason) -> String {
    format!("{:?}", reason)
}

fn format_core_skip_reason(reason: &CoreHookSkipReason) -> String {
    format!("{:?}", reason)
}

fn format_proto_event(event: &ProtoHookEvent) -> String {
    format!("{:?}", event)
}

fn format_core_event(event: &CoreHookEvent) -> String {
    format!("{:?}", event)
}

fn format_core_outcome(outcome: &codex_core::hooks::HookOutcome) -> String {
    format!("{:?}", outcome)
}

fn status_label(status: &HookValidationStatus) -> &'static str {
    match status {
        HookValidationStatus::Ok => "ok",
        HookValidationStatus::Warning => "warning",
        HookValidationStatus::Error => "error",
    }
}

fn hooks_log_path(config: &Config) -> PathBuf {
    config.codex_home.join("logs").join("hooks.jsonl")
}

fn read_exec_log(path: &PathBuf) -> Result<Vec<HookExecutionRecord>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(path).with_context(|| format!("Unable to open {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let record: HookExecutionRecord = serde_json::from_str(&line)?;
        records.push(record);
    }
    Ok(records)
}

fn parse_datetime(value: &str) -> Result<DateTime<Utc>> {
    let parsed = DateTime::parse_from_rfc3339(value)?;
    Ok(parsed.with_timezone(&Utc))
}
