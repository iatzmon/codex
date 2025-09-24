use clap::{Parser, Subcommand, ValueEnum};
use codex_common::CliConfigOverrides;
use codex_core::config::{Config, ConfigOverrides};
use codex_core::subagents::{
    SubagentDefinition, SubagentInventory, SubagentInvocationError, SubagentScope,
    invocation::InvocationSession,
    record::{SubagentRecord, SubagentStatus},
};
use codex_core::{AuthManager, ConversationManager};
use serde::Serialize;
use std::path::PathBuf;
use tracing::info;

/// Entry point for the `codex agents` command family.
#[derive(Debug, Parser)]
pub struct AgentsCli {
    #[clap(skip)]
    pub config_overrides: CliConfigOverrides,

    #[clap(subcommand)]
    command: AgentsCommand,
}

#[derive(Debug, Subcommand)]
enum AgentsCommand {
    /// List available subagents.
    List(AgentsListCommand),

    /// Invoke a subagent directly.
    Run(AgentsRunCommand),

    /// Show details about a specific subagent.
    Show(AgentsShowCommand),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ScopeFilter {
    Project,
    User,
    All,
}

impl Default for ScopeFilter {
    fn default() -> Self {
        ScopeFilter::All
    }
}

#[derive(Debug, Parser)]
struct AgentsListCommand {
    /// Filter agents by scope.
    #[arg(long = "scope", value_enum, default_value_t = ScopeFilter::All)]
    scope: ScopeFilter,

    /// Include invalid definitions that failed validation.
    #[arg(long = "invalid")]
    show_invalid: bool,

    /// Emit JSON instead of plaintext output.
    #[arg(long = "json")]
    json: bool,
}

#[derive(Debug, Parser)]
struct AgentsRunCommand {
    /// Name of the subagent to invoke (kebab-case).
    #[arg(value_name = "NAME")]
    name: String,

    /// Restrict the invocation to specific tools (repeatable).
    #[arg(long = "tool", value_name = "TOOL")]
    tool: Vec<String>,

    /// Emit JSON instead of plaintext output.
    #[arg(long = "json")]
    json: bool,
}

#[derive(Debug, Parser)]
struct AgentsShowCommand {
    /// Name of the subagent to inspect.
    #[arg(value_name = "NAME")]
    name: String,

    /// Emit JSON instead of plaintext output.
    #[arg(long = "json")]
    json: bool,
}

pub async fn run_agents_cli(cli: AgentsCli, cwd_override: Option<PathBuf>) -> anyhow::Result<()> {
    let overrides = cli.config_overrides.clone();
    let config = load_config(&overrides, cwd_override)?;
    let manager = ConversationManager::new(AuthManager::shared(config.codex_home.clone()));

    match cli.command {
        AgentsCommand::List(cmd) => run_list(cmd, &config, &manager).await?,
        AgentsCommand::Run(cmd) => run_run(cmd, &config, &manager).await?,
        AgentsCommand::Show(cmd) => run_show(cmd, &config, &manager).await?,
    }

    Ok(())
}

fn load_config(
    overrides: &CliConfigOverrides,
    cwd_override: Option<PathBuf>,
) -> anyhow::Result<Config> {
    let pairs = overrides.parse_overrides().map_err(anyhow::Error::msg)?;
    Ok(Config::load_with_cli_overrides(
        pairs,
        ConfigOverrides {
            cwd: cwd_override,
            ..Default::default()
        },
    )?)
}

async fn run_list(
    cmd: AgentsListCommand,
    config: &Config,
    manager: &ConversationManager,
) -> anyhow::Result<()> {
    let inventory = manager.subagent_inventory(config);

    if !config.subagents.is_enabled() {
        info!("subagents feature disabled via configuration");
    }

    if cmd.json {
        let payload = build_list_payload(&inventory, cmd.scope, cmd.show_invalid);
        serde_json::to_writer_pretty(std::io::stdout(), &payload)?;
        println!();
    } else {
        print_list(&inventory, cmd.scope, cmd.show_invalid);
    }

    Ok(())
}

async fn run_run(
    cmd: AgentsRunCommand,
    config: &Config,
    manager: &ConversationManager,
) -> anyhow::Result<()> {
    let normalized_name = SubagentDefinition::normalize_name(&cmd.name);
    let mut session = InvocationSession::new(normalized_name).confirmed();
    if !cmd.tool.is_empty() {
        session.requested_tools = cmd.tool.clone();
    }

    let result = manager.invoke_subagent(config, session).await;
    match result {
        Ok(response) => {
            if cmd.json {
                serde_json::to_writer_pretty(std::io::stdout(), &RunOutput::from(&response))?;
                println!();
            } else {
                println!("Subagent: {}", response.subagent_name);
                if let Some(model) = response.resolved_model.as_ref() {
                    println!("Model: {model}");
                }
                if !response.requested_tools.is_empty() {
                    println!("Tools: {}", response.requested_tools.join(", "));
                }
                if let Some(summary) = response.summary.as_ref() {
                    println!("Summary: {summary}");
                }
                if !response.detail_artifacts.is_empty() {
                    println!("Detail: {}", response.detail_artifacts[0].display());
                }
            }
        }
        Err(err) => {
            if cmd.json {
                serde_json::to_writer_pretty(std::io::stdout(), &ErrorOutput::from(&err))?;
                println!();
            } else {
                eprintln!("Error: {err}");
            }
            return Err(anyhow::Error::new(err));
        }
    }

    Ok(())
}

async fn run_show(
    cmd: AgentsShowCommand,
    config: &Config,
    manager: &ConversationManager,
) -> anyhow::Result<()> {
    let inventory = manager.subagent_inventory(config);
    let key = SubagentDefinition::normalize_name(&cmd.name);

    let record = inventory.subagents.get(&key).or_else(|| {
        inventory
            .invalid()
            .into_iter()
            .find(|record| record.definition.name == key || record.definition.raw_name == cmd.name)
    });

    let Some(record) = record else {
        let err = anyhow::anyhow!("No subagent named '{}'", cmd.name);
        if cmd.json {
            serde_json::to_writer_pretty(
                std::io::stdout(),
                &serde_json::json!({ "error": err.to_string() }),
            )?;
            println!();
        } else {
            eprintln!(
                "{}
",
                err
            );
        }
        return Err(err);
    };

    if cmd.json {
        serde_json::to_writer_pretty(std::io::stdout(), &RecordOutput::from(record))?;
        println!();
    } else {
        print_record(record);
    }

    Ok(())
}

#[derive(Serialize)]
struct ListPayload {
    subagents: Vec<RecordOutput>,
    invalid: Vec<RecordOutput>,
}

fn build_list_payload(
    inventory: &SubagentInventory,
    scope: ScopeFilter,
    include_invalid: bool,
) -> ListPayload {
    let mut subagents = Vec::new();
    for record in inventory.subagents.values() {
        if !scope_matches(scope, record.definition.scope) {
            continue;
        }
        subagents.push(record.into());
    }

    let mut invalid = Vec::new();
    if include_invalid {
        for record in inventory.invalid() {
            if !scope_matches(scope, record.definition.scope) {
                continue;
            }
            invalid.push(record.into());
        }
    }

    ListPayload { subagents, invalid }
}

fn print_list(inventory: &SubagentInventory, scope: ScopeFilter, include_invalid: bool) {
    if inventory.subagents.is_empty() && (!include_invalid || inventory.invalid().is_empty()) {
        println!("No subagents found.");
        return;
    }

    println!("Available subagents:");
    for (name, record) in &inventory.subagents {
        if !scope_matches(scope, record.definition.scope) {
            continue;
        }
        println!("  - {} ({})", name, display_scope(record.definition.scope));
        println!("    Description: {}", record.definition.description);
        if !record.effective_tools.is_empty() {
            println!("    Tools: {}", record.effective_tools.join(", "));
        }
        if let Some(model) = record.effective_model.as_ref() {
            println!("    Model: {model}");
        }
        println!("    Source: {}", record.definition.source_path.display());
        if record.status != SubagentStatus::Active {
            println!("    Status: {:?}", record.status);
        }
    }

    if include_invalid {
        let invalid_records: Vec<_> = inventory
            .invalid()
            .into_iter()
            .filter(|record| scope_matches(scope, record.definition.scope))
            .collect();
        if !invalid_records.is_empty() {
            println!("\nInvalid definitions:");
            for record in invalid_records {
                println!(
                    "  - {} ({})",
                    record.definition.name,
                    display_scope(record.definition.scope)
                );
                println!("    Errors: {}", record.validation_errors.join("; "));
                println!("    Source: {}", record.definition.source_path.display());
            }
        }
    }
}

fn print_record(record: &SubagentRecord) {
    println!("Name: {}", record.definition.name);
    println!("Scope: {}", display_scope(record.definition.scope));
    println!("Description: {}", record.definition.description);
    if !record.effective_tools.is_empty() {
        println!("Tools: {}", record.effective_tools.join(", "));
    }
    if let Some(model) = record.effective_model.as_ref() {
        println!("Model: {model}");
    }
    println!("Source: {}", record.definition.source_path.display());
    println!("Status: {:?}", record.status);
    if !record.validation_errors.is_empty() {
        println!("Validation errors: {}", record.validation_errors.join("; "));
    }
}

fn scope_matches(filter: ScopeFilter, scope: SubagentScope) -> bool {
    match filter {
        ScopeFilter::All => true,
        ScopeFilter::Project => matches!(scope, SubagentScope::Project),
        ScopeFilter::User => matches!(scope, SubagentScope::User),
    }
}

fn display_scope(scope: SubagentScope) -> &'static str {
    match scope {
        SubagentScope::Project => "project",
        SubagentScope::User => "user",
    }
}

#[derive(Serialize)]
struct RecordOutput {
    name: String,
    scope: &'static str,
    description: String,
    tools: Vec<String>,
    model: Option<String>,
    status: String,
    source_path: String,
    validation_errors: Vec<String>,
}

impl From<&SubagentRecord> for RecordOutput {
    fn from(record: &SubagentRecord) -> Self {
        Self {
            name: record.definition.name.clone(),
            scope: display_scope(record.definition.scope),
            description: record.definition.description.clone(),
            tools: record.effective_tools.clone(),
            model: record.effective_model.clone(),
            status: format!("{:?}", record.status),
            source_path: record.definition.source_path.display().to_string(),
            validation_errors: record.validation_errors.clone(),
        }
    }
}

#[derive(Serialize)]
struct RunOutput {
    name: String,
    summary: Option<String>,
    model: Option<String>,
    tools: Vec<String>,
    detail_artifacts: Vec<String>,
}

impl From<&InvocationSession> for RunOutput {
    fn from(session: &InvocationSession) -> Self {
        Self {
            name: session.subagent_name.clone(),
            summary: session.summary.clone(),
            model: session.resolved_model.clone(),
            tools: session.requested_tools.clone(),
            detail_artifacts: session
                .detail_artifacts
                .iter()
                .map(|path| path.display().to_string())
                .collect(),
        }
    }
}

#[derive(Serialize)]
struct ErrorOutput {
    error: String,
}

impl From<&SubagentInvocationError> for ErrorOutput {
    fn from(err: &SubagentInvocationError) -> Self {
        Self {
            error: err.to_string(),
        }
    }
}
