use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::atomic::AtomicU64;
use std::time::Duration;
use uuid::Uuid;

use crate::AuthManager;
use crate::client_common::REVIEW_PROMPT;
use crate::event_mapping::map_response_item_to_event_messages;
use async_channel::Receiver;
use async_channel::Sender;
use codex_apply_patch::ApplyPatchAction;
use codex_apply_patch::MaybeApplyPatchVerified;
use codex_apply_patch::maybe_parse_apply_patch_verified;
use codex_protocol::mcp_protocol::ConversationId;
use codex_protocol::protocol::ConversationPathResponseEvent;
use codex_protocol::protocol::ExitedReviewModeEvent;
use codex_protocol::protocol::ReviewRequest;
use codex_protocol::protocol::RolloutItem;
use codex_protocol::protocol::TaskStartedEvent;
use codex_protocol::protocol::TurnAbortReason;
use codex_protocol::protocol::TurnAbortedEvent;
use codex_protocol::protocol::TurnContextItem;
use futures::prelude::*;
use mcp_types::CallToolResult;
use serde::Deserialize;
use serde::Serialize;
use serde_json;
use tokio::sync::oneshot;
use tokio::task::AbortHandle;
use tracing::debug;
use tracing::error;
use tracing::info;
use tracing::trace;
use tracing::warn;

use crate::ModelProviderInfo;
use crate::apply_patch;
use crate::apply_patch::ApplyPatchExec;
use crate::apply_patch::CODEX_APPLY_PATCH_ARG1;
use crate::apply_patch::InternalApplyPatchInvocation;
use crate::apply_patch::convert_apply_patch_to_protocol;
use crate::client::ModelClient;
use crate::client_common::Prompt;
use crate::client_common::ResponseEvent;
use crate::config::Config;
use crate::config_types::ShellEnvironmentPolicy;
use crate::conversation_history::ConversationHistory;
use crate::environment_context::EnvironmentContext;
use crate::error::CodexErr;
use crate::error::Result as CodexResult;
use crate::error::SandboxErr;
use crate::error::get_error_message_ui;
use crate::exec::ExecParams;
use crate::exec::ExecToolCallOutput;
use crate::exec::SandboxType;
use crate::exec::StdoutStream;
use crate::exec::StreamOutput;
use crate::exec::process_exec_tool_call;
use crate::exec_command::EXEC_COMMAND_TOOL_NAME;
use crate::exec_command::ExecCommandParams;
use crate::exec_command::ExecSessionManager;
use crate::exec_command::WRITE_STDIN_TOOL_NAME;
use crate::exec_command::WriteStdinParams;
use crate::exec_env::create_env;
use crate::hooks::executor::{HookExecutor, PreToolUsePayload};
use crate::hooks::snapshot::build_hook_registry_snapshot;
use crate::hooks::{HookDecision, HookLogWriter, HookOutcome, HookScope};
use crate::mcp_connection_manager::MCP_TOOL_NAME_DELIMITER;
use crate::mcp_connection_manager::McpConnectionManager;
use crate::mcp_tool_call::handle_mcp_tool_call;
use crate::model_family::find_family_for_model;
use crate::openai_model_info::get_model_info;
use crate::openai_tools::ApplyPatchToolArgs;
use crate::openai_tools::SubagentToolRegistration;
use crate::openai_tools::ToolsConfig;
use crate::openai_tools::ToolsConfigParams;
use crate::openai_tools::get_openai_tools;
use crate::parse_command::parse_command;
use crate::plan_mode::PLAN_MODE_SYSTEM_PROMPT;
use crate::plan_mode::PlanArtifact;
use crate::plan_mode::PlanEntryType;
use crate::plan_mode::PlanModeEvent;
use crate::plan_mode::PlanModeSession;
use crate::plan_mode::PlanTelemetry;
use crate::plan_mode::ToolCapability;
use crate::plan_mode::ToolMode;
use crate::plan_tool::StepStatus;
use crate::plan_tool::UpdatePlanArgs;
use crate::plan_tool::handle_update_plan;
use crate::project_doc::get_user_instructions;
use crate::protocol::AgentMessageDeltaEvent;
use crate::protocol::AgentMessageEvent;
use crate::protocol::AgentReasoningDeltaEvent;
use crate::protocol::AgentReasoningRawContentDeltaEvent;
use crate::protocol::AgentReasoningSectionBreakEvent;
use crate::protocol::ApplyPatchApprovalRequestEvent;
use crate::protocol::AskForApproval;
use crate::protocol::BackgroundEventEvent;
use crate::protocol::ErrorEvent;
use crate::protocol::Event;
use crate::protocol::EventMsg;
use crate::protocol::ExecApprovalRequestEvent;
use crate::protocol::ExecCommandBeginEvent;
use crate::protocol::ExecCommandEndEvent;
use crate::protocol::FileChange;
use crate::protocol::HookExecLogResponseEvent;
use crate::protocol::HookRegistrySnapshotEvent;
use crate::protocol::HookReloadResultEvent;
use crate::protocol::HookValidationResultEvent;
use crate::protocol::InputItem;
use crate::protocol::ListCustomPromptsResponseEvent;
use crate::protocol::Op;
use crate::protocol::PatchApplyBeginEvent;
use crate::protocol::PatchApplyEndEvent;
use crate::protocol::ReviewDecision;
use crate::protocol::ReviewOutputEvent;
use crate::protocol::SandboxPolicy;
use crate::protocol::SessionConfiguredEvent;
use crate::protocol::StreamErrorEvent;
use crate::protocol::SubagentApprovalDecision;
use crate::protocol::SubagentApprovalRequestEvent;
use crate::protocol::Submission;
use crate::protocol::TaskCompleteEvent;
use crate::protocol::TokenUsage;
use crate::protocol::TokenUsageInfo;
use crate::protocol::TurnDiffEvent;
use crate::protocol::WebSearchBeginEvent;
use crate::rollout::RolloutRecorder;
use crate::rollout::RolloutRecorderParams;
use crate::safety::SafetyCheck;
use crate::safety::assess_command_safety;
use crate::safety::assess_safety_for_untrusted_command;
use crate::shell;
#[cfg(feature = "slash_commands")]
use crate::slash_commands::CommandInvocation;
#[cfg(feature = "slash_commands")]
use crate::slash_commands::InvocationError;
#[cfg(feature = "slash_commands")]
use crate::slash_commands::SlashCommandService;
use crate::subagents::InvocationSession;
use crate::subagents::SubagentConfig;
use crate::subagents::SubagentDefinition;
use crate::subagents::SubagentInventory;
use crate::subagents::SubagentInvocationError;
use crate::subagents::SubagentRunner;
use crate::subagents::build_inventory_for_config;
use crate::subagents::config::SubagentDiscoveryMode;
use crate::subagents::execute_subagent_invocation;
use crate::turn_diff_tracker::TurnDiffTracker;
use crate::unified_exec::UnifiedExecSessionManager;
use crate::user_instructions::UserInstructions;
use crate::user_notification::UserNotification;
use crate::util::backoff;
use codex_protocol::config_types::ReasoningEffort as ReasoningEffortConfig;
use codex_protocol::config_types::ReasoningSummary as ReasoningSummaryConfig;
use codex_protocol::custom_prompts::CustomPrompt;
use codex_protocol::hooks::{
    HookExecLogResponse, HookReloadResponse, HookValidationStatus, HookValidationSummary,
};
use codex_protocol::models::ContentItem;
use codex_protocol::models::FunctionCallOutputPayload;
use codex_protocol::models::LocalShellAction;
use codex_protocol::models::ResponseInputItem;
use codex_protocol::models::ResponseItem;
use codex_protocol::models::ShellToolCallParams;
use codex_protocol::plan_mode::PlanModeActivatedEvent;
use codex_protocol::plan_mode::PlanModeAppliedEvent;
use codex_protocol::plan_mode::PlanModeExitedEvent;
use codex_protocol::plan_mode::PlanModeSessionPayload;
use codex_protocol::plan_mode::PlanModeUpdatedEvent;
use codex_protocol::protocol::InitialHistory;

mod compact;
use self::compact::build_compacted_history;
use self::compact::collect_user_messages;

// A convenience extension trait for acquiring mutex locks where poisoning is
// unrecoverable and should abort the program. This avoids scattered `.unwrap()`
// calls on `lock()` while still surfacing a clear panic message when a lock is
// poisoned.
trait MutexExt<T> {
    fn lock_unchecked(&self) -> MutexGuard<'_, T>;
}

impl<T> MutexExt<T> for Mutex<T> {
    fn lock_unchecked(&self) -> MutexGuard<'_, T> {
        #[expect(clippy::expect_used)]
        self.lock().expect("poisoned lock")
    }
}

/// The high-level interface to the Codex system.
/// It operates as a queue pair where you send submissions and receive events.
pub struct Codex {
    next_id: AtomicU64,
    tx_sub: Sender<Submission>,
    rx_event: Receiver<Event>,
}

/// Wrapper returned by [`Codex::spawn`] containing the spawned [`Codex`],
/// the submission id for the initial `ConfigureSession` request and the
/// unique session id.
pub struct CodexSpawnOk {
    pub codex: Codex,
    pub conversation_id: ConversationId,
    pub(crate) session: Arc<Session>,
}

pub(crate) const INITIAL_SUBMIT_ID: &str = "";
pub(crate) const SUBMISSION_CHANNEL_CAPACITY: usize = 64;

// Model-formatting limits: clients get full streams; oonly content sent to the model is truncated.
pub(crate) const MODEL_FORMAT_MAX_BYTES: usize = 10 * 1024; // 10 KiB
pub(crate) const MODEL_FORMAT_MAX_LINES: usize = 256; // lines
pub(crate) const MODEL_FORMAT_HEAD_LINES: usize = MODEL_FORMAT_MAX_LINES / 2;
pub(crate) const MODEL_FORMAT_TAIL_LINES: usize = MODEL_FORMAT_MAX_LINES - MODEL_FORMAT_HEAD_LINES; // 128
pub(crate) const MODEL_FORMAT_HEAD_BYTES: usize = MODEL_FORMAT_MAX_BYTES / 2;
const PLAN_IMPLEMENTATION_TRIGGER_TEXT: &str = "(auto) Plan approved. Begin implementing the approved plan now by following the recorded steps, executing safe commands, and reporting progress as you go.";

impl Codex {
    /// Spawn a new [`Codex`] and initialize the session.
    pub async fn spawn(
        config: Config,
        auth_manager: Arc<AuthManager>,
        conversation_history: InitialHistory,
    ) -> CodexResult<CodexSpawnOk> {
        let (tx_sub, rx_sub) = async_channel::bounded(SUBMISSION_CHANNEL_CAPACITY);
        let (tx_event, rx_event) = async_channel::unbounded();

        let user_instructions = get_user_instructions(&config).await;

        let config = Arc::new(config);

        let configure_session = ConfigureSession {
            provider: config.model_provider.clone(),
            model: config.model.clone(),
            model_reasoning_effort: config.model_reasoning_effort,
            model_reasoning_summary: config.model_reasoning_summary,
            user_instructions,
            base_instructions: config.base_instructions.clone(),
            approval_policy: config.approval_policy,
            sandbox_policy: config.sandbox_policy.clone(),
            notify: config.notify.clone(),
            cwd: config.cwd.clone(),
        };

        // Generate a unique ID for the lifetime of this Codex session.
        let (session, turn_context) = Session::new(
            configure_session,
            config.clone(),
            auth_manager.clone(),
            tx_event.clone(),
            conversation_history,
        )
        .await
        .map_err(|e| {
            error!("Failed to create session: {e:#}");
            CodexErr::InternalAgentDied
        })?;
        let conversation_id = session.conversation_id;

        // This task will run until Op::Shutdown is received.
        let submission_session = Arc::clone(&session);
        tokio::spawn(submission_loop(
            submission_session,
            turn_context,
            config,
            rx_sub,
        ));
        let codex = Codex {
            next_id: AtomicU64::new(0),
            tx_sub,
            rx_event,
        };

        Ok(CodexSpawnOk {
            codex,
            conversation_id,
            session,
        })
    }

    /// Submit the `op` wrapped in a `Submission` with a unique ID.
    pub async fn submit(&self, op: Op) -> CodexResult<String> {
        let id = self
            .next_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
            .to_string();
        let sub = Submission { id: id.clone(), op };
        self.submit_with_id(sub).await?;
        Ok(id)
    }

    /// Use sparingly: prefer `submit()` so Codex is responsible for generating
    /// unique IDs for each submission.
    pub async fn submit_with_id(&self, sub: Submission) -> CodexResult<()> {
        self.tx_sub
            .send(sub)
            .await
            .map_err(|_| CodexErr::InternalAgentDied)?;
        Ok(())
    }

    pub async fn next_event(&self) -> CodexResult<Event> {
        let event = self
            .rx_event
            .recv()
            .await
            .map_err(|_| CodexErr::InternalAgentDied)?;
        Ok(event)
    }
}

/// Mutable state of the agent
#[derive(Default)]
struct State {
    approved_commands: HashSet<Vec<String>>,
    current_task: Option<AgentTask>,
    pending_approvals: HashMap<String, oneshot::Sender<ReviewDecision>>,
    pending_input: Vec<ResponseInputItem>,
    pending_subagent_approvals: HashMap<String, oneshot::Sender<SubagentApprovalDecision>>,
    history: ConversationHistory,
    token_info: Option<TokenUsageInfo>,
    next_internal_sub_id: u64,
    plan_mode: Option<PlanModeSession>,
    plan_mode_prompt_recorded: bool,
}

/// Context for an initialized model agent
///
/// A session has at most 1 running task at a time, and can be interrupted by user input.
pub(crate) struct Session {
    conversation_id: ConversationId,
    tx_event: Sender<Event>,

    /// Manager for external MCP servers/tools.
    mcp_connection_manager: McpConnectionManager,
    session_manager: ExecSessionManager,
    unified_exec_manager: UnifiedExecSessionManager,

    /// External notifier command (will be passed as args to exec()). When
    /// `None` this feature is disabled.
    notify: Option<Vec<String>>,

    /// Hook executor coordinating lifecycle hook invocations.
    hook_executor: HookExecutor,

    /// Optional rollout recorder for persisting the conversation transcript so
    /// sessions can be replayed or inspected later.
    rollout: Mutex<Option<RolloutRecorder>>,
    state: Mutex<State>,
    #[cfg(feature = "slash_commands")]
    slash_commands: Option<SlashCommandService>,
    codex_linux_sandbox_exe: Option<PathBuf>,
    user_shell: shell::Shell,
    show_raw_agent_reasoning: bool,
}

/// The context needed for a single turn of the conversation.
#[derive(Debug)]
pub(crate) struct TurnContext {
    pub(crate) client: ModelClient,
    /// The session's current working directory. All relative paths provided by
    /// the model as well as sandbox policies are resolved against this path
    /// instead of `std::env::current_dir()`.
    pub(crate) cwd: PathBuf,
    pub(crate) base_instructions: Option<String>,
    pub(crate) user_instructions: Option<String>,
    pub(crate) approval_policy: AskForApproval,
    pub(crate) sandbox_policy: SandboxPolicy,
    pub(crate) shell_environment_policy: ShellEnvironmentPolicy,
    pub(crate) tools_config: ToolsConfig,
    pub(crate) is_review_mode: bool,
    pub(crate) subagent_inventory: Option<Arc<SubagentInventory>>,
    pub(crate) subagent_tool: Option<SubagentToolRegistration>,
    pub(crate) subagent_config: Option<SubagentConfig>,
}

impl TurnContext {
    fn resolve_path(&self, path: Option<String>) -> PathBuf {
        path.as_ref()
            .map(PathBuf::from)
            .map_or_else(|| self.cwd.clone(), |p| self.cwd.join(p))
    }
}

fn compute_subagent_tooling(
    config: &Config,
) -> (
    Option<Arc<SubagentInventory>>,
    Option<SubagentToolRegistration>,
    Option<SubagentConfig>,
) {
    if !config.subagents.is_enabled() {
        return (None, None, None);
    }

    let inventory = build_inventory_for_config(config);
    let registration = SubagentToolRegistration::from_inventory(&config.subagents, &inventory);
    let subagent_config = Some(config.subagents.clone());

    (Some(Arc::new(inventory)), registration, subagent_config)
}

const SUBAGENT_GUIDANCE_START: &str = "--- subagent-guidance ---";
const SUBAGENT_GUIDANCE_END: &str = "--- end-subagent-guidance ---";

fn merge_subagent_user_instructions(
    existing: Option<String>,
    inventory: Option<&SubagentInventory>,
) -> Option<String> {
    let Some(inventory) = inventory else {
        return existing;
    };
    let Some(body) = build_subagent_guidance(inventory) else {
        return existing;
    };

    let block = format!("{SUBAGENT_GUIDANCE_START}\n{body}\n{SUBAGENT_GUIDANCE_END}");

    match existing {
        Some(mut text) => {
            if let Some(start) = text.find(SUBAGENT_GUIDANCE_START) {
                let suffix_search = &text[start..];
                if let Some(end_rel) = suffix_search.find(SUBAGENT_GUIDANCE_END) {
                    let end = start + end_rel + SUBAGENT_GUIDANCE_END.len();
                    text.replace_range(start..end, &block);
                } else {
                    text.replace_range(start.., &block);
                }
            } else {
                if !text.trim_end().is_empty() {
                    if !text.ends_with('\n') {
                        text.push('\n');
                    }
                    text.push('\n');
                }
                text.push_str(&block);
            }
            Some(text)
        }
        None => Some(block),
    }
}

fn build_subagent_guidance(inventory: &SubagentInventory) -> Option<String> {
    if inventory.subagents.is_empty() {
        return None;
    }

    let mut lines = Vec::new();
    lines.push("Subagent delegation guidance:".to_string());
    lines.push(
        "- Read available subagent descriptions each turn and follow any directives they contain."
            .to_string(),
    );
    lines.push("- When a description says to proactively handle the current request, name the subagent, ask the user for approval, invoke it with `invoke_subagent` (retry with `confirmed: true` after approval), and relay the subagent's report before adding your own analysis. Continue solo only if the user declines.".to_string());
    lines.push("- Delegate only when the user's request matches the subagent's remit.".to_string());
    lines.push("Available subagents:".to_string());

    for record in inventory.subagents.values() {
        let name = &record.definition.name;
        let description = record.definition.description.trim();
        if description.is_empty() {
            lines.push(format!("  - {name}"));
        } else {
            lines.push(format!("  - {name}: {description}"));
        }
    }

    Some(lines.join("\n"))
}

/// Configure the model session.
struct ConfigureSession {
    /// Provider identifier ("openai", "openrouter", ...).
    provider: ModelProviderInfo,

    /// If not specified, server will use its default model.
    model: String,

    model_reasoning_effort: Option<ReasoningEffortConfig>,
    model_reasoning_summary: ReasoningSummaryConfig,

    /// Model instructions that are appended to the base instructions.
    user_instructions: Option<String>,

    /// Base instructions override.
    base_instructions: Option<String>,

    /// When to escalate for approval for execution
    approval_policy: AskForApproval,
    /// How to sandbox commands executed in the system
    sandbox_policy: SandboxPolicy,

    /// Optional external notifier command tokens. Present only when the
    /// client wants the agent to spawn a program after each completed
    /// turn.
    notify: Option<Vec<String>>,

    /// Working directory that should be treated as the *root* of the
    /// session. All relative paths supplied by the model as well as the
    /// execution sandbox are resolved against this directory **instead**
    /// of the process-wide current working directory. CLI front-ends are
    /// expected to expand this to an absolute path before sending the
    /// `ConfigureSession` operation so that the business-logic layer can
    /// operate deterministically.
    cwd: PathBuf,
}

#[cfg(feature = "slash_commands")]
async fn handle_slash_command_turn(
    sess: Arc<Session>,
    base_context: Arc<TurnContext>,
    config: Arc<Config>,
    sub_id: String,
    model_override: String,
    items: Vec<InputItem>,
) {
    let provider = base_context.client.get_provider();
    let auth_manager = base_context.client.get_auth_manager();
    let model_family =
        find_family_for_model(&model_override).unwrap_or_else(|| config.model_family.clone());

    let mut per_turn_config = (*config).clone();
    per_turn_config.model = model_override;
    per_turn_config.model_family = model_family.clone();
    if let Some(model_info) = get_model_info(&model_family) {
        per_turn_config.model_context_window = Some(model_info.context_window);
    }

    let per_turn_config = Arc::new(per_turn_config);
    let client = ModelClient::new(
        per_turn_config.clone(),
        auth_manager,
        provider,
        base_context.client.get_reasoning_effort(),
        base_context.client.get_reasoning_summary(),
        sess.conversation_id,
    );

    let tools_config = ToolsConfig::new(&ToolsConfigParams {
        model_family: &model_family,
        approval_policy: base_context.approval_policy,
        sandbox_policy: base_context.sandbox_policy.clone(),
        include_plan_tool: config.include_plan_tool,
        include_apply_patch_tool: config.include_apply_patch_tool,
        include_web_search_request: config.tools_web_search_request,
        use_streamable_shell_tool: config.use_experimental_streamable_shell_tool,
        include_view_image_tool: config.include_view_image_tool,
        experimental_unified_exec_tool: config.use_experimental_unified_exec_tool,
    });

    let (subagent_inventory, subagent_tool, subagent_config) =
        compute_subagent_tooling(per_turn_config.as_ref());

    let user_instructions = merge_subagent_user_instructions(
        base_context.user_instructions.clone(),
        subagent_inventory.as_deref(),
    );

    let turn_context = TurnContext {
        client,
        tools_config,
        user_instructions,
        base_instructions: base_context.base_instructions.clone(),
        approval_policy: base_context.approval_policy,
        sandbox_policy: base_context.sandbox_policy.clone(),
        shell_environment_policy: base_context.shell_environment_policy.clone(),
        cwd: base_context.cwd.clone(),
        is_review_mode: base_context.is_review_mode,
        subagent_inventory,
        subagent_tool,
        subagent_config,
    };

    let task = AgentTask::spawn(sess.clone(), Arc::new(turn_context), sub_id, items);
    sess.set_task(task);
}

impl Session {
    async fn new(
        configure_session: ConfigureSession,
        config: Arc<Config>,
        auth_manager: Arc<AuthManager>,
        tx_event: Sender<Event>,
        initial_history: InitialHistory,
    ) -> anyhow::Result<(Arc<Self>, TurnContext)> {
        let ConfigureSession {
            provider,
            model,
            model_reasoning_effort,
            model_reasoning_summary,
            user_instructions,
            base_instructions,
            approval_policy,
            sandbox_policy,
            notify,
            cwd,
        } = configure_session;
        let (subagent_inventory, subagent_tool, subagent_config) =
            compute_subagent_tooling(config.as_ref());
        let user_instructions =
            merge_subagent_user_instructions(user_instructions, subagent_inventory.as_deref());

        debug!("Configuring session: model={model}; provider={provider:?}");
        if !cwd.is_absolute() {
            return Err(anyhow::anyhow!("cwd is not absolute: {cwd:?}"));
        }

        let (conversation_id, rollout_params) = match &initial_history {
            InitialHistory::New | InitialHistory::Forked(_) => {
                let conversation_id = ConversationId::default();
                (
                    conversation_id,
                    RolloutRecorderParams::new(conversation_id, user_instructions.clone()),
                )
            }
            InitialHistory::Resumed(resumed_history) => (
                resumed_history.conversation_id,
                RolloutRecorderParams::resume(resumed_history.rollout_path.clone()),
            ),
        };

        // Error messages to dispatch after SessionConfigured is sent.
        let mut post_session_configured_error_events = Vec::<Event>::new();

        // Kick off independent async setup tasks in parallel to reduce startup latency.
        //
        // - initialize RolloutRecorder with new or resumed session info
        // - spin up MCP connection manager
        // - perform default shell discovery
        // - load history metadata
        let rollout_fut = RolloutRecorder::new(&config, rollout_params);

        let mcp_fut = McpConnectionManager::new(config.mcp_servers.clone());
        let default_shell_fut = shell::default_user_shell();
        let history_meta_fut = crate::message_history::history_metadata(&config);

        // Join all independent futures.
        let (rollout_recorder, mcp_res, default_shell, (history_log_id, history_entry_count)) =
            tokio::join!(rollout_fut, mcp_fut, default_shell_fut, history_meta_fut);

        let rollout_recorder = rollout_recorder.map_err(|e| {
            error!("failed to initialize rollout recorder: {e:#}");
            anyhow::anyhow!("failed to initialize rollout recorder: {e:#}")
        })?;
        let rollout_path = rollout_recorder.rollout_path.clone();
        // Create the mutable state for the Session.
        let mut state = State {
            history: ConversationHistory::new(),
            ..Default::default()
        };

        #[cfg(feature = "slash_commands")]
        let slash_commands = match SlashCommandService::new(config.as_ref()).await {
            Ok(service) => Some(service),
            Err(err) => {
                let message = format!("Failed to load slash commands: {err}");
                error!("{message}");
                post_session_configured_error_events.push(Event {
                    id: INITIAL_SUBMIT_ID.to_owned(),
                    msg: EventMsg::Error(ErrorEvent { message }),
                });
                None
            }
        };

        // Handle MCP manager result and record any startup failures.
        let (mcp_connection_manager, failed_clients) = match mcp_res {
            Ok((mgr, failures)) => (mgr, failures),
            Err(e) => {
                let message = format!("Failed to create MCP connection manager: {e:#}");
                error!("{message}");
                post_session_configured_error_events.push(Event {
                    id: INITIAL_SUBMIT_ID.to_owned(),
                    msg: EventMsg::Error(ErrorEvent { message }),
                });
                (McpConnectionManager::default(), Default::default())
            }
        };

        if config.plan_mode.plan_enabled {
            state.plan_mode = Some(PlanModeSession::new(
                Uuid::from(conversation_id),
                approval_policy,
                Session::plan_mode_capabilities(&mcp_connection_manager),
                &config.plan_mode,
                config.sandbox_policy.has_full_network_access(),
            ));
            state.plan_mode_prompt_recorded = false;
        }

        // Surface individual client start-up failures to the user.
        if !failed_clients.is_empty() {
            for (server_name, err) in failed_clients {
                let message = format!("MCP client for `{server_name}` failed to start: {err:#}");
                error!("{message}");
                post_session_configured_error_events.push(Event {
                    id: INITIAL_SUBMIT_ID.to_owned(),
                    msg: EventMsg::Error(ErrorEvent { message }),
                });
            }
        }

        // Now that the conversation id is final (may have been updated by resume),
        // construct the model client.
        let client = ModelClient::new(
            config.clone(),
            Some(auth_manager.clone()),
            provider.clone(),
            model_reasoning_effort,
            model_reasoning_summary,
            conversation_id,
        );
        let mut turn_context = TurnContext {
            client,
            tools_config: ToolsConfig::new(&ToolsConfigParams {
                model_family: &config.model_family,
                approval_policy,
                sandbox_policy: sandbox_policy.clone(),
                include_plan_tool: config.include_plan_tool,
                include_apply_patch_tool: config.include_apply_patch_tool,
                include_web_search_request: config.tools_web_search_request,
                use_streamable_shell_tool: config.use_experimental_streamable_shell_tool,
                include_view_image_tool: config.include_view_image_tool,
                experimental_unified_exec_tool: config.use_experimental_unified_exec_tool,
            }),
            user_instructions,
            base_instructions,
            approval_policy,
            sandbox_policy,
            shell_environment_policy: config.shell_environment_policy.clone(),
            cwd,
            is_review_mode: false,
            subagent_inventory,
            subagent_tool,
            subagent_config,
        };

        if config.plan_mode.plan_enabled {
            turn_context.tools_config = ToolsConfig::new(&ToolsConfigParams {
                model_family: &config.model_family,
                approval_policy,
                sandbox_policy: turn_context.sandbox_policy.clone(),
                include_plan_tool: true,
                include_apply_patch_tool: false,
                include_web_search_request: config.tools_web_search_request,
                use_streamable_shell_tool: config.use_experimental_streamable_shell_tool,
                include_view_image_tool: config.include_view_image_tool,
                experimental_unified_exec_tool: config.use_experimental_unified_exec_tool,
            });
        }
        let hook_log_path = config.codex_home.join("logs").join("hooks.jsonl");
        let hook_executor = HookExecutor::with_runtime(
            config.hook_registry.clone(),
            HookLogWriter::new(hook_log_path),
            HookScope::LocalUser {
                codex_home: config.codex_home.clone(),
            },
        );

        let sess = Arc::new(Session {
            conversation_id,
            tx_event: tx_event.clone(),
            mcp_connection_manager,
            session_manager: ExecSessionManager::default(),
            unified_exec_manager: UnifiedExecSessionManager::default(),
            notify,
            hook_executor,
            state: Mutex::new(state),
            #[cfg(feature = "slash_commands")]
            slash_commands,
            rollout: Mutex::new(Some(rollout_recorder)),
            codex_linux_sandbox_exe: config.codex_linux_sandbox_exe.clone(),
            user_shell: default_shell,
            show_raw_agent_reasoning: config.show_raw_agent_reasoning,
        });

        if config.plan_mode.plan_enabled
            && let Some(telemetry) = sess
                .state
                .lock_unchecked()
                .plan_mode
                .as_ref()
                .map(PlanModeSession::entered_telemetry)
        {
            sess.log_plan_telemetry(&telemetry);
        }

        // Dispatch the SessionConfiguredEvent first and then report any errors.
        // If resuming, include converted initial messages in the payload so UIs can render them immediately.
        let initial_messages = initial_history.get_event_msgs();
        sess.record_initial_history(&turn_context, initial_history)
            .await;

        sess.ensure_plan_mode_prompt_recorded().await;

        let events = std::iter::once(Event {
            id: INITIAL_SUBMIT_ID.to_owned(),
            msg: EventMsg::SessionConfigured(SessionConfiguredEvent {
                session_id: conversation_id,
                model,
                reasoning_effort: model_reasoning_effort,
                history_log_id,
                history_entry_count,
                initial_messages,
                rollout_path,
            }),
        })
        .chain(post_session_configured_error_events.into_iter());
        for event in events {
            sess.send_event(event).await;
        }

        if let Some(session) = sess.plan_mode_payload() {
            let event = Event {
                id: INITIAL_SUBMIT_ID.to_owned(),
                msg: EventMsg::PlanModeActivated(PlanModeActivatedEvent { session }),
            };
            sess.send_event(event).await;
        }

        sess.notify_session_start().await;

        Ok((sess, turn_context))
    }

    pub fn set_task(&self, task: AgentTask) {
        let mut state = self.state.lock_unchecked();
        if let Some(current_task) = state.current_task.take() {
            current_task.abort(TurnAbortReason::Replaced);
        }
        state.current_task = Some(task);
    }

    pub fn remove_task(&self, sub_id: &str) {
        let mut state = self.state.lock_unchecked();
        if let Some(task) = &state.current_task
            && task.sub_id == sub_id
        {
            state.current_task.take();
        }
    }

    fn next_internal_sub_id(&self) -> String {
        self.next_internal_sub_id_with_prefix("auto-compact")
    }

    fn next_internal_sub_id_with_prefix(&self, prefix: &str) -> String {
        let mut state = self.state.lock_unchecked();
        let id = state.next_internal_sub_id;
        state.next_internal_sub_id += 1;
        format!("{prefix}-{id}")
    }

    async fn record_initial_history(
        &self,
        turn_context: &TurnContext,
        conversation_history: InitialHistory,
    ) {
        match conversation_history {
            InitialHistory::New => {
                // Build and record initial items (user instructions + environment context)
                let items = self.build_initial_context(turn_context);
                self.record_conversation_items(&items).await;
            }
            InitialHistory::Resumed(_) | InitialHistory::Forked(_) => {
                let rollout_items = conversation_history.get_rollout_items();
                let persist = matches!(conversation_history, InitialHistory::Forked(_));

                // Always add response items to conversation history
                let reconstructed_history =
                    self.reconstruct_history_from_rollout(turn_context, &rollout_items);
                if !reconstructed_history.is_empty() {
                    self.record_into_history(&reconstructed_history);
                }

                // If persisting, persist all rollout items as-is (recorder filters)
                if persist && !rollout_items.is_empty() {
                    self.persist_rollout_items(&rollout_items).await;
                }
            }
        }
    }

    /// Persist the event to rollout and send it to clients.
    pub(crate) async fn send_event(&self, event: Event) {
        // Persist the event into rollout (recorder filters as needed)
        let rollout_items = vec![RolloutItem::EventMsg(event.msg.clone())];
        self.persist_rollout_items(&rollout_items).await;
        if let Err(e) = self.tx_event.send(event).await {
            error!("failed to send tool call event: {e}");
        }
    }

    pub async fn request_command_approval(
        &self,
        sub_id: String,
        call_id: String,
        command: Vec<String>,
        cwd: PathBuf,
        reason: Option<String>,
    ) -> oneshot::Receiver<ReviewDecision> {
        // Add the tx_approve callback to the map before sending the request.
        let (tx_approve, rx_approve) = oneshot::channel();
        let event_id = sub_id.clone();
        let prev_entry = {
            let mut state = self.state.lock_unchecked();
            state.pending_approvals.insert(sub_id, tx_approve)
        };
        if prev_entry.is_some() {
            warn!("Overwriting existing pending approval for sub_id: {event_id}");
        }

        let event = Event {
            id: event_id,
            msg: EventMsg::ExecApprovalRequest(ExecApprovalRequestEvent {
                call_id,
                command,
                cwd,
                reason,
            }),
        };
        self.send_event(event).await;
        rx_approve
    }

    pub async fn request_patch_approval(
        &self,
        sub_id: String,
        call_id: String,
        action: &ApplyPatchAction,
        reason: Option<String>,
        grant_root: Option<PathBuf>,
    ) -> oneshot::Receiver<ReviewDecision> {
        // Add the tx_approve callback to the map before sending the request.
        let (tx_approve, rx_approve) = oneshot::channel();
        let event_id = sub_id.clone();
        let prev_entry = {
            let mut state = self.state.lock_unchecked();
            state.pending_approvals.insert(sub_id, tx_approve)
        };
        if prev_entry.is_some() {
            warn!("Overwriting existing pending approval for sub_id: {event_id}");
        }

        let event = Event {
            id: event_id,
            msg: EventMsg::ApplyPatchApprovalRequest(ApplyPatchApprovalRequestEvent {
                call_id,
                changes: convert_apply_patch_to_protocol(action),
                reason,
                grant_root,
            }),
        };
        self.send_event(event).await;
        rx_approve
    }

    pub async fn request_subagent_approval(
        &self,
        sub_id: String,
        payload: SubagentApprovalRequestEvent,
    ) -> oneshot::Receiver<SubagentApprovalDecision> {
        let (tx_decision, rx_decision) = oneshot::channel();
        let subagent_key = payload.subagent.clone();
        let prev_entry = {
            let mut state = self.state.lock_unchecked();
            state
                .pending_subagent_approvals
                .insert(subagent_key.clone(), tx_decision)
        };
        if prev_entry.is_some() {
            warn!("Overwriting existing pending subagent approval for {subagent_key}");
        }

        let event = Event {
            id: sub_id,
            msg: EventMsg::SubagentApprovalRequest(payload),
        };
        self.send_event(event).await;
        rx_decision
    }

    pub fn notify_subagent_approval(&self, subagent: &str, decision: SubagentApprovalDecision) {
        let entry = {
            let mut state = self.state.lock_unchecked();
            state.pending_subagent_approvals.remove(subagent)
        };
        match entry {
            Some(tx_decision) => {
                let _ = tx_decision.send(decision);
            }
            None => {
                warn!("No pending subagent approval found for subagent: {subagent}");
            }
        }
    }

    pub fn notify_approval(&self, sub_id: &str, decision: ReviewDecision) {
        let entry = {
            let mut state = self.state.lock_unchecked();
            state.pending_approvals.remove(sub_id)
        };
        match entry {
            Some(tx_approve) => {
                tx_approve.send(decision).ok();
            }
            None => {
                warn!("No pending approval found for sub_id: {sub_id}");
            }
        }
    }

    pub fn add_approved_command(&self, cmd: Vec<String>) {
        let mut state = self.state.lock_unchecked();
        state.approved_commands.insert(cmd);
    }

    /// Records input items: always append to conversation history and
    /// persist these response items to rollout.
    async fn record_conversation_items(&self, items: &[ResponseItem]) {
        self.record_into_history(items);
        self.persist_rollout_response_items(items).await;
    }

    fn reconstruct_history_from_rollout(
        &self,
        turn_context: &TurnContext,
        rollout_items: &[RolloutItem],
    ) -> Vec<ResponseItem> {
        let mut history = ConversationHistory::new();
        for item in rollout_items {
            match item {
                RolloutItem::ResponseItem(response_item) => {
                    history.record_items(std::iter::once(response_item));
                }
                RolloutItem::Compacted(compacted) => {
                    let snapshot = history.contents();
                    let user_messages = collect_user_messages(&snapshot);
                    let rebuilt = build_compacted_history(
                        self.build_initial_context(turn_context),
                        &user_messages,
                        &compacted.message,
                    );
                    history.replace(rebuilt);
                }
                _ => {}
            }
        }
        history.contents()
    }

    /// Append ResponseItems to the in-memory conversation history only.
    fn record_into_history(&self, items: &[ResponseItem]) {
        self.state
            .lock_unchecked()
            .history
            .record_items(items.iter());
    }

    async fn persist_rollout_response_items(&self, items: &[ResponseItem]) {
        let rollout_items: Vec<RolloutItem> = items
            .iter()
            .cloned()
            .map(RolloutItem::ResponseItem)
            .collect();
        self.persist_rollout_items(&rollout_items).await;
    }

    fn build_initial_context(&self, turn_context: &TurnContext) -> Vec<ResponseItem> {
        let mut items = Vec::<ResponseItem>::with_capacity(2);
        if let Some(user_instructions) = turn_context.user_instructions.as_deref() {
            items.push(UserInstructions::new(user_instructions.to_string()).into());
        }
        items.push(ResponseItem::from(EnvironmentContext::new(
            Some(turn_context.cwd.clone()),
            Some(turn_context.approval_policy),
            Some(turn_context.sandbox_policy.clone()),
            Some(self.user_shell.clone()),
        )));
        items
    }

    async fn persist_rollout_items(&self, items: &[RolloutItem]) {
        let recorder = {
            let guard = self.rollout.lock_unchecked();
            guard.as_ref().cloned()
        };
        if let Some(rec) = recorder
            && let Err(e) = rec.record_items(items).await
        {
            error!("failed to record rollout items: {e:#}");
        }
    }

    fn update_token_usage_info(
        &self,
        turn_context: &TurnContext,
        token_usage: &Option<TokenUsage>,
    ) -> Option<TokenUsageInfo> {
        let mut state = self.state.lock_unchecked();
        let info = TokenUsageInfo::new_or_append(
            &state.token_info,
            token_usage,
            turn_context.client.get_model_context_window(),
        );
        state.token_info = info.clone();
        info
    }

    /// Record a user input item to conversation history and also persist a
    /// corresponding UserMessage EventMsg to rollout.
    async fn record_input_and_rollout_usermsg(&self, response_input: &ResponseInputItem) {
        let response_item: ResponseItem = response_input.clone().into();
        // Add to conversation history and persist response item to rollout
        self.record_conversation_items(std::slice::from_ref(&response_item))
            .await;

        // Derive user message events and persist only UserMessage to rollout
        let msgs =
            map_response_item_to_event_messages(&response_item, self.show_raw_agent_reasoning);
        let user_msgs: Vec<RolloutItem> = msgs
            .into_iter()
            .filter_map(|m| match m {
                EventMsg::UserMessage(ev) => Some(RolloutItem::EventMsg(EventMsg::UserMessage(ev))),
                _ => None,
            })
            .collect();
        if !user_msgs.is_empty() {
            self.persist_rollout_items(&user_msgs).await;
        }
    }

    async fn on_exec_command_begin(
        &self,
        turn_diff_tracker: &mut TurnDiffTracker,
        exec_command_context: ExecCommandContext,
    ) {
        let ExecCommandContext {
            sub_id,
            call_id,
            command_for_display,
            cwd,
            apply_patch,
        } = exec_command_context;
        let msg = match apply_patch {
            Some(ApplyPatchCommandContext {
                user_explicitly_approved_this_action,
                changes,
            }) => {
                turn_diff_tracker.on_patch_begin(&changes);

                EventMsg::PatchApplyBegin(PatchApplyBeginEvent {
                    call_id,
                    auto_approved: !user_explicitly_approved_this_action,
                    changes,
                })
            }
            None => EventMsg::ExecCommandBegin(ExecCommandBeginEvent {
                call_id,
                command: command_for_display.clone(),
                cwd,
                parsed_cmd: parse_command(&command_for_display)
                    .into_iter()
                    .map(Into::into)
                    .collect(),
            }),
        };
        let event = Event {
            id: sub_id.to_string(),
            msg,
        };
        self.send_event(event).await;
    }

    async fn on_exec_command_end(
        &self,
        turn_diff_tracker: &mut TurnDiffTracker,
        sub_id: &str,
        call_id: &str,
        output: &ExecToolCallOutput,
        is_apply_patch: bool,
    ) {
        let ExecToolCallOutput {
            stdout,
            stderr,
            aggregated_output,
            duration,
            exit_code,
            timed_out: _,
        } = output;
        // Send full stdout/stderr to clients; do not truncate.
        let stdout = stdout.text.clone();
        let stderr = stderr.text.clone();
        let formatted_output = format_exec_output_str(output);
        let aggregated_output: String = aggregated_output.text.clone();

        let msg = if is_apply_patch {
            EventMsg::PatchApplyEnd(PatchApplyEndEvent {
                call_id: call_id.to_string(),
                stdout,
                stderr,
                success: *exit_code == 0,
            })
        } else {
            EventMsg::ExecCommandEnd(ExecCommandEndEvent {
                call_id: call_id.to_string(),
                stdout,
                stderr,
                aggregated_output,
                exit_code: *exit_code,
                duration: *duration,
                formatted_output,
            })
        };

        let event = Event {
            id: sub_id.to_string(),
            msg,
        };
        self.send_event(event).await;

        // If this is an apply_patch, after we emit the end patch, emit a second event
        // with the full turn diff if there is one.
        if is_apply_patch {
            let unified_diff = turn_diff_tracker.get_unified_diff();
            if let Ok(Some(unified_diff)) = unified_diff {
                let msg = EventMsg::TurnDiff(TurnDiffEvent { unified_diff });
                let event = Event {
                    id: sub_id.into(),
                    msg,
                };
                self.send_event(event).await;
            }
        }
    }
    /// Runs the exec tool call and emits events for the begin and end of the
    /// command even on error.
    ///
    /// Returns the output of the exec tool call.
    async fn run_exec_with_events<'a>(
        &self,
        turn_diff_tracker: &mut TurnDiffTracker,
        begin_ctx: ExecCommandContext,
        exec_args: ExecInvokeArgs<'a>,
    ) -> crate::error::Result<ExecToolCallOutput> {
        let is_apply_patch = begin_ctx.apply_patch.is_some();
        let sub_id = begin_ctx.sub_id.clone();
        let call_id = begin_ctx.call_id.clone();

        let raw_command = begin_ctx.command_for_display.join(" ");
        let command_label = if raw_command.is_empty() {
            "command execution".to_string()
        } else {
            raw_command.clone()
        };

        let plan_mode_shell_allow = {
            let state = self.state.lock_unchecked();
            state.plan_mode.as_ref().map(|session| {
                if begin_ctx.apply_patch.is_some() {
                    false
                } else {
                    session.is_shell_allowed(&begin_ctx.command_for_display)
                }
            })
        };

        if matches!(plan_mode_shell_allow, Some(false)) {
            let (entry_type, summary, details) =
                if let Some(patch_ctx) = begin_ctx.apply_patch.as_ref() {
                    let change_count = patch_ctx.changes.len();
                    let files: Vec<String> = patch_ctx
                        .changes
                        .keys()
                        .map(|path| path.display().to_string())
                        .collect();
                    let summary = if change_count == 0 {
                        "apply_patch".to_string()
                    } else {
                        format!(
                            "apply_patch affecting {change_count} file{}",
                            if change_count == 1 { "" } else { "s" }
                        )
                    };
                    let details = if files.is_empty() {
                        None
                    } else {
                        Some(format!("files: {}", files.join(", ")))
                    };
                    (PlanEntryType::FileChange, summary, details)
                } else {
                    let mut detail_lines = Vec::new();
                    if !command_label.is_empty() {
                        detail_lines.push(format!("command: {command_label}"));
                    }
                    detail_lines.push(format!("cwd: {}", begin_ctx.cwd.display()));
                    let details = Some(detail_lines.join(
                        "
",
                    ));
                    (PlanEntryType::Command, command_label, details)
                };
            let mut message = match entry_type {
                PlanEntryType::FileChange => {
                    format!("Plan Mode captured proposed file changes ({summary})")
                }
                PlanEntryType::Command => format!("Plan Mode captured command `{summary}`"),
                PlanEntryType::Research | PlanEntryType::Decision => summary.clone(),
            };
            if let Some(detail_text) = details.clone() {
                message.push('\n');
                message.push_str(&detail_text);
            }
            self.capture_plan_entry(&sub_id, entry_type, summary.clone(), details.clone())
                .await;
            self.notify_background_event(&sub_id, message.clone()).await;
            let output = ExecToolCallOutput {
                exit_code: -1,
                stdout: StreamOutput::new(String::new()),
                stderr: StreamOutput::new(message.clone()),
                aggregated_output: StreamOutput::new(message),
                duration: Duration::default(),
                timed_out: false,
            };
            return Ok(output);
        }

        let pre_hook_decision = self.run_pre_tool_hooks(&raw_command).await;

        if let Some(decision) = &pre_hook_decision {
            if decision.decision != HookOutcome::Allow {
                let mut messages: Vec<String> = decision
                    .message
                    .iter()
                    .chain(decision.system_message.iter())
                    .cloned()
                    .collect();

                if messages.is_empty() {
                    messages.push(format!(
                        "Command blocked by pre-tool hook (outcome: {:?}).",
                        decision.decision
                    ));
                }

                let combined_message = messages.join("\n");
                let exit_code = if decision.exit_code == 0 {
                    1
                } else {
                    decision.exit_code
                };

                let output = ExecToolCallOutput {
                    exit_code,
                    stdout: StreamOutput::new(String::new()),
                    stderr: StreamOutput::new(combined_message.clone()),
                    aggregated_output: StreamOutput::new(combined_message),
                    duration: Duration::default(),
                    timed_out: false,
                };

                self.run_post_tool_hooks(&raw_command, Some(&output), pre_hook_decision.as_ref())
                    .await;

                return Ok(output);
            }
        }

        self.on_exec_command_begin(turn_diff_tracker, begin_ctx.clone())
            .await;

        let result = process_exec_tool_call(
            exec_args.params,
            exec_args.sandbox_type,
            exec_args.sandbox_policy,
            exec_args.codex_linux_sandbox_exe,
            exec_args.stdout_stream,
        )
        .await;

        let output_stderr;
        let borrowed: &ExecToolCallOutput = match &result {
            Ok(output) => output,
            Err(CodexErr::Sandbox(SandboxErr::Timeout { output })) => output,
            Err(e) => {
                output_stderr = ExecToolCallOutput {
                    exit_code: -1,
                    stdout: StreamOutput::new(String::new()),
                    stderr: StreamOutput::new(get_error_message_ui(e)),
                    aggregated_output: StreamOutput::new(get_error_message_ui(e)),
                    duration: Duration::default(),
                    timed_out: false,
                };
                &output_stderr
            }
        };

        self.run_post_tool_hooks(
            &raw_command,
            result.as_ref().ok(),
            pre_hook_decision.as_ref(),
        )
        .await;

        self.on_exec_command_end(
            turn_diff_tracker,
            &sub_id,
            &call_id,
            borrowed,
            is_apply_patch,
        )
        .await;

        result
    }

    /// Helper that emits a BackgroundEvent with the given message. This keeps
    /// the call‑sites terse so adding more diagnostics does not clutter the
    /// core agent logic.
    async fn notify_background_event(&self, sub_id: &str, message: impl Into<String>) {
        let event = Event {
            id: sub_id.to_string(),
            msg: EventMsg::BackgroundEvent(BackgroundEventEvent {
                message: message.into(),
            }),
        };
        self.send_event(event).await;
    }

    async fn run_pre_tool_hooks(&self, raw_command: &str) -> Option<HookDecision> {
        if self.hook_executor.is_empty() {
            return None;
        }

        let payload = PreToolUsePayload {
            tool_name: EXEC_COMMAND_TOOL_NAME.to_string(),
            command: raw_command.to_string(),
        };

        match self.hook_executor.evaluate_pre_tool_use(&payload).await {
            Ok(decision) => {
                if decision.decision != HookOutcome::Allow {
                    warn!(
                        %raw_command,
                        outcome = ?decision.decision,
                        "pre-tool hook returned non-allow outcome"
                    );
                }
                Some(decision)
            }
            Err(err) => {
                warn!(%raw_command, "pre-tool hook evaluation failed: {err}");
                None
            }
        }
    }

    async fn run_post_tool_hooks(
        &self,
        raw_command: &str,
        output: Option<&ExecToolCallOutput>,
        _pre_decision: Option<&HookDecision>,
    ) {
        if self.hook_executor.is_empty() {
            return;
        }

        let exit_code = output.map(|result| result.exit_code).unwrap_or_default();
        self.hook_executor
            .record_post_tool_use(raw_command, exit_code)
            .await;
    }

    pub(crate) async fn notify_session_start(&self) {
        if self.hook_executor.is_empty() {
            return;
        }
        self.hook_executor.notify_session_start().await;
    }

    pub(crate) async fn notify_session_end(&self) {
        if self.hook_executor.is_empty() {
            return;
        }
        self.hook_executor.notify_session_end().await;
    }

    pub(crate) async fn notify_user_prompt(&self) {
        if self.hook_executor.is_empty() {
            return;
        }
        self.hook_executor.notify_user_prompt().await;
    }

    pub(crate) async fn on_submission(&self, op: &Op) {
        match op {
            Op::UserInput { .. } | Op::UserTurn { .. } => {
                self.notify_user_prompt().await;
            }
            _ => {}
        }
    }

    async fn notify_stream_error(&self, sub_id: &str, message: impl Into<String>) {
        let event = Event {
            id: sub_id.to_string(),
            msg: EventMsg::StreamError(StreamErrorEvent {
                message: message.into(),
            }),
        };
        self.send_event(event).await;
    }

    /// Build the full turn input by concatenating the current conversation
    /// history with additional items for this turn.
    pub fn turn_input_with_history(&self, extra: Vec<ResponseItem>) -> Vec<ResponseItem> {
        let (history, plan_prompt_required) = {
            let state = self.state.lock_unchecked();
            (
                state.history.contents(),
                state.plan_mode.is_some() && state.plan_mode_prompt_recorded,
            )
        };

        let mut items = Vec::with_capacity(history.len() + extra.len() + 1);
        if plan_prompt_required {
            items.push(ResponseItem::Message {
                id: None,
                role: "user".to_string(),
                content: vec![ContentItem::InputText {
                    text: PLAN_MODE_SYSTEM_PROMPT.to_string(),
                }],
            });
        }
        items.extend(history);
        items.extend(extra);
        items
    }

    /// Returns the input if there was no task running to inject into
    pub fn inject_input(&self, input: Vec<InputItem>) -> Result<(), Vec<InputItem>> {
        let mut state = self.state.lock_unchecked();
        if state.current_task.is_some() {
            state.pending_input.push(input.into());
            Ok(())
        } else {
            Err(input)
        }
    }

    pub fn get_pending_input(&self) -> Vec<ResponseInputItem> {
        let mut state = self.state.lock_unchecked();
        if state.pending_input.is_empty() {
            Vec::with_capacity(0)
        } else {
            let mut ret = Vec::new();
            std::mem::swap(&mut ret, &mut state.pending_input);
            ret
        }
    }

    pub async fn call_tool(
        &self,
        sub_id: &str,
        server: &str,
        tool: &str,
        arguments: Option<serde_json::Value>,
        timeout: Option<Duration>,
    ) -> anyhow::Result<CallToolResult> {
        let blocked = {
            let state = self.state.lock_unchecked();
            match state.plan_mode.as_ref() {
                Some(session) => {
                    let qualified = format!("{server}{MCP_TOOL_NAME_DELIMITER}{tool}");
                    if session.is_tool_allowed(&qualified) || session.is_tool_allowed(tool) {
                        None
                    } else {
                        Some(())
                    }
                }
                None => None,
            }
        };

        if blocked.is_some() {
            let summary = format!("Call MCP tool {server}::{tool}");
            let details = arguments.as_ref().map(|args| format!("arguments: {args}"));
            self.capture_plan_entry(sub_id, PlanEntryType::Research, summary, details)
                .await;
            let message = format!(
                "Plan Mode blocked MCP tool {server}::{tool}. Exit Plan Mode or update plan_mode.allowed_read_only_tools to run it."
            );
            self.notify_background_event(sub_id, message.clone()).await;
            return Err(anyhow::anyhow!(message));
        }

        self.mcp_connection_manager
            .call_tool(server, tool, arguments, timeout)
            .await
    }

    fn interrupt_task(&self) {
        info!("interrupt received: abort current task, if any");
        let mut state = self.state.lock_unchecked();
        state.pending_approvals.clear();
        state.pending_input.clear();
        if let Some(task) = state.current_task.take() {
            task.abort(TurnAbortReason::Interrupted);
        }
    }

    /// Spawn the configured notifier (if any) with the given JSON payload as
    /// the last argument. Failures are logged but otherwise ignored so that
    /// notification issues do not interfere with the main workflow.
    fn plan_mode_capabilities(manager: &McpConnectionManager) -> Vec<ToolCapability> {
        let mut capabilities = Vec::new();
        let mut seen = HashSet::new();

        for (qualified_name, tool) in manager.list_all_tools() {
            let mcp_types::Tool {
                annotations, name, ..
            } = tool;

            let read_only = annotations
                .as_ref()
                .and_then(|ann| ann.read_only_hint)
                .unwrap_or(false);
            if !read_only {
                continue;
            }

            let requires_network = annotations
                .as_ref()
                .and_then(|ann| ann.open_world_hint)
                .unwrap_or(false);

            Self::push_capability(
                &mut capabilities,
                &mut seen,
                qualified_name,
                requires_network,
            );
            Self::push_capability(&mut capabilities, &mut seen, name, requires_network);
        }

        // Attachments are allowed only when explicitly configured.
        Self::push_capability(
            &mut capabilities,
            &mut seen,
            "attachments.read".to_string(),
            true,
        );

        capabilities
    }

    fn push_capability(
        capabilities: &mut Vec<ToolCapability>,
        seen: &mut HashSet<String>,
        id: String,
        requires_network: bool,
    ) {
        if seen.insert(id.clone()) {
            capabilities.push(
                ToolCapability::new(id, ToolMode::ReadOnly)
                    .with_network_requirement(requires_network),
            );
        }
    }

    fn plan_mode_payload(&self) -> Option<PlanModeSessionPayload> {
        self.state
            .lock_unchecked()
            .plan_mode
            .as_ref()
            .map(PlanModeSession::to_payload)
    }

    async fn ensure_plan_mode_prompt_recorded(&self) {
        let mut state = self.state.lock_unchecked();
        if state.plan_mode.is_some() && !state.plan_mode_prompt_recorded {
            state.plan_mode_prompt_recorded = true;
        }
    }

    fn try_enter_plan_mode(
        &self,
        approval_policy: AskForApproval,
        config: &Config,
        network_enabled: bool,
    ) -> Result<PlanModeSessionPayload, String> {
        let mut state = self.state.lock_unchecked();
        if state.plan_mode.is_some() {
            return Err("Plan Mode is already active".to_string());
        }
        let session = PlanModeSession::new(
            Uuid::from(self.conversation_id),
            approval_policy,
            Self::plan_mode_capabilities(&self.mcp_connection_manager),
            &config.plan_mode,
            network_enabled,
        );
        let telemetry = session.entered_telemetry();
        let payload = session.to_payload();
        state.plan_mode = Some(session);
        state.plan_mode_prompt_recorded = false;
        drop(state);
        self.log_plan_telemetry(&telemetry);
        Ok(payload)
    }

    fn try_exit_plan_mode(&self) -> Result<AskForApproval, String> {
        let mut state = self.state.lock_unchecked();
        let Some(mut session) = state.plan_mode.take() else {
            return Err("Plan Mode is not active".to_string());
        };
        state.plan_mode_prompt_recorded = false;
        let previous_mode = session.entered_from;
        let entry_count = session.plan_artifact.entry_count();
        session.exit_plan_mode();
        drop(state);
        let telemetry = PlanTelemetry::new(PlanModeEvent::Exit, previous_mode, entry_count);
        self.log_plan_telemetry(&telemetry);
        Ok(previous_mode)
    }

    fn try_apply_plan_mode(
        &self,
        target_mode: Option<AskForApproval>,
    ) -> Result<(AskForApproval, usize, PlanArtifact), String> {
        let mut state = self.state.lock_unchecked();
        let Some(mut session) = state.plan_mode.take() else {
            return Err("Plan Mode is not active".to_string());
        };
        state.plan_mode_prompt_recorded = false;
        let previous_mode = session.entered_from;
        let resolved_mode = target_mode.unwrap_or(session.entered_from);
        let entry_count = session.plan_artifact.entry_count();
        session.begin_apply(Some(resolved_mode));
        let artifact = session.plan_artifact.clone();
        drop(state);
        let telemetry = PlanTelemetry::new(PlanModeEvent::ApplySuccess, previous_mode, entry_count);
        self.log_plan_telemetry(&telemetry);
        Ok((resolved_mode, entry_count, artifact))
    }

    async fn capture_plan_entry(
        &self,
        sub_id: &str,
        entry_type: PlanEntryType,
        summary: impl Into<String>,
        details: Option<String>,
    ) -> Option<PlanTelemetry> {
        let summary = summary.into();
        let result = {
            let mut state = self.state.lock_unchecked();
            let session = state.plan_mode.as_mut()?;
            let telemetry = session.record_refusal(entry_type, summary.clone(), details.clone());
            let payload = session.to_payload();
            (telemetry, payload)
        };
        let (telemetry, payload) = result;
        let event = Event {
            id: sub_id.to_owned(),
            msg: EventMsg::PlanModeUpdated(PlanModeUpdatedEvent { session: payload }),
        };
        self.send_event(event).await;
        self.log_plan_telemetry(&telemetry);
        Some(telemetry)
    }

    pub(crate) async fn apply_plan_tool_update(
        &self,
        sub_id: &str,
        update: &UpdatePlanArgs,
    ) -> Option<()> {
        let payload = {
            let mut state = self.state.lock_unchecked();
            let session = match state.plan_mode.as_mut() {
                Some(session) => session,
                None => return None,
            };
            session.plan_artifact.next_actions = update
                .plan
                .iter()
                .map(|item| match item.status {
                    StepStatus::Pending => format!("[pending] {}", item.step),
                    StepStatus::InProgress => format!("[in-progress] {}", item.step),
                    StepStatus::Completed => format!("[completed] {}", item.step),
                })
                .collect();
            if let Some(explanation) = &update.explanation {
                session.plan_artifact.assumptions = vec![explanation.clone()];
            }
            session.to_payload()
        };
        let event = Event {
            id: sub_id.to_owned(),
            msg: EventMsg::PlanModeUpdated(PlanModeUpdatedEvent { session: payload }),
        };
        self.send_event(event).await;
        Some(())
    }

    async fn record_plan_summary(&self, sub_id: &str, artifact: &PlanArtifact) {
        let summary = artifact.to_summary_markdown();
        if summary.is_empty() {
            return;
        }

        let response_item = ResponseItem::Message {
            id: None,
            role: "system".to_string(),
            content: vec![ContentItem::InputText {
                text: summary.clone(),
            }],
        };
        self.record_conversation_items(std::slice::from_ref(&response_item))
            .await;

        let event = Event {
            id: sub_id.to_owned(),
            msg: EventMsg::AgentMessage(AgentMessageEvent { message: summary }),
        };
        self.send_event(event).await;
    }

    fn log_plan_telemetry(&self, telemetry: &PlanTelemetry) {
        info!(
            target: "plan_mode",
            event = ?telemetry.event,
            previous_mode = ?telemetry.previous_mode,
            plan_entry_count = telemetry.plan_entry_count,
            occurred_at = %telemetry.occurred_at.to_rfc3339(),
            "plan_mode_telemetry"
        );
    }

    fn validate_plan_mode_attachments(
        &self,
        items: &[InputItem],
        turn_context: &TurnContext,
    ) -> Result<(), String> {
        let env_context = EnvironmentContext::from(turn_context);
        let allow_attachments = {
            let state = self.state.lock_unchecked();
            let Some(session) = state.plan_mode.as_ref() else {
                return Ok(());
            };
            session.is_tool_allowed("attachments.read")
        };

        let attachments: Vec<PathBuf> = items
            .iter()
            .filter_map(|item| match item {
                InputItem::LocalImage { path } => Some(path.clone()),
                _ => None,
            })
            .collect();

        if attachments.is_empty() {
            return Ok(());
        }

        if !allow_attachments {
            return Err(
                "Attachments are disabled while Plan Mode is active. Remove the files or exit Plan Mode before continuing.".to_string(),
            );
        }

        for original in attachments {
            let absolute = if original.is_absolute() {
                original.clone()
            } else {
                turn_context.cwd.join(&original)
            };
            let canonical = match absolute.canonicalize() {
                Ok(path) => path,
                Err(_) => {
                    return Err(format!(
                        "Plan Mode could not verify attachment path {path}. Remove the attachment or exit Plan Mode.",
                        path = absolute.display()
                    ));
                }
            };
            if !env_context.is_path_within_workspace(&canonical) {
                return Err(format!(
                    "Plan Mode only permits attachments from {workspace}. Remove {path} or exit Plan Mode before continuing.",
                    workspace = turn_context.cwd.display(),
                    path = canonical.display()
                ));
            }
        }

        Ok(())
    }

    fn maybe_notify(&self, notification: UserNotification) {
        let Some(notify_command) = &self.notify else {
            return;
        };

        if notify_command.is_empty() {
            return;
        }

        let Ok(json) = serde_json::to_string(&notification) else {
            error!("failed to serialise notification payload");
            return;
        };

        let mut command = std::process::Command::new(&notify_command[0]);
        if notify_command.len() > 1 {
            command.args(&notify_command[1..]);
        }
        command.arg(json);

        // Fire-and-forget – we do not wait for completion.
        if let Err(e) = command.spawn() {
            warn!("failed to spawn notifier '{}': {e}", notify_command[0]);
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.interrupt_task();
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ExecCommandContext {
    pub(crate) sub_id: String,
    pub(crate) call_id: String,
    pub(crate) command_for_display: Vec<String>,
    pub(crate) cwd: PathBuf,
    pub(crate) apply_patch: Option<ApplyPatchCommandContext>,
}

#[derive(Clone, Debug)]
pub(crate) struct ApplyPatchCommandContext {
    pub(crate) user_explicitly_approved_this_action: bool,
    pub(crate) changes: HashMap<PathBuf, FileChange>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum AgentTaskKind {
    Regular,
    Review,
    Compact,
}

/// A series of Turns in response to user input.
pub(crate) struct AgentTask {
    sess: Arc<Session>,
    sub_id: String,
    handle: AbortHandle,
    kind: AgentTaskKind,
}

impl AgentTask {
    fn spawn(
        sess: Arc<Session>,
        turn_context: Arc<TurnContext>,
        sub_id: String,
        input: Vec<InputItem>,
    ) -> Self {
        let handle = {
            let sess = sess.clone();
            let sub_id = sub_id.clone();
            let tc = Arc::clone(&turn_context);
            tokio::spawn(async move { run_task(sess, tc, sub_id, input).await }).abort_handle()
        };
        Self {
            sess,
            sub_id,
            handle,
            kind: AgentTaskKind::Regular,
        }
    }

    fn review(
        sess: Arc<Session>,
        turn_context: Arc<TurnContext>,
        sub_id: String,
        input: Vec<InputItem>,
    ) -> Self {
        let handle = {
            let sess = sess.clone();
            let sub_id = sub_id.clone();
            let tc = Arc::clone(&turn_context);
            tokio::spawn(async move { run_task(sess, tc, sub_id, input).await }).abort_handle()
        };
        Self {
            sess,
            sub_id,
            handle,
            kind: AgentTaskKind::Review,
        }
    }

    fn compact(
        sess: Arc<Session>,
        turn_context: Arc<TurnContext>,
        sub_id: String,
        input: Vec<InputItem>,
        compact_instructions: String,
    ) -> Self {
        let handle = {
            let sess = sess.clone();
            let sub_id = sub_id.clone();
            let tc = Arc::clone(&turn_context);
            tokio::spawn(async move {
                compact::run_compact_task(sess, tc, sub_id, input, compact_instructions).await
            })
            .abort_handle()
        };
        Self {
            sess,
            sub_id,
            handle,
            kind: AgentTaskKind::Compact,
        }
    }

    fn abort(self, reason: TurnAbortReason) {
        // TOCTOU?
        if !self.handle.is_finished() {
            if self.kind == AgentTaskKind::Review {
                let sess = self.sess.clone();
                let sub_id = self.sub_id.clone();
                tokio::spawn(async move {
                    exit_review_mode(sess, sub_id, None).await;
                });
            }
            self.handle.abort();
            let event = Event {
                id: self.sub_id,
                msg: EventMsg::TurnAborted(TurnAbortedEvent { reason }),
            };
            let sess = self.sess;
            tokio::spawn(async move {
                sess.send_event(event).await;
            });
        }
    }
}

async fn submission_loop(
    sess: Arc<Session>,
    turn_context: TurnContext,
    config: Arc<Config>,
    rx_sub: Receiver<Submission>,
) {
    // Wrap once to avoid cloning TurnContext for each task.
    let mut turn_context = Arc::new(turn_context);
    // To break out of this loop, send Op::Shutdown.
    while let Ok(sub) = rx_sub.recv().await {
        debug!(?sub, "Submission");
        match sub.op {
            Op::Interrupt => {
                sess.interrupt_task();
            }
            #[cfg(feature = "slash_commands")]
            Op::ReloadSlashCommands => {
                if let Some(service) = sess.slash_commands.as_ref() {
                    match service.reload().await {
                        Ok(count) => {
                            let message = match count {
                                0 => "Reloaded 0 slash commands".to_string(),
                                1 => "Reloaded 1 slash command".to_string(),
                                _ => format!("Reloaded {count} slash commands"),
                            };
                            sess.notify_background_event(&sub.id, message).await;
                        }
                        Err(err) => {
                            let message = format!("Failed to reload slash commands: {err}");
                            let event = Event {
                                id: sub.id.clone(),
                                msg: EventMsg::Error(ErrorEvent { message }),
                            };
                            sess.send_event(event).await;
                        }
                    }
                } else {
                    let event = Event {
                        id: sub.id.clone(),
                        msg: EventMsg::Error(ErrorEvent {
                            message: "Slash commands are not enabled for this session.".into(),
                        }),
                    };
                    sess.send_event(event).await;
                }
            }
            Op::OverrideTurnContext {
                cwd,
                approval_policy,
                sandbox_policy,
                model,
                effort,
                summary,
            } => {
                // Recalculate the persistent turn context with provided overrides.
                let prev = Arc::clone(&turn_context);
                let provider = prev.client.get_provider();

                // Effective model + family
                let (effective_model, effective_family) = if let Some(ref m) = model {
                    let fam =
                        find_family_for_model(m).unwrap_or_else(|| config.model_family.clone());
                    (m.clone(), fam)
                } else {
                    (prev.client.get_model(), prev.client.get_model_family())
                };

                // Effective reasoning settings
                let effective_effort = effort.unwrap_or(prev.client.get_reasoning_effort());
                let effective_summary = summary.unwrap_or(prev.client.get_reasoning_summary());

                let auth_manager = prev.client.get_auth_manager();

                // Build updated config for the client
                let mut updated_config = (*config).clone();
                updated_config.model = effective_model.clone();
                updated_config.model_family = effective_family.clone();
                if let Some(model_info) = get_model_info(&effective_family) {
                    updated_config.model_context_window = Some(model_info.context_window);
                }

                let updated_config = Arc::new(updated_config);
                let client = ModelClient::new(
                    updated_config.clone(),
                    auth_manager,
                    provider,
                    effective_effort,
                    effective_summary,
                    sess.conversation_id,
                );

                let new_approval_policy = approval_policy.unwrap_or(prev.approval_policy);
                let new_sandbox_policy = sandbox_policy
                    .clone()
                    .unwrap_or(prev.sandbox_policy.clone());
                let new_cwd = cwd.clone().unwrap_or_else(|| prev.cwd.clone());

                let tools_config = ToolsConfig::new(&ToolsConfigParams {
                    model_family: &effective_family,
                    approval_policy: new_approval_policy,
                    sandbox_policy: new_sandbox_policy.clone(),
                    include_plan_tool: config.include_plan_tool,
                    include_apply_patch_tool: config.include_apply_patch_tool,
                    include_web_search_request: config.tools_web_search_request,
                    use_streamable_shell_tool: config.use_experimental_streamable_shell_tool,
                    include_view_image_tool: config.include_view_image_tool,
                    experimental_unified_exec_tool: config.use_experimental_unified_exec_tool,
                });

                let (subagent_inventory, subagent_tool, subagent_config) =
                    compute_subagent_tooling(updated_config.as_ref());

                let user_instructions = merge_subagent_user_instructions(
                    prev.user_instructions.clone(),
                    subagent_inventory.as_deref(),
                );

                let new_turn_context = TurnContext {
                    client,
                    tools_config,
                    user_instructions,
                    base_instructions: prev.base_instructions.clone(),
                    approval_policy: new_approval_policy,
                    sandbox_policy: new_sandbox_policy.clone(),
                    shell_environment_policy: prev.shell_environment_policy.clone(),
                    cwd: new_cwd.clone(),
                    is_review_mode: false,
                    subagent_inventory,
                    subagent_tool,
                    subagent_config,
                };

                // Install the new persistent context for subsequent tasks/turns.
                turn_context = Arc::new(new_turn_context);

                // Optionally persist changes to model / effort
                if cwd.is_some() || approval_policy.is_some() || sandbox_policy.is_some() {
                    sess.record_conversation_items(&[ResponseItem::from(EnvironmentContext::new(
                        cwd,
                        approval_policy,
                        sandbox_policy,
                        // Shell is not configurable from turn to turn
                        None,
                    ))])
                    .await;
                }
            }
            Op::UserInput { items } => {
                #[allow(unused_mut)]
                let mut items = items;
                #[cfg(feature = "slash_commands")]
                let mut slash_model_override: Option<String> = None;
                #[cfg(feature = "slash_commands")]
                {
                    if let Some(service) = sess.slash_commands.as_ref()
                        && let Some(command_text) = items.iter().find_map(|item| {
                            if let InputItem::Text { text } = item {
                                Some(text.clone())
                            } else {
                                None
                            }
                        })
                    {
                        match service.resolve(&command_text).await {
                            Ok(invocation) => {
                                fn rebuild_items(
                                    original: &[InputItem],
                                    invocation: &CommandInvocation,
                                ) -> Vec<InputItem> {
                                    let mut replaced = false;
                                    let mut out = Vec::with_capacity(original.len().max(1));
                                    for item in original {
                                        match item {
                                            InputItem::Text { .. } if !replaced => {
                                                out.push(InputItem::Text {
                                                    text: invocation.rendered_body.clone(),
                                                });
                                                replaced = true;
                                            }
                                            InputItem::Text { text } => {
                                                out.push(InputItem::Text { text: text.clone() });
                                            }
                                            InputItem::Image { image_url } => {
                                                out.push(InputItem::Image {
                                                    image_url: image_url.clone(),
                                                });
                                            }
                                            InputItem::LocalImage { path } => {
                                                out.push(InputItem::LocalImage {
                                                    path: path.clone(),
                                                });
                                            }
                                            _ => out.push(item.clone()),
                                        }
                                    }
                                    if !replaced {
                                        out.insert(
                                            0,
                                            InputItem::Text {
                                                text: invocation.rendered_body.clone(),
                                            },
                                        );
                                    }
                                    out
                                }

                                slash_model_override = invocation.command.metadata.model.clone();
                                items = rebuild_items(&items, &invocation);
                            }
                            Err(InvocationError::NotCommand) => {}
                            Err(InvocationError::NotFound { name, suggestions }) => {
                                let mut message = format!("Unknown slash command: /{name}");
                                if !suggestions.is_empty() {
                                    message.push_str(". Did you mean: ");
                                    message.push_str(&suggestions.join(", "));
                                }
                                let event = Event {
                                    id: sub.id.clone(),
                                    msg: EventMsg::Error(ErrorEvent { message }),
                                };
                                sess.send_event(event).await;
                                continue;
                            }
                            Err(InvocationError::Ambiguous { matches, .. }) => {
                                let message = format!(
                                    "Slash command is ambiguous; try one of: {}",
                                    matches.join(", ")
                                );
                                let event = Event {
                                    id: sub.id.clone(),
                                    msg: EventMsg::Error(ErrorEvent { message }),
                                };
                                sess.send_event(event).await;
                                continue;
                            }
                            Err(InvocationError::Interpolation(message)) => {
                                let event = Event {
                                    id: sub.id.clone(),
                                    msg: EventMsg::Error(ErrorEvent { message }),
                                };
                                sess.send_event(event).await;
                                continue;
                            }
                        }
                    }
                }

                if let Err(message) = sess.validate_plan_mode_attachments(&items, &turn_context) {
                    let event = Event {
                        id: sub.id.clone(),
                        msg: EventMsg::Error(ErrorEvent { message }),
                    };
                    sess.send_event(event).await;
                    continue;
                }

                // attempt to inject input into current task
                if let Err(items) = sess.inject_input(items) {
                    #[cfg(feature = "slash_commands")]
                    if let Some(model) = slash_model_override {
                        handle_slash_command_turn(
                            sess.clone(),
                            Arc::clone(&turn_context),
                            config.clone(),
                            sub.id.clone(),
                            model,
                            items,
                        )
                        .await;
                        continue;
                    }
                    // no current task, spawn a new one
                    let task =
                        AgentTask::spawn(sess.clone(), Arc::clone(&turn_context), sub.id, items);
                    sess.set_task(task);
                }
            }
            Op::UserTurn {
                items,
                cwd,
                approval_policy,
                sandbox_policy,
                model,
                effort,
                summary,
            } => {
                // attempt to inject input into current task
                if let Err(items) = sess.inject_input(items) {
                    // Derive a fresh TurnContext for this turn using the provided overrides.
                    let provider = turn_context.client.get_provider();
                    let auth_manager = turn_context.client.get_auth_manager();

                    // Derive a model family for the requested model; fall back to the session's.
                    let model_family = find_family_for_model(&model)
                        .unwrap_or_else(|| config.model_family.clone());

                    // Create a per‑turn Config clone with the requested model/family.
                    let mut per_turn_config = (*config).clone();
                    per_turn_config.model = model.clone();
                    per_turn_config.model_family = model_family.clone();
                    if let Some(model_info) = get_model_info(&model_family) {
                        per_turn_config.model_context_window = Some(model_info.context_window);
                    }

                    // Build a new client with per‑turn reasoning settings.
                    // Reuse the same provider and session id; auth defaults to env/API key.
                    let per_turn_config = Arc::new(per_turn_config);
                    let client = ModelClient::new(
                        per_turn_config.clone(),
                        auth_manager,
                        provider,
                        effort,
                        summary,
                        sess.conversation_id,
                    );

                    let (subagent_inventory, subagent_tool, subagent_config) =
                        compute_subagent_tooling(per_turn_config.as_ref());

                    let user_instructions = merge_subagent_user_instructions(
                        turn_context.user_instructions.clone(),
                        subagent_inventory.as_deref(),
                    );

                    let fresh_turn_context = TurnContext {
                        client,
                        tools_config: ToolsConfig::new(&ToolsConfigParams {
                            model_family: &model_family,
                            approval_policy,
                            sandbox_policy: sandbox_policy.clone(),
                            include_plan_tool: config.include_plan_tool,
                            include_apply_patch_tool: config.include_apply_patch_tool,
                            include_web_search_request: config.tools_web_search_request,
                            use_streamable_shell_tool: config
                                .use_experimental_streamable_shell_tool,
                            include_view_image_tool: config.include_view_image_tool,
                            experimental_unified_exec_tool: config
                                .use_experimental_unified_exec_tool,
                        }),
                        user_instructions,
                        base_instructions: turn_context.base_instructions.clone(),
                        approval_policy,
                        sandbox_policy,
                        shell_environment_policy: turn_context.shell_environment_policy.clone(),
                        cwd,
                        is_review_mode: false,
                        subagent_inventory,
                        subagent_tool,
                        subagent_config,
                    };

                    // if the environment context has changed, record it in the conversation history
                    let previous_env_context = EnvironmentContext::from(turn_context.as_ref());
                    let new_env_context = EnvironmentContext::from(&fresh_turn_context);
                    if !new_env_context.equals_except_shell(&previous_env_context) {
                        sess.record_conversation_items(&[ResponseItem::from(new_env_context)])
                            .await;
                    }

                    // Install the new persistent context for subsequent tasks/turns.
                    turn_context = Arc::new(fresh_turn_context);

                    // no current task, spawn a new one with the per‑turn context
                    let task =
                        AgentTask::spawn(sess.clone(), Arc::clone(&turn_context), sub.id, items);
                    sess.set_task(task);
                }
            }
            Op::ExecApproval { id, decision } => match decision {
                ReviewDecision::Abort => {
                    sess.interrupt_task();
                }
                other => sess.notify_approval(&id, other),
            },
            Op::PatchApproval { id, decision } => match decision {
                ReviewDecision::Abort => {
                    sess.interrupt_task();
                }
                other => sess.notify_approval(&id, other),
            },
            Op::SubagentApproval { name, decision } => {
                sess.notify_subagent_approval(&name, decision);
            }
            Op::AddToHistory { text } => {
                let id = sess.conversation_id;
                let config = config.clone();
                tokio::spawn(async move {
                    if let Err(e) = crate::message_history::append_entry(&text, &id, &config).await
                    {
                        warn!("failed to append to message history: {e}");
                    }
                });
            }

            Op::GetHistoryEntryRequest { offset, log_id } => {
                let config = config.clone();
                let sess_clone = sess.clone();
                let sub_id = sub.id.clone();

                tokio::spawn(async move {
                    // Run lookup in blocking thread because it does file IO + locking.
                    let entry_opt = tokio::task::spawn_blocking(move || {
                        crate::message_history::lookup(log_id, offset, &config)
                    })
                    .await
                    .unwrap_or(None);

                    let event = Event {
                        id: sub_id,
                        msg: EventMsg::GetHistoryEntryResponse(
                            crate::protocol::GetHistoryEntryResponseEvent {
                                offset,
                                log_id,
                                entry: entry_opt.map(|e| {
                                    codex_protocol::message_history::HistoryEntry {
                                        conversation_id: e.session_id,
                                        ts: e.ts,
                                        text: e.text,
                                    }
                                }),
                            },
                        ),
                    };

                    sess_clone.send_event(event).await;
                });
            }
            Op::HookList(request) => {
                let registry = sess.hook_executor.registry();
                let snapshot = build_hook_registry_snapshot(&registry, &request);
                let event = Event {
                    id: sub.id.clone(),
                    msg: EventMsg::HookListResponse(HookRegistrySnapshotEvent {
                        registry: snapshot,
                    }),
                };
                sess.send_event(event).await;
            }
            Op::HookExecLog(_request) => {
                let event = Event {
                    id: sub.id.clone(),
                    msg: EventMsg::HookExecLogResponse(HookExecLogResponseEvent {
                        logs: HookExecLogResponse {
                            records: Vec::new(),
                        },
                    }),
                };
                sess.send_event(event).await;
            }
            Op::HookValidate(_request) => {
                let event = Event {
                    id: sub.id.clone(),
                    msg: EventMsg::HookValidationResult(HookValidationResultEvent {
                        summary: HookValidationSummary {
                            status: HookValidationStatus::Ok,
                            errors: Vec::new(),
                            warnings: Vec::new(),
                            layers: Vec::new(),
                        },
                    }),
                };
                sess.send_event(event).await;
            }
            Op::HookReload => {
                let event = Event {
                    id: sub.id.clone(),
                    msg: EventMsg::HookReloadResult(HookReloadResultEvent {
                        result: HookReloadResponse {
                            reloaded: false,
                            message: Some("hook reload placeholder".to_string()),
                        },
                    }),
                };
                sess.send_event(event).await;
            }
            Op::ListMcpTools => {
                let sub_id = sub.id.clone();

                // This is a cheap lookup from the connection manager's cache.
                let tools = sess.mcp_connection_manager.list_all_tools();
                let event = Event {
                    id: sub_id,
                    msg: EventMsg::McpListToolsResponse(
                        crate::protocol::McpListToolsResponseEvent { tools },
                    ),
                };
                sess.send_event(event).await;
            }
            Op::ListCustomPrompts => {
                let sub_id = sub.id.clone();

                let custom_prompts: Vec<CustomPrompt> =
                    if let Some(dir) = crate::custom_prompts::default_prompts_dir() {
                        crate::custom_prompts::discover_prompts_in(&dir).await
                    } else {
                        Vec::new()
                    };

                let event = Event {
                    id: sub_id,
                    msg: EventMsg::ListCustomPromptsResponse(ListCustomPromptsResponseEvent {
                        custom_prompts,
                    }),
                };
                sess.send_event(event).await;
            }
            Op::Compact => {
                // Attempt to inject input into current task
                if let Err(items) = sess.inject_input(vec![InputItem::Text {
                    text: compact::COMPACT_TRIGGER_TEXT.to_string(),
                }]) {
                    compact::spawn_compact_task(
                        sess.clone(),
                        Arc::clone(&turn_context),
                        sub.id,
                        items,
                    );
                }
            }
            Op::EnterPlanMode => {
                let network_enabled = turn_context.sandbox_policy.has_full_network_access();
                match sess.try_enter_plan_mode(
                    turn_context.approval_policy,
                    &config,
                    network_enabled,
                ) {
                    Ok(session) => {
                        let event = Event {
                            id: sub.id.clone(),
                            msg: EventMsg::PlanModeActivated(PlanModeActivatedEvent { session }),
                        };
                        sess.send_event(event).await;
                        let previous = Arc::clone(&turn_context);
                        let tools_config = ToolsConfig::new(&ToolsConfigParams {
                            model_family: &config.model_family,
                            approval_policy: previous.approval_policy,
                            sandbox_policy: previous.sandbox_policy.clone(),
                            include_plan_tool: true,
                            include_apply_patch_tool: false,
                            include_web_search_request: config.tools_web_search_request,
                            use_streamable_shell_tool: config
                                .use_experimental_streamable_shell_tool,
                            include_view_image_tool: config.include_view_image_tool,
                            experimental_unified_exec_tool: config
                                .use_experimental_unified_exec_tool,
                        });
                        turn_context = Arc::new(TurnContext {
                            client: previous.client.clone(),
                            tools_config,
                            user_instructions: previous.user_instructions.clone(),
                            base_instructions: previous.base_instructions.clone(),
                            approval_policy: previous.approval_policy,
                            sandbox_policy: previous.sandbox_policy.clone(),
                            shell_environment_policy: previous.shell_environment_policy.clone(),
                            cwd: previous.cwd.clone(),
                            is_review_mode: previous.is_review_mode,
                            subagent_inventory: previous.subagent_inventory.clone(),
                            subagent_tool: previous.subagent_tool.clone(),
                            subagent_config: previous.subagent_config.clone(),
                        });
                        sess.ensure_plan_mode_prompt_recorded().await;
                    }
                    Err(message) => {
                        let event = Event {
                            id: sub.id.clone(),
                            msg: EventMsg::Error(ErrorEvent { message }),
                        };
                        sess.send_event(event).await;
                    }
                }
            }
            Op::ExitPlanMode => match sess.try_exit_plan_mode() {
                Ok(previous_mode) => {
                    let event = Event {
                        id: sub.id.clone(),
                        msg: EventMsg::PlanModeExited(PlanModeExitedEvent { previous_mode }),
                    };
                    sess.send_event(event).await;
                    let previous = Arc::clone(&turn_context);
                    let tools_config = ToolsConfig::new(&ToolsConfigParams {
                        model_family: &config.model_family,
                        approval_policy: previous_mode,
                        sandbox_policy: previous.sandbox_policy.clone(),
                        include_plan_tool: config.include_plan_tool,
                        include_apply_patch_tool: config.include_apply_patch_tool,
                        include_web_search_request: config.tools_web_search_request,
                        use_streamable_shell_tool: config.use_experimental_streamable_shell_tool,
                        include_view_image_tool: config.include_view_image_tool,
                        experimental_unified_exec_tool: config.use_experimental_unified_exec_tool,
                    });
                    turn_context = Arc::new(TurnContext {
                        client: previous.client.clone(),
                        tools_config,
                        user_instructions: previous.user_instructions.clone(),
                        base_instructions: previous.base_instructions.clone(),
                        approval_policy: previous_mode,
                        sandbox_policy: previous.sandbox_policy.clone(),
                        shell_environment_policy: previous.shell_environment_policy.clone(),
                        cwd: previous.cwd.clone(),
                        is_review_mode: previous.is_review_mode,
                        subagent_inventory: previous.subagent_inventory.clone(),
                        subagent_tool: previous.subagent_tool.clone(),
                        subagent_config: previous.subagent_config.clone(),
                    });
                }
                Err(message) => {
                    let event = Event {
                        id: sub.id.clone(),
                        msg: EventMsg::Error(ErrorEvent { message }),
                    };
                    sess.send_event(event).await;
                }
            },
            Op::ApplyPlanMode { target_mode } => match sess.try_apply_plan_mode(target_mode) {
                Ok((target_mode, plan_entries, plan_artifact)) => {
                    let event = Event {
                        id: sub.id.clone(),
                        msg: EventMsg::PlanModeApplied(PlanModeAppliedEvent {
                            target_mode,
                            plan_entries,
                        }),
                    };
                    sess.send_event(event).await;
                    sess.record_plan_summary(&sub.id, &plan_artifact).await;
                    let previous = Arc::clone(&turn_context);
                    let tools_config = ToolsConfig::new(&ToolsConfigParams {
                        model_family: &config.model_family,
                        approval_policy: target_mode,
                        sandbox_policy: previous.sandbox_policy.clone(),
                        include_plan_tool: config.include_plan_tool,
                        include_apply_patch_tool: config.include_apply_patch_tool,
                        include_web_search_request: config.tools_web_search_request,
                        use_streamable_shell_tool: config.use_experimental_streamable_shell_tool,
                        include_view_image_tool: config.include_view_image_tool,
                        experimental_unified_exec_tool: config.use_experimental_unified_exec_tool,
                    });
                    turn_context = Arc::new(TurnContext {
                        client: previous.client.clone(),
                        tools_config,
                        user_instructions: previous.user_instructions.clone(),
                        base_instructions: previous.base_instructions.clone(),
                        approval_policy: target_mode,
                        sandbox_policy: previous.sandbox_policy.clone(),
                        shell_environment_policy: previous.shell_environment_policy.clone(),
                        cwd: previous.cwd.clone(),
                        is_review_mode: previous.is_review_mode,
                        subagent_inventory: previous.subagent_inventory.clone(),
                        subagent_tool: previous.subagent_tool.clone(),
                        subagent_config: previous.subagent_config.clone(),
                    });

                    let follow_up_sub_id =
                        sess.next_internal_sub_id_with_prefix("plan-implementation");
                    let follow_up_input = vec![InputItem::Text {
                        text: PLAN_IMPLEMENTATION_TRIGGER_TEXT.to_string(),
                    }];
                    let follow_up_task = AgentTask::spawn(
                        sess.clone(),
                        Arc::clone(&turn_context),
                        follow_up_sub_id,
                        follow_up_input,
                    );
                    sess.set_task(follow_up_task);
                }
                Err(message) => {
                    let event = Event {
                        id: sub.id.clone(),
                        msg: EventMsg::Error(ErrorEvent { message }),
                    };
                    sess.send_event(event).await;
                }
            },
            Op::Shutdown => {
                info!("Shutting down Codex instance");

                // Gracefully flush and shutdown rollout recorder on session end so tests
                // that inspect the rollout file do not race with the background writer.
                let recorder_opt = sess.rollout.lock_unchecked().take();
                if let Some(rec) = recorder_opt
                    && let Err(e) = rec.shutdown().await
                {
                    warn!("failed to shutdown rollout recorder: {e}");
                    let event = Event {
                        id: sub.id.clone(),
                        msg: EventMsg::Error(ErrorEvent {
                            message: "Failed to shutdown rollout recorder".to_string(),
                        }),
                    };
                    sess.send_event(event).await;
                }

                let event = Event {
                    id: sub.id.clone(),
                    msg: EventMsg::ShutdownComplete,
                };
                sess.send_event(event).await;
                break;
            }
            Op::GetPath => {
                let sub_id = sub.id.clone();
                // Flush rollout writes before returning the path so readers observe a consistent file.
                let (path, rec_opt) = {
                    let guard = sess.rollout.lock_unchecked();
                    match guard.as_ref() {
                        Some(rec) => (rec.get_rollout_path(), Some(rec.clone())),
                        None => {
                            error!("rollout recorder not found");
                            continue;
                        }
                    }
                };
                if let Some(rec) = rec_opt
                    && let Err(e) = rec.flush().await
                {
                    warn!("failed to flush rollout recorder before GetHistory: {e}");
                }
                let event = Event {
                    id: sub_id.clone(),
                    msg: EventMsg::ConversationPath(ConversationPathResponseEvent {
                        conversation_id: sess.conversation_id,
                        path,
                    }),
                };
                sess.send_event(event).await;
            }
            Op::Review { review_request } => {
                spawn_review_thread(
                    sess.clone(),
                    config.clone(),
                    turn_context.clone(),
                    sub.id,
                    review_request,
                )
                .await;
            }
            _ => {
                // Ignore unknown ops; enum is non_exhaustive to allow extensions.
            }
        }
    }
    debug!("Agent loop exited");
}

/// Spawn a review thread using the given prompt.
async fn spawn_review_thread(
    sess: Arc<Session>,
    config: Arc<Config>,
    parent_turn_context: Arc<TurnContext>,
    sub_id: String,
    review_request: ReviewRequest,
) {
    let model = config.review_model.clone();
    let review_model_family = find_family_for_model(&model)
        .unwrap_or_else(|| parent_turn_context.client.get_model_family());
    let tools_config = ToolsConfig::new(&ToolsConfigParams {
        model_family: &review_model_family,
        approval_policy: parent_turn_context.approval_policy,
        sandbox_policy: parent_turn_context.sandbox_policy.clone(),
        include_plan_tool: false,
        include_apply_patch_tool: config.include_apply_patch_tool,
        include_web_search_request: false,
        use_streamable_shell_tool: false,
        include_view_image_tool: false,
        experimental_unified_exec_tool: config.use_experimental_unified_exec_tool,
    });

    let base_instructions = Some(REVIEW_PROMPT.to_string());
    let provider = parent_turn_context.client.get_provider();
    let auth_manager = parent_turn_context.client.get_auth_manager();
    let model_family = review_model_family.clone();

    // Build per‑turn client with the requested model/family.
    let mut per_turn_config = (*config).clone();
    per_turn_config.model = model.clone();
    per_turn_config.model_family = model_family.clone();
    if let Some(model_info) = get_model_info(&model_family) {
        per_turn_config.model_context_window = Some(model_info.context_window);
    }

    let per_turn_config = Arc::new(per_turn_config);
    let client = ModelClient::new(
        per_turn_config.clone(),
        auth_manager,
        provider,
        parent_turn_context.client.get_reasoning_effort(),
        parent_turn_context.client.get_reasoning_summary(),
        sess.conversation_id,
    );

    let (subagent_inventory, subagent_tool, subagent_config) =
        compute_subagent_tooling(per_turn_config.as_ref());

    let review_turn_context = TurnContext {
        client,
        tools_config,
        user_instructions: None,
        base_instructions,
        approval_policy: parent_turn_context.approval_policy,
        sandbox_policy: parent_turn_context.sandbox_policy.clone(),
        shell_environment_policy: parent_turn_context.shell_environment_policy.clone(),
        cwd: parent_turn_context.cwd.clone(),
        is_review_mode: true,
        subagent_inventory,
        subagent_tool,
        subagent_config,
    };

    // Seed the child task with the review prompt as the initial user message.
    let input: Vec<InputItem> = vec![InputItem::Text {
        text: review_request.prompt.clone(),
    }];
    let tc = Arc::new(review_turn_context);

    // Clone sub_id for the upcoming announcement before moving it into the task.
    let sub_id_for_event = sub_id.clone();
    let task = AgentTask::review(sess.clone(), tc.clone(), sub_id, input);
    sess.set_task(task);

    // Announce entering review mode so UIs can switch modes.
    sess.send_event(Event {
        id: sub_id_for_event,
        msg: EventMsg::EnteredReviewMode(review_request),
    })
    .await;
}

/// Takes a user message as input and runs a loop where, at each turn, the model
/// replies with either:
///
/// - requested function calls
/// - an assistant message
///
/// While it is possible for the model to return multiple of these items in a
/// single turn, in practice, we generally one item per turn:
///
/// - If the model requests a function call, we execute it and send the output
///   back to the model in the next turn.
/// - If the model sends only an assistant message, we record it in the
///   conversation history and consider the task complete.
///
/// Review mode: when `turn_context.is_review_mode` is true, the turn runs in an
/// isolated in-memory thread without the parent session's prior history or
/// user_instructions. Emits ExitedReviewMode upon final review message.
async fn run_task(
    sess: Arc<Session>,
    turn_context: Arc<TurnContext>,
    sub_id: String,
    input: Vec<InputItem>,
) {
    if input.is_empty() {
        return;
    }
    let event = Event {
        id: sub_id.clone(),
        msg: EventMsg::TaskStarted(TaskStartedEvent {
            model_context_window: turn_context.client.get_model_context_window(),
        }),
    };
    sess.send_event(event).await;

    let initial_input_for_turn: ResponseInputItem = ResponseInputItem::from(input);
    // For review threads, keep an isolated in-memory history so the
    // model sees a fresh conversation without the parent session's history.
    // For normal turns, continue recording to the session history as before.
    let is_review_mode = turn_context.is_review_mode;
    let mut review_thread_history: Vec<ResponseItem> = Vec::new();
    if is_review_mode {
        review_thread_history.push(initial_input_for_turn.into());
    } else {
        sess.record_input_and_rollout_usermsg(&initial_input_for_turn)
            .await;
    }

    let mut last_agent_message: Option<String> = None;
    // Although from the perspective of codex.rs, TurnDiffTracker has the lifecycle of a Task which contains
    // many turns, from the perspective of the user, it is a single turn.
    let mut turn_diff_tracker = TurnDiffTracker::new();
    let mut auto_compact_recently_attempted = false;

    loop {
        // Note that pending_input would be something like a message the user
        // submitted through the UI while the model was running. Though the UI
        // may support this, the model might not.
        let pending_input = sess
            .get_pending_input()
            .into_iter()
            .map(ResponseItem::from)
            .collect::<Vec<ResponseItem>>();

        // Construct the input that we will send to the model.
        //
        // - For review threads, use the isolated in-memory history so the
        //   model sees a fresh conversation (no parent history/user_instructions).
        //
        // - For normal turns, use the session's full history. When using the
        //   chat completions API (or ZDR clients), the model needs the full
        //   conversation history on each turn. The rollout file, however, should
        //   only record the new items that originated in this turn so that it
        //   represents an append-only log without duplicates.
        let turn_input: Vec<ResponseItem> = if is_review_mode {
            if !pending_input.is_empty() {
                review_thread_history.extend(pending_input);
            }
            review_thread_history.clone()
        } else {
            sess.record_conversation_items(&pending_input).await;
            sess.turn_input_with_history(pending_input)
        };

        let turn_input_messages: Vec<String> = turn_input
            .iter()
            .filter_map(|item| match item {
                ResponseItem::Message { content, .. } => Some(content),
                _ => None,
            })
            .flat_map(|content| {
                content.iter().filter_map(|item| match item {
                    ContentItem::OutputText { text } => Some(text.clone()),
                    _ => None,
                })
            })
            .collect();
        match run_turn(
            &sess,
            turn_context.as_ref(),
            &mut turn_diff_tracker,
            sub_id.clone(),
            turn_input,
        )
        .await
        {
            Ok(turn_output) => {
                let TurnRunResult {
                    processed_items,
                    total_token_usage,
                } = turn_output;
                let limit = turn_context
                    .client
                    .get_auto_compact_token_limit()
                    .unwrap_or(i64::MAX);
                let total_usage_tokens = total_token_usage
                    .as_ref()
                    .map(|usage| usage.tokens_in_context_window());
                let token_limit_reached = total_usage_tokens
                    .map(|tokens| (tokens as i64) >= limit)
                    .unwrap_or(false);
                let mut items_to_record_in_conversation_history = Vec::<ResponseItem>::new();
                let mut responses = Vec::<ResponseInputItem>::new();
                for processed_response_item in processed_items {
                    let ProcessedResponseItem { item, response } = processed_response_item;
                    match (&item, &response) {
                        (ResponseItem::Message { role, .. }, None) if role == "assistant" => {
                            // If the model returned a message, we need to record it.
                            items_to_record_in_conversation_history.push(item);
                        }
                        (
                            ResponseItem::LocalShellCall { .. },
                            Some(ResponseInputItem::FunctionCallOutput { call_id, output }),
                        ) => {
                            items_to_record_in_conversation_history.push(item);
                            items_to_record_in_conversation_history.push(
                                ResponseItem::FunctionCallOutput {
                                    call_id: call_id.clone(),
                                    output: output.clone(),
                                },
                            );
                        }
                        (
                            ResponseItem::FunctionCall { .. },
                            Some(ResponseInputItem::FunctionCallOutput { call_id, output }),
                        ) => {
                            items_to_record_in_conversation_history.push(item);
                            items_to_record_in_conversation_history.push(
                                ResponseItem::FunctionCallOutput {
                                    call_id: call_id.clone(),
                                    output: output.clone(),
                                },
                            );
                        }
                        (
                            ResponseItem::CustomToolCall { .. },
                            Some(ResponseInputItem::CustomToolCallOutput { call_id, output }),
                        ) => {
                            items_to_record_in_conversation_history.push(item);
                            items_to_record_in_conversation_history.push(
                                ResponseItem::CustomToolCallOutput {
                                    call_id: call_id.clone(),
                                    output: output.clone(),
                                },
                            );
                        }
                        (
                            ResponseItem::FunctionCall { .. },
                            Some(ResponseInputItem::McpToolCallOutput { call_id, result }),
                        ) => {
                            items_to_record_in_conversation_history.push(item);
                            let output = match result {
                                Ok(call_tool_result) => {
                                    convert_call_tool_result_to_function_call_output_payload(
                                        call_tool_result,
                                    )
                                }
                                Err(err) => FunctionCallOutputPayload {
                                    content: err.clone(),
                                    success: Some(false),
                                },
                            };
                            items_to_record_in_conversation_history.push(
                                ResponseItem::FunctionCallOutput {
                                    call_id: call_id.clone(),
                                    output,
                                },
                            );
                        }
                        (
                            ResponseItem::Reasoning {
                                id,
                                summary,
                                content,
                                encrypted_content,
                            },
                            None,
                        ) => {
                            items_to_record_in_conversation_history.push(ResponseItem::Reasoning {
                                id: id.clone(),
                                summary: summary.clone(),
                                content: content.clone(),
                                encrypted_content: encrypted_content.clone(),
                            });
                        }
                        _ => {
                            warn!("Unexpected response item: {item:?} with response: {response:?}");
                        }
                    };
                    if let Some(response) = response {
                        responses.push(response);
                    }
                }

                // Only attempt to take the lock if there is something to record.
                if !items_to_record_in_conversation_history.is_empty() {
                    if is_review_mode {
                        review_thread_history
                            .extend(items_to_record_in_conversation_history.clone());
                    } else {
                        sess.record_conversation_items(&items_to_record_in_conversation_history)
                            .await;
                    }
                }

                if token_limit_reached {
                    if auto_compact_recently_attempted {
                        let limit_str = limit.to_string();
                        let current_tokens = total_usage_tokens
                            .map(|tokens| tokens.to_string())
                            .unwrap_or_else(|| "unknown".to_string());
                        let event = Event {
                            id: sub_id.clone(),
                            msg: EventMsg::Error(ErrorEvent {
                                message: format!(
                                    "Conversation is still above the token limit after automatic summarization (limit {limit_str}, current {current_tokens}). Please start a new session or trim your input."
                                ),
                            }),
                        };
                        sess.send_event(event).await;
                        break;
                    }
                    auto_compact_recently_attempted = true;
                    compact::run_inline_auto_compact_task(sess.clone(), turn_context.clone()).await;
                    continue;
                }

                auto_compact_recently_attempted = false;

                if responses.is_empty() {
                    last_agent_message = get_last_assistant_message_from_turn(
                        &items_to_record_in_conversation_history,
                    );
                    sess.maybe_notify(UserNotification::AgentTurnComplete {
                        turn_id: sub_id.clone(),
                        input_messages: turn_input_messages,
                        last_assistant_message: last_agent_message.clone(),
                    });
                    break;
                }
                continue;
            }
            Err(e) => {
                info!("Turn error: {e:#}");
                let event = Event {
                    id: sub_id.clone(),
                    msg: EventMsg::Error(ErrorEvent {
                        message: e.to_string(),
                    }),
                };
                sess.send_event(event).await;
                // let the user continue the conversation
                break;
            }
        }
    }

    // If this was a review thread and we have a final assistant message,
    // try to parse it as a ReviewOutput.
    //
    // If parsing fails, construct a minimal ReviewOutputEvent using the plain
    // text as the overall explanation. Else, just exit review mode with None.
    //
    // Emits an ExitedReviewMode event with the parsed review output.
    if turn_context.is_review_mode {
        exit_review_mode(
            sess.clone(),
            sub_id.clone(),
            last_agent_message.as_deref().map(parse_review_output_event),
        )
        .await;
    }

    sess.remove_task(&sub_id);
    let event = Event {
        id: sub_id,
        msg: EventMsg::TaskComplete(TaskCompleteEvent { last_agent_message }),
    };
    sess.send_event(event).await;
}

/// Parse the review output; when not valid JSON, build a structured
/// fallback that carries the plain text as the overall explanation.
///
/// Returns: a ReviewOutputEvent parsed from JSON or a fallback populated from text.
fn parse_review_output_event(text: &str) -> ReviewOutputEvent {
    // Try direct parse first
    if let Ok(ev) = serde_json::from_str::<ReviewOutputEvent>(text) {
        return ev;
    }
    // If wrapped in markdown fences or extra prose, attempt to extract the first JSON object
    if let (Some(start), Some(end)) = (text.find('{'), text.rfind('}'))
        && start < end
        && let Some(slice) = text.get(start..=end)
        && let Ok(ev) = serde_json::from_str::<ReviewOutputEvent>(slice)
    {
        return ev;
    }
    // Not JSON – return a structured ReviewOutputEvent that carries
    // the plain text as the overall explanation.
    ReviewOutputEvent {
        overall_explanation: text.to_string(),
        ..Default::default()
    }
}

async fn run_turn(
    sess: &Session,
    turn_context: &TurnContext,
    turn_diff_tracker: &mut TurnDiffTracker,
    sub_id: String,
    input: Vec<ResponseItem>,
) -> CodexResult<TurnRunResult> {
    let tools = get_openai_tools(
        &turn_context.tools_config,
        Some(sess.mcp_connection_manager.list_all_tools()),
        turn_context.subagent_tool.as_ref(),
    );

    let prompt = Prompt {
        input,
        tools,
        base_instructions_override: turn_context.base_instructions.clone(),
    };

    let mut retries = 0;
    loop {
        match try_run_turn(sess, turn_context, turn_diff_tracker, &sub_id, &prompt).await {
            Ok(output) => return Ok(output),
            Err(CodexErr::Interrupted) => return Err(CodexErr::Interrupted),
            Err(CodexErr::EnvVar(var)) => return Err(CodexErr::EnvVar(var)),
            Err(e @ (CodexErr::UsageLimitReached(_) | CodexErr::UsageNotIncluded)) => {
                return Err(e);
            }
            Err(e) => {
                // Use the configured provider-specific stream retry budget.
                let max_retries = turn_context.client.get_provider().stream_max_retries();
                if retries < max_retries {
                    retries += 1;
                    let delay = match e {
                        CodexErr::Stream(_, Some(delay)) => delay,
                        _ => backoff(retries),
                    };
                    warn!(
                        "stream disconnected - retrying turn ({retries}/{max_retries} in {delay:?})...",
                    );

                    // Surface retry information to any UI/front‑end so the
                    // user understands what is happening instead of staring
                    // at a seemingly frozen screen.
                    sess.notify_stream_error(
                        &sub_id,
                        format!(
                            "stream error: {e}; retrying {retries}/{max_retries} in {delay:?}…"
                        ),
                    )
                    .await;

                    tokio::time::sleep(delay).await;
                } else {
                    return Err(e);
                }
            }
        }
    }
}

/// When the model is prompted, it returns a stream of events. Some of these
/// events map to a `ResponseItem`. A `ResponseItem` may need to be
/// "handled" such that it produces a `ResponseInputItem` that needs to be
/// sent back to the model on the next turn.
#[derive(Debug)]
struct ProcessedResponseItem {
    item: ResponseItem,
    response: Option<ResponseInputItem>,
}

#[derive(Debug)]
struct TurnRunResult {
    processed_items: Vec<ProcessedResponseItem>,
    total_token_usage: Option<TokenUsage>,
}

async fn try_run_turn(
    sess: &Session,
    turn_context: &TurnContext,
    turn_diff_tracker: &mut TurnDiffTracker,
    sub_id: &str,
    prompt: &Prompt,
) -> CodexResult<TurnRunResult> {
    // call_ids that are part of this response.
    let completed_call_ids = prompt
        .input
        .iter()
        .filter_map(|ri| match ri {
            ResponseItem::FunctionCallOutput { call_id, .. } => Some(call_id),
            ResponseItem::LocalShellCall {
                call_id: Some(call_id),
                ..
            } => Some(call_id),
            ResponseItem::CustomToolCallOutput { call_id, .. } => Some(call_id),
            _ => None,
        })
        .collect::<Vec<_>>();

    // call_ids that were pending but are not part of this response.
    // This usually happens because the user interrupted the model before we responded to one of its tool calls
    // and then the user sent a follow-up message.
    let missing_calls = {
        prompt
            .input
            .iter()
            .filter_map(|ri| match ri {
                ResponseItem::FunctionCall { call_id, .. } => Some(call_id),
                ResponseItem::LocalShellCall {
                    call_id: Some(call_id),
                    ..
                } => Some(call_id),
                ResponseItem::CustomToolCall { call_id, .. } => Some(call_id),
                _ => None,
            })
            .filter_map(|call_id| {
                if completed_call_ids.contains(&call_id) {
                    None
                } else {
                    Some(call_id.clone())
                }
            })
            .map(|call_id| ResponseItem::CustomToolCallOutput {
                call_id,
                output: "aborted".to_string(),
            })
            .collect::<Vec<_>>()
    };
    let prompt: Cow<Prompt> = if missing_calls.is_empty() {
        Cow::Borrowed(prompt)
    } else {
        // Add the synthetic aborted missing calls to the beginning of the input to ensure all call ids have responses.
        let input = [missing_calls, prompt.input.clone()].concat();
        Cow::Owned(Prompt {
            input,
            ..prompt.clone()
        })
    };

    let rollout_item = RolloutItem::TurnContext(TurnContextItem {
        cwd: turn_context.cwd.clone(),
        approval_policy: turn_context.approval_policy,
        sandbox_policy: turn_context.sandbox_policy.clone(),
        model: turn_context.client.get_model(),
        effort: turn_context.client.get_reasoning_effort(),
        summary: turn_context.client.get_reasoning_summary(),
    });
    sess.persist_rollout_items(&[rollout_item]).await;
    let mut stream = turn_context.client.clone().stream(&prompt).await?;

    let mut output = Vec::new();

    loop {
        // Poll the next item from the model stream. We must inspect *both* Ok and Err
        // cases so that transient stream failures (e.g., dropped SSE connection before
        // `response.completed`) bubble up and trigger the caller's retry logic.
        let event = stream.next().await;
        let Some(event) = event else {
            // Channel closed without yielding a final Completed event or explicit error.
            // Treat as a disconnected stream so the caller can retry.
            return Err(CodexErr::Stream(
                "stream closed before response.completed".into(),
                None,
            ));
        };

        let event = match event {
            Ok(ev) => ev,
            Err(e) => {
                // Propagate the underlying stream error to the caller (run_turn), which
                // will apply the configured `stream_max_retries` policy.
                return Err(e);
            }
        };

        match event {
            ResponseEvent::Created => {}
            ResponseEvent::OutputItemDone(item) => {
                let response = handle_response_item(
                    sess,
                    turn_context,
                    turn_diff_tracker,
                    sub_id,
                    item.clone(),
                )
                .await?;
                output.push(ProcessedResponseItem { item, response });
            }
            ResponseEvent::WebSearchCallBegin { call_id } => {
                let _ = sess
                    .tx_event
                    .send(Event {
                        id: sub_id.to_string(),
                        msg: EventMsg::WebSearchBegin(WebSearchBeginEvent { call_id }),
                    })
                    .await;
            }
            ResponseEvent::Completed {
                response_id: _,
                token_usage,
            } => {
                let info = sess.update_token_usage_info(turn_context, &token_usage);
                let _ = sess
                    .send_event(Event {
                        id: sub_id.to_string(),
                        msg: EventMsg::TokenCount(crate::protocol::TokenCountEvent { info }),
                    })
                    .await;

                let unified_diff = turn_diff_tracker.get_unified_diff();
                if let Ok(Some(unified_diff)) = unified_diff {
                    let msg = EventMsg::TurnDiff(TurnDiffEvent { unified_diff });
                    let event = Event {
                        id: sub_id.to_string(),
                        msg,
                    };
                    sess.send_event(event).await;
                }

                let result = TurnRunResult {
                    processed_items: output,
                    total_token_usage: token_usage.clone(),
                };

                return Ok(result);
            }
            ResponseEvent::OutputTextDelta(delta) => {
                // In review child threads, suppress assistant text deltas; the
                // UI will show a selection popup from the final ReviewOutput.
                if !turn_context.is_review_mode {
                    let event = Event {
                        id: sub_id.to_string(),
                        msg: EventMsg::AgentMessageDelta(AgentMessageDeltaEvent { delta }),
                    };
                    sess.send_event(event).await;
                } else {
                    trace!("suppressing OutputTextDelta in review mode");
                }
            }
            ResponseEvent::ReasoningSummaryDelta(delta) => {
                let event = Event {
                    id: sub_id.to_string(),
                    msg: EventMsg::AgentReasoningDelta(AgentReasoningDeltaEvent { delta }),
                };
                sess.send_event(event).await;
            }
            ResponseEvent::ReasoningSummaryPartAdded => {
                let event = Event {
                    id: sub_id.to_string(),
                    msg: EventMsg::AgentReasoningSectionBreak(AgentReasoningSectionBreakEvent {}),
                };
                sess.send_event(event).await;
            }
            ResponseEvent::ReasoningContentDelta(delta) => {
                if sess.show_raw_agent_reasoning {
                    let event = Event {
                        id: sub_id.to_string(),
                        msg: EventMsg::AgentReasoningRawContentDelta(
                            AgentReasoningRawContentDeltaEvent { delta },
                        ),
                    };
                    sess.send_event(event).await;
                }
            }
        }
    }
}

async fn handle_response_item(
    sess: &Session,
    turn_context: &TurnContext,
    turn_diff_tracker: &mut TurnDiffTracker,
    sub_id: &str,
    item: ResponseItem,
) -> CodexResult<Option<ResponseInputItem>> {
    debug!(?item, "Output item");
    let output = match item {
        ResponseItem::FunctionCall {
            name,
            arguments,
            call_id,
            ..
        } => {
            info!("FunctionCall: {name}({arguments})");
            Some(
                handle_function_call(
                    sess,
                    turn_context,
                    turn_diff_tracker,
                    sub_id.to_string(),
                    name,
                    arguments,
                    call_id,
                )
                .await,
            )
        }
        ResponseItem::LocalShellCall {
            id,
            call_id,
            status: _,
            action,
        } => {
            let LocalShellAction::Exec(action) = action;
            tracing::info!("LocalShellCall: {action:?}");
            let params = ShellToolCallParams {
                command: action.command,
                workdir: action.working_directory,
                timeout_ms: action.timeout_ms,
                with_escalated_permissions: None,
                justification: None,
            };
            let effective_call_id = match (call_id, id) {
                (Some(call_id), _) => call_id,
                (None, Some(id)) => id,
                (None, None) => {
                    error!("LocalShellCall without call_id or id");
                    return Ok(Some(ResponseInputItem::FunctionCallOutput {
                        call_id: "".to_string(),
                        output: FunctionCallOutputPayload {
                            content: "LocalShellCall without call_id or id".to_string(),
                            success: None,
                        },
                    }));
                }
            };

            let exec_params = to_exec_params(params, turn_context);
            Some(
                handle_container_exec_with_params(
                    exec_params,
                    sess,
                    turn_context,
                    turn_diff_tracker,
                    sub_id.to_string(),
                    effective_call_id,
                )
                .await,
            )
        }
        ResponseItem::CustomToolCall {
            id: _,
            call_id,
            name,
            input,
            status: _,
        } => Some(
            handle_custom_tool_call(
                sess,
                turn_context,
                turn_diff_tracker,
                sub_id.to_string(),
                name,
                input,
                call_id,
            )
            .await,
        ),
        ResponseItem::FunctionCallOutput { .. } => {
            debug!("unexpected FunctionCallOutput from stream");
            None
        }
        ResponseItem::CustomToolCallOutput { .. } => {
            debug!("unexpected CustomToolCallOutput from stream");
            None
        }
        ResponseItem::Message { .. }
        | ResponseItem::Reasoning { .. }
        | ResponseItem::WebSearchCall { .. } => {
            // In review child threads, suppress assistant message events but
            // keep reasoning/web search.
            let msgs = match &item {
                ResponseItem::Message { .. } if turn_context.is_review_mode => {
                    trace!("suppressing assistant Message in review mode");
                    Vec::new()
                }
                _ => map_response_item_to_event_messages(&item, sess.show_raw_agent_reasoning),
            };
            for msg in msgs {
                let event = Event {
                    id: sub_id.to_string(),
                    msg,
                };
                sess.send_event(event).await;
            }
            None
        }
        ResponseItem::Other => None,
    };
    Ok(output)
}

async fn handle_unified_exec_tool_call(
    sess: &Session,
    call_id: String,
    session_id: Option<String>,
    arguments: Vec<String>,
    timeout_ms: Option<u64>,
) -> ResponseInputItem {
    let parsed_session_id = if let Some(session_id) = session_id {
        match session_id.parse::<i32>() {
            Ok(parsed) => Some(parsed),
            Err(output) => {
                return ResponseInputItem::FunctionCallOutput {
                    call_id: call_id.to_string(),
                    output: FunctionCallOutputPayload {
                        content: format!("invalid session_id: {session_id} due to error {output}"),
                        success: Some(false),
                    },
                };
            }
        }
    } else {
        None
    };

    let request = crate::unified_exec::UnifiedExecRequest {
        session_id: parsed_session_id,
        input_chunks: &arguments,
        timeout_ms,
    };

    let result = sess.unified_exec_manager.handle_request(request).await;

    let output_payload = match result {
        Ok(value) => {
            #[derive(Serialize)]
            struct SerializedUnifiedExecResult<'a> {
                session_id: Option<String>,
                output: &'a str,
            }

            match serde_json::to_string(&SerializedUnifiedExecResult {
                session_id: value.session_id.map(|id| id.to_string()),
                output: &value.output,
            }) {
                Ok(serialized) => FunctionCallOutputPayload {
                    content: serialized,
                    success: Some(true),
                },
                Err(err) => FunctionCallOutputPayload {
                    content: format!("failed to serialize unified exec output: {err}"),
                    success: Some(false),
                },
            }
        }
        Err(err) => FunctionCallOutputPayload {
            content: format!("unified exec failed: {err}"),
            success: Some(false),
        },
    };

    ResponseInputItem::FunctionCallOutput {
        call_id,
        output: output_payload,
    }
}

async fn handle_function_call(
    sess: &Session,
    turn_context: &TurnContext,
    turn_diff_tracker: &mut TurnDiffTracker,
    sub_id: String,
    name: String,
    arguments: String,
    call_id: String,
) -> ResponseInputItem {
    match name.as_str() {
        "container.exec" | "shell" => {
            let params = match parse_container_exec_arguments(arguments, turn_context, &call_id) {
                Ok(params) => params,
                Err(output) => {
                    return *output;
                }
            };
            handle_container_exec_with_params(
                params,
                sess,
                turn_context,
                turn_diff_tracker,
                sub_id,
                call_id,
            )
            .await
        }
        "unified_exec" => {
            #[derive(Deserialize)]
            struct UnifiedExecArgs {
                input: Vec<String>,
                #[serde(default)]
                session_id: Option<String>,
                #[serde(default)]
                timeout_ms: Option<u64>,
            }

            let args = match serde_json::from_str::<UnifiedExecArgs>(&arguments) {
                Ok(args) => args,
                Err(err) => {
                    return ResponseInputItem::FunctionCallOutput {
                        call_id,
                        output: FunctionCallOutputPayload {
                            content: format!("failed to parse function arguments: {err}"),
                            success: Some(false),
                        },
                    };
                }
            };

            handle_unified_exec_tool_call(
                sess,
                call_id,
                args.session_id,
                args.input,
                args.timeout_ms,
            )
            .await
        }
        "view_image" => {
            #[derive(serde::Deserialize)]
            struct SeeImageArgs {
                path: String,
            }
            let args = match serde_json::from_str::<SeeImageArgs>(&arguments) {
                Ok(a) => a,
                Err(e) => {
                    return ResponseInputItem::FunctionCallOutput {
                        call_id,
                        output: FunctionCallOutputPayload {
                            content: format!("failed to parse function arguments: {e}"),
                            success: Some(false),
                        },
                    };
                }
            };
            let abs = turn_context.resolve_path(Some(args.path));
            if let Err(message) = sess.validate_plan_mode_attachments(
                &[InputItem::LocalImage { path: abs.clone() }],
                turn_context,
            ) {
                return ResponseInputItem::FunctionCallOutput {
                    call_id,
                    output: FunctionCallOutputPayload {
                        content: message,
                        success: Some(false),
                    },
                };
            }
            let output = match sess.inject_input(vec![InputItem::LocalImage { path: abs }]) {
                Ok(()) => FunctionCallOutputPayload {
                    content: "attached local image path".to_string(),
                    success: Some(true),
                },
                Err(_) => FunctionCallOutputPayload {
                    content: "unable to attach image (no active task)".to_string(),
                    success: Some(false),
                },
            };
            ResponseInputItem::FunctionCallOutput { call_id, output }
        }
        "invoke_subagent" => {
            #[derive(Deserialize)]
            struct InvokeSubagentArgs {
                name: String,
                #[serde(default)]
                instructions: Option<String>,
                #[serde(default)]
                requested_tools: Vec<String>,
                #[serde(default)]
                model: Option<String>,
                #[serde(default)]
                confirmed: bool,
            }

            let args = match serde_json::from_str::<InvokeSubagentArgs>(&arguments) {
                Ok(args) => args,
                Err(err) => {
                    return ResponseInputItem::FunctionCallOutput {
                        call_id,
                        output: FunctionCallOutputPayload {
                            content: format!("failed to parse function arguments: {err}"),
                            success: Some(false),
                        },
                    };
                }
            };

            let Some(inventory_arc) = turn_context.subagent_inventory.as_ref().map(Arc::clone)
            else {
                return ResponseInputItem::FunctionCallOutput {
                    call_id,
                    output: FunctionCallOutputPayload {
                        content: "Subagents feature is disabled for this session.".to_string(),
                        success: Some(false),
                    },
                };
            };
            let Some(subagent_config) = turn_context.subagent_config.clone() else {
                return ResponseInputItem::FunctionCallOutput {
                    call_id,
                    output: FunctionCallOutputPayload {
                        content:
                            "Subagents configuration is unavailable; enable subagents.enabled in config.".to_string(),
                        success: Some(false),
                    },
                };
            };

            let normalized_name = SubagentDefinition::normalize_name(args.name.trim());
            let mut session = InvocationSession::new(&normalized_name);
            session.parent_session_id = Some(sess.conversation_id.to_string());
            if !args.requested_tools.is_empty() {
                session.requested_tools = args.requested_tools.clone();
            }
            if let Some(model) = args.model.clone() {
                session.resolved_model = Some(model);
            }
            if let Some(instructions) = args
                .instructions
                .as_ref()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
            {
                session.extra_instructions = Some(instructions.to_string());
                session
                    .execution_log
                    .push(format!("instructions: {instructions}"));
            }

            if args.confirmed || matches!(subagent_config.discovery, SubagentDiscoveryMode::Manual)
            {
                session = session.confirmed();
            }

            let runner = SubagentRunner::new(&subagent_config, inventory_arc.as_ref());
            let mut current_session = session;
            let prepared = loop {
                match runner.invoke(current_session) {
                    Ok(prepared) => break prepared,
                    Err(SubagentInvocationError::ConfirmationRequired {
                        subagent,
                        record,
                        session: pending_session,
                    }) => {
                        let description = if record.definition.description.trim().is_empty() {
                            None
                        } else {
                            Some(record.definition.description.clone())
                        };

                        let extra_instructions = pending_session
                            .extra_instructions
                            .clone()
                            .filter(|s| !s.trim().is_empty());

                        let payload = SubagentApprovalRequestEvent {
                            subagent: subagent.clone(),
                            description,
                            extra_instructions,
                            allowed_tools: record.effective_tools.clone(),
                            requested_tools: pending_session.requested_tools.clone(),
                            model: pending_session
                                .resolved_model
                                .clone()
                                .or_else(|| record.effective_model.clone()),
                        };

                        let rx = sess
                            .request_subagent_approval(sub_id.clone(), payload)
                            .await;
                        let decision = rx.await.unwrap_or_default();
                        match decision {
                            SubagentApprovalDecision::Approved => {
                                current_session = pending_session.confirmed();
                            }
                            SubagentApprovalDecision::Denied => {
                                return ResponseInputItem::FunctionCallOutput {
                                    call_id,
                                    output: FunctionCallOutputPayload {
                                        content: format!(
                                            "Subagent '{subagent}' invocation denied by user."
                                        ),
                                        success: Some(false),
                                    },
                                };
                            }
                        }
                    }
                    Err(SubagentInvocationError::FeatureDisabled) => {
                        return ResponseInputItem::FunctionCallOutput {
                            call_id,
                            output: FunctionCallOutputPayload {
                                content: "Subagents feature is disabled.".to_string(),
                                success: Some(false),
                            },
                        };
                    }
                    Err(SubagentInvocationError::UnknownSubagent(name)) => {
                        return ResponseInputItem::FunctionCallOutput {
                            call_id,
                            output: FunctionCallOutputPayload {
                                content: format!("No subagent named '{name}'."),
                                success: Some(false),
                            },
                        };
                    }
                    Err(SubagentInvocationError::InvalidSubagent(name)) => {
                        return ResponseInputItem::FunctionCallOutput {
                            call_id,
                            output: FunctionCallOutputPayload {
                                content: format!("Subagent '{name}' is invalid."),
                                success: Some(false),
                            },
                        };
                    }
                    Err(SubagentInvocationError::DisabledSubagent(name)) => {
                        return ResponseInputItem::FunctionCallOutput {
                            call_id,
                            output: FunctionCallOutputPayload {
                                content: format!("Subagent '{name}' is disabled."),
                                success: Some(false),
                            },
                        };
                    }
                    Err(SubagentInvocationError::ToolNotAllowed { subagent, tool }) => {
                        return ResponseInputItem::FunctionCallOutput {
                            call_id,
                            output: FunctionCallOutputPayload {
                                content: format!(
                                    "Tool '{tool}' not allowed for subagent '{subagent}'."
                                ),
                                success: Some(false),
                            },
                        };
                    }
                    Err(SubagentInvocationError::ExecutionFailed(reason)) => {
                        return ResponseInputItem::FunctionCallOutput {
                            call_id,
                            output: FunctionCallOutputPayload {
                                content: reason,
                                success: Some(false),
                            },
                        };
                    }
                    Err(SubagentInvocationError::MissingAuthManager) => {
                        return ResponseInputItem::FunctionCallOutput {
                            call_id,
                            output: FunctionCallOutputPayload {
                                content: "Subagent execution requires authentication context."
                                    .to_string(),
                                success: Some(false),
                            },
                        };
                    }
                }
            };

            let config_arc = turn_context.client.get_config();
            let auth_manager = match turn_context.client.get_auth_manager() {
                Some(manager) => manager,
                None => {
                    return ResponseInputItem::FunctionCallOutput {
                        call_id,
                        output: FunctionCallOutputPayload {
                            content: "Subagent execution requires authentication context."
                                .to_string(),
                            success: Some(false),
                        },
                    };
                }
            };

            match execute_subagent_invocation(config_arc.as_ref(), auth_manager, prepared).await {
                Ok(result_session) => {
                    let detail_artifacts: Vec<String> = result_session
                        .detail_artifacts
                        .iter()
                        .map(|p| p.to_string_lossy().to_string())
                        .collect();
                    let payload_json = serde_json::json!({
                        "name": result_session.subagent_name,
                        "summary": result_session.summary,
                        "detail_artifacts": detail_artifacts,
                        "requested_tools": result_session.requested_tools,
                        "model": result_session.resolved_model,
                        "execution_log": result_session.execution_log,
                    });

                    ResponseInputItem::FunctionCallOutput {
                        call_id,
                        output: FunctionCallOutputPayload {
                            content: payload_json.to_string(),
                            success: Some(true),
                        },
                    }
                }
                Err(err) => {
                    let message = err.to_string();
                    ResponseInputItem::FunctionCallOutput {
                        call_id,
                        output: FunctionCallOutputPayload {
                            content: message,
                            success: Some(false),
                        },
                    }
                }
            }
        }
        "apply_patch" => {
            let args = match serde_json::from_str::<ApplyPatchToolArgs>(&arguments) {
                Ok(a) => a,
                Err(e) => {
                    return ResponseInputItem::FunctionCallOutput {
                        call_id,
                        output: FunctionCallOutputPayload {
                            content: format!("failed to parse function arguments: {e}"),
                            success: None,
                        },
                    };
                }
            };
            let exec_params = ExecParams {
                command: vec!["apply_patch".to_string(), args.input.clone()],
                cwd: turn_context.cwd.clone(),
                timeout_ms: None,
                env: HashMap::new(),
                with_escalated_permissions: None,
                justification: None,
            };
            handle_container_exec_with_params(
                exec_params,
                sess,
                turn_context,
                turn_diff_tracker,
                sub_id,
                call_id,
            )
            .await
        }
        "update_plan" => handle_update_plan(sess, arguments, sub_id, call_id).await,
        EXEC_COMMAND_TOOL_NAME => {
            // TODO(mbolin): Sandbox check.
            let exec_params = match serde_json::from_str::<ExecCommandParams>(&arguments) {
                Ok(params) => params,
                Err(e) => {
                    return ResponseInputItem::FunctionCallOutput {
                        call_id,
                        output: FunctionCallOutputPayload {
                            content: format!("failed to parse function arguments: {e}"),
                            success: Some(false),
                        },
                    };
                }
            };
            let result = sess
                .session_manager
                .handle_exec_command_request(exec_params)
                .await;
            let function_call_output = crate::exec_command::result_into_payload(result);
            ResponseInputItem::FunctionCallOutput {
                call_id,
                output: function_call_output,
            }
        }
        WRITE_STDIN_TOOL_NAME => {
            let write_stdin_params = match serde_json::from_str::<WriteStdinParams>(&arguments) {
                Ok(params) => params,
                Err(e) => {
                    return ResponseInputItem::FunctionCallOutput {
                        call_id,
                        output: FunctionCallOutputPayload {
                            content: format!("failed to parse function arguments: {e}"),
                            success: Some(false),
                        },
                    };
                }
            };
            let result = sess
                .session_manager
                .handle_write_stdin_request(write_stdin_params)
                .await;
            let function_call_output: FunctionCallOutputPayload =
                crate::exec_command::result_into_payload(result);
            ResponseInputItem::FunctionCallOutput {
                call_id,
                output: function_call_output,
            }
        }
        _ => {
            match sess.mcp_connection_manager.parse_tool_name(&name) {
                Some((server, tool_name)) => {
                    // TODO(mbolin): Determine appropriate timeout for tool call.
                    let timeout = None;
                    handle_mcp_tool_call(
                        sess, &sub_id, call_id, server, tool_name, arguments, timeout,
                    )
                    .await
                }
                None => {
                    // Unknown function: reply with structured failure so the model can adapt.
                    ResponseInputItem::FunctionCallOutput {
                        call_id,
                        output: FunctionCallOutputPayload {
                            content: format!("unsupported call: {name}"),
                            success: None,
                        },
                    }
                }
            }
        }
    }
}

async fn handle_custom_tool_call(
    sess: &Session,
    turn_context: &TurnContext,
    turn_diff_tracker: &mut TurnDiffTracker,
    sub_id: String,
    name: String,
    input: String,
    call_id: String,
) -> ResponseInputItem {
    info!("CustomToolCall: {name} {input}");
    match name.as_str() {
        "apply_patch" => {
            let exec_params = ExecParams {
                command: vec!["apply_patch".to_string(), input.clone()],
                cwd: turn_context.cwd.clone(),
                timeout_ms: None,
                env: HashMap::new(),
                with_escalated_permissions: None,
                justification: None,
            };
            let resp = handle_container_exec_with_params(
                exec_params,
                sess,
                turn_context,
                turn_diff_tracker,
                sub_id,
                call_id,
            )
            .await;

            // Convert function-call style output into a custom tool call output
            match resp {
                ResponseInputItem::FunctionCallOutput { call_id, output } => {
                    ResponseInputItem::CustomToolCallOutput {
                        call_id,
                        output: output.content,
                    }
                }
                // Pass through if already a custom tool output or other variant
                other => other,
            }
        }
        _ => {
            debug!("unexpected CustomToolCall from stream");
            ResponseInputItem::CustomToolCallOutput {
                call_id,
                output: format!("unsupported custom tool call: {name}"),
            }
        }
    }
}

fn to_exec_params(params: ShellToolCallParams, turn_context: &TurnContext) -> ExecParams {
    ExecParams {
        command: params.command,
        cwd: turn_context.resolve_path(params.workdir.clone()),
        timeout_ms: params.timeout_ms,
        env: create_env(&turn_context.shell_environment_policy),
        with_escalated_permissions: params.with_escalated_permissions,
        justification: params.justification,
    }
}

fn parse_container_exec_arguments(
    arguments: String,
    turn_context: &TurnContext,
    call_id: &str,
) -> Result<ExecParams, Box<ResponseInputItem>> {
    // parse command
    match serde_json::from_str::<ShellToolCallParams>(&arguments) {
        Ok(shell_tool_call_params) => Ok(to_exec_params(shell_tool_call_params, turn_context)),
        Err(e) => {
            // allow model to re-sample
            let output = ResponseInputItem::FunctionCallOutput {
                call_id: call_id.to_string(),
                output: FunctionCallOutputPayload {
                    content: format!("failed to parse function arguments: {e}"),
                    success: None,
                },
            };
            Err(Box::new(output))
        }
    }
}

pub struct ExecInvokeArgs<'a> {
    pub params: ExecParams,
    pub sandbox_type: SandboxType,
    pub sandbox_policy: &'a SandboxPolicy,
    pub codex_linux_sandbox_exe: &'a Option<PathBuf>,
    pub stdout_stream: Option<StdoutStream>,
}

fn maybe_translate_shell_command(
    params: ExecParams,
    sess: &Session,
    turn_context: &TurnContext,
) -> ExecParams {
    let should_translate = matches!(sess.user_shell, crate::shell::Shell::PowerShell(_))
        || turn_context.shell_environment_policy.use_profile;

    if should_translate
        && let Some(command) = sess
            .user_shell
            .format_default_shell_invocation(params.command.clone())
    {
        return ExecParams { command, ..params };
    }
    params
}

async fn handle_container_exec_with_params(
    params: ExecParams,
    sess: &Session,
    turn_context: &TurnContext,
    turn_diff_tracker: &mut TurnDiffTracker,
    sub_id: String,
    call_id: String,
) -> ResponseInputItem {
    // check if this was a patch, and apply it if so
    let apply_patch_exec = match maybe_parse_apply_patch_verified(&params.command, &params.cwd) {
        MaybeApplyPatchVerified::Body(changes) => {
            match apply_patch::apply_patch(sess, turn_context, &sub_id, &call_id, changes).await {
                InternalApplyPatchInvocation::Output(item) => return item,
                InternalApplyPatchInvocation::DelegateToExec(apply_patch_exec) => {
                    Some(apply_patch_exec)
                }
            }
        }
        MaybeApplyPatchVerified::CorrectnessError(parse_error) => {
            // It looks like an invocation of `apply_patch`, but we
            // could not resolve it into a patch that would apply
            // cleanly. Return to model for resample.
            return ResponseInputItem::FunctionCallOutput {
                call_id,
                output: FunctionCallOutputPayload {
                    content: format!("error: {parse_error:#}"),
                    success: None,
                },
            };
        }
        MaybeApplyPatchVerified::ShellParseError(error) => {
            trace!("Failed to parse shell command, {error:?}");
            None
        }
        MaybeApplyPatchVerified::NotApplyPatch => None,
    };

    let (params, safety, command_for_display) = match &apply_patch_exec {
        Some(ApplyPatchExec {
            action: ApplyPatchAction { patch, cwd, .. },
            user_explicitly_approved_this_action,
        }) => {
            let path_to_codex = std::env::current_exe()
                .ok()
                .map(|p| p.to_string_lossy().to_string());
            let Some(path_to_codex) = path_to_codex else {
                return ResponseInputItem::FunctionCallOutput {
                    call_id,
                    output: FunctionCallOutputPayload {
                        content: "failed to determine path to codex executable".to_string(),
                        success: None,
                    },
                };
            };

            let params = ExecParams {
                command: vec![
                    path_to_codex,
                    CODEX_APPLY_PATCH_ARG1.to_string(),
                    patch.clone(),
                ],
                cwd: cwd.clone(),
                timeout_ms: params.timeout_ms,
                env: HashMap::new(),
                with_escalated_permissions: params.with_escalated_permissions,
                justification: params.justification.clone(),
            };
            let safety = if *user_explicitly_approved_this_action {
                SafetyCheck::AutoApprove {
                    sandbox_type: SandboxType::None,
                }
            } else {
                assess_safety_for_untrusted_command(
                    turn_context.approval_policy,
                    &turn_context.sandbox_policy,
                    params.with_escalated_permissions.unwrap_or(false),
                )
            };
            (
                params,
                safety,
                vec!["apply_patch".to_string(), patch.clone()],
            )
        }
        None => {
            let safety = {
                let state = sess.state.lock_unchecked();
                assess_command_safety(
                    &params.command,
                    turn_context.approval_policy,
                    &turn_context.sandbox_policy,
                    &state.approved_commands,
                    params.with_escalated_permissions.unwrap_or(false),
                )
            };
            let command_for_display = params.command.clone();
            (params, safety, command_for_display)
        }
    };

    let sandbox_type = match safety {
        SafetyCheck::AutoApprove { sandbox_type } => sandbox_type,
        SafetyCheck::AskUser => {
            let rx_approve = sess
                .request_command_approval(
                    sub_id.clone(),
                    call_id.clone(),
                    params.command.clone(),
                    params.cwd.clone(),
                    params.justification.clone(),
                )
                .await;
            match rx_approve.await.unwrap_or_default() {
                ReviewDecision::Approved => (),
                ReviewDecision::ApprovedForSession => {
                    sess.add_approved_command(params.command.clone());
                }
                ReviewDecision::Denied | ReviewDecision::Abort => {
                    return ResponseInputItem::FunctionCallOutput {
                        call_id,
                        output: FunctionCallOutputPayload {
                            content: "exec command rejected by user".to_string(),
                            success: None,
                        },
                    };
                }
            }
            // No sandboxing is applied because the user has given
            // explicit approval. Often, we end up in this case because
            // the command cannot be run in a sandbox, such as
            // installing a new dependency that requires network access.
            SandboxType::None
        }
        SafetyCheck::Reject { reason } => {
            return ResponseInputItem::FunctionCallOutput {
                call_id,
                output: FunctionCallOutputPayload {
                    content: format!("exec command rejected: {reason}"),
                    success: None,
                },
            };
        }
    };

    let exec_command_context = ExecCommandContext {
        sub_id: sub_id.clone(),
        call_id: call_id.clone(),
        command_for_display: command_for_display.clone(),
        cwd: params.cwd.clone(),
        apply_patch: apply_patch_exec.map(
            |ApplyPatchExec {
                 action,
                 user_explicitly_approved_this_action,
             }| ApplyPatchCommandContext {
                user_explicitly_approved_this_action,
                changes: convert_apply_patch_to_protocol(&action),
            },
        ),
    };

    let params = maybe_translate_shell_command(params, sess, turn_context);
    let output_result = sess
        .run_exec_with_events(
            turn_diff_tracker,
            exec_command_context.clone(),
            ExecInvokeArgs {
                params: params.clone(),
                sandbox_type,
                sandbox_policy: &turn_context.sandbox_policy,
                codex_linux_sandbox_exe: &sess.codex_linux_sandbox_exe,
                stdout_stream: if exec_command_context.apply_patch.is_some() {
                    None
                } else {
                    Some(StdoutStream {
                        sub_id: sub_id.clone(),
                        call_id: call_id.clone(),
                        tx_event: sess.tx_event.clone(),
                    })
                },
            },
        )
        .await;

    match output_result {
        Ok(output) => {
            let ExecToolCallOutput { exit_code, .. } = &output;

            let is_success = *exit_code == 0;
            let content = format_exec_output(&output);
            ResponseInputItem::FunctionCallOutput {
                call_id: call_id.clone(),
                output: FunctionCallOutputPayload {
                    content,
                    success: Some(is_success),
                },
            }
        }
        Err(CodexErr::Sandbox(error)) => {
            handle_sandbox_error(
                turn_diff_tracker,
                params,
                exec_command_context,
                error,
                sandbox_type,
                sess,
                turn_context,
            )
            .await
        }
        Err(e) => ResponseInputItem::FunctionCallOutput {
            call_id: call_id.clone(),
            output: FunctionCallOutputPayload {
                content: format!("execution error: {e}"),
                success: None,
            },
        },
    }
}

async fn handle_sandbox_error(
    turn_diff_tracker: &mut TurnDiffTracker,
    params: ExecParams,
    exec_command_context: ExecCommandContext,
    error: SandboxErr,
    sandbox_type: SandboxType,
    sess: &Session,
    turn_context: &TurnContext,
) -> ResponseInputItem {
    let call_id = exec_command_context.call_id.clone();
    let sub_id = exec_command_context.sub_id.clone();
    let cwd = exec_command_context.cwd.clone();

    if let SandboxErr::Timeout { output } = &error {
        let content = format_exec_output(output);
        return ResponseInputItem::FunctionCallOutput {
            call_id,
            output: FunctionCallOutputPayload {
                content,
                success: Some(false),
            },
        };
    }

    // Early out if either the user never wants to be asked for approval, or
    // we're letting the model manage escalation requests. Otherwise, continue
    match turn_context.approval_policy {
        AskForApproval::Never | AskForApproval::OnRequest => {
            return ResponseInputItem::FunctionCallOutput {
                call_id,
                output: FunctionCallOutputPayload {
                    content: format!(
                        "failed in sandbox {sandbox_type:?} with execution error: {error}"
                    ),
                    success: Some(false),
                },
            };
        }
        AskForApproval::UnlessTrusted | AskForApproval::OnFailure => (),
    }

    // Note that when `error` is `SandboxErr::Denied`, it could be a false
    // positive. That is, it may have exited with a non-zero exit code, not
    // because the sandbox denied it, but because that is its expected behavior,
    // i.e., a grep command that did not match anything. Ideally we would
    // include additional metadata on the command to indicate whether non-zero
    // exit codes merit a retry.

    // For now, we categorically ask the user to retry without sandbox and
    // emit the raw error as a background event.
    sess.notify_background_event(&sub_id, format!("Execution failed: {error}"))
        .await;

    let rx_approve = sess
        .request_command_approval(
            sub_id.clone(),
            call_id.clone(),
            params.command.clone(),
            cwd.clone(),
            Some("command failed; retry without sandbox?".to_string()),
        )
        .await;

    match rx_approve.await.unwrap_or_default() {
        ReviewDecision::Approved | ReviewDecision::ApprovedForSession => {
            // Persist this command as pre‑approved for the
            // remainder of the session so future
            // executions skip the sandbox directly.
            // TODO(ragona): Isn't this a bug? It always saves the command in an | fork?
            sess.add_approved_command(params.command.clone());
            // Inform UI we are retrying without sandbox.
            sess.notify_background_event(&sub_id, "retrying command without sandbox")
                .await;

            // This is an escalated retry; the policy will not be
            // examined and the sandbox has been set to `None`.
            let retry_output_result = sess
                .run_exec_with_events(
                    turn_diff_tracker,
                    exec_command_context.clone(),
                    ExecInvokeArgs {
                        params,
                        sandbox_type: SandboxType::None,
                        sandbox_policy: &turn_context.sandbox_policy,
                        codex_linux_sandbox_exe: &sess.codex_linux_sandbox_exe,
                        stdout_stream: if exec_command_context.apply_patch.is_some() {
                            None
                        } else {
                            Some(StdoutStream {
                                sub_id: sub_id.clone(),
                                call_id: call_id.clone(),
                                tx_event: sess.tx_event.clone(),
                            })
                        },
                    },
                )
                .await;

            match retry_output_result {
                Ok(retry_output) => {
                    let ExecToolCallOutput { exit_code, .. } = &retry_output;

                    let is_success = *exit_code == 0;
                    let content = format_exec_output(&retry_output);

                    ResponseInputItem::FunctionCallOutput {
                        call_id: call_id.clone(),
                        output: FunctionCallOutputPayload {
                            content,
                            success: Some(is_success),
                        },
                    }
                }
                Err(e) => ResponseInputItem::FunctionCallOutput {
                    call_id: call_id.clone(),
                    output: FunctionCallOutputPayload {
                        content: format!("retry failed: {e}"),
                        success: None,
                    },
                },
            }
        }
        ReviewDecision::Denied | ReviewDecision::Abort => {
            // Fall through to original failure handling.
            ResponseInputItem::FunctionCallOutput {
                call_id,
                output: FunctionCallOutputPayload {
                    content: "exec command rejected by user".to_string(),
                    success: None,
                },
            }
        }
    }
}

fn format_exec_output_str(exec_output: &ExecToolCallOutput) -> String {
    let ExecToolCallOutput {
        aggregated_output, ..
    } = exec_output;

    // Head+tail truncation for the model: show the beginning and end with an elision.
    // Clients still receive full streams; only this formatted summary is capped.

    let mut s = &aggregated_output.text;
    let prefixed_str: String;

    if exec_output.timed_out {
        prefixed_str = format!(
            "command timed out after {} milliseconds\n",
            exec_output.duration.as_millis()
        ) + s;
        s = &prefixed_str;
    }

    let total_lines = s.lines().count();
    if s.len() <= MODEL_FORMAT_MAX_BYTES && total_lines <= MODEL_FORMAT_MAX_LINES {
        return s.to_string();
    }

    let lines: Vec<&str> = s.lines().collect();
    let head_take = MODEL_FORMAT_HEAD_LINES.min(lines.len());
    let tail_take = MODEL_FORMAT_TAIL_LINES.min(lines.len().saturating_sub(head_take));
    let omitted = lines.len().saturating_sub(head_take + tail_take);

    // Join head and tail blocks (lines() strips newlines; reinsert them)
    let head_block = lines
        .iter()
        .take(head_take)
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");
    let tail_block = if tail_take > 0 {
        lines[lines.len() - tail_take..].join("\n")
    } else {
        String::new()
    };
    let marker = format!("\n[... omitted {omitted} of {total_lines} lines ...]\n\n");

    // Byte budgets for head/tail around the marker
    let mut head_budget = MODEL_FORMAT_HEAD_BYTES.min(MODEL_FORMAT_MAX_BYTES);
    let tail_budget = MODEL_FORMAT_MAX_BYTES.saturating_sub(head_budget + marker.len());
    if tail_budget == 0 && marker.len() >= MODEL_FORMAT_MAX_BYTES {
        // Degenerate case: marker alone exceeds budget; return a clipped marker
        return take_bytes_at_char_boundary(&marker, MODEL_FORMAT_MAX_BYTES).to_string();
    }
    if tail_budget == 0 {
        // Make room for the marker by shrinking head
        head_budget = MODEL_FORMAT_MAX_BYTES.saturating_sub(marker.len());
    }

    // Enforce line-count cap by trimming head/tail lines
    let head_lines_text = head_block;
    let tail_lines_text = tail_block;
    // Build final string respecting byte budgets
    let head_part = take_bytes_at_char_boundary(&head_lines_text, head_budget);
    let mut result = String::with_capacity(MODEL_FORMAT_MAX_BYTES.min(s.len()));

    result.push_str(head_part);
    result.push_str(&marker);

    let remaining = MODEL_FORMAT_MAX_BYTES.saturating_sub(result.len());
    let tail_budget_final = remaining;
    let tail_part = take_last_bytes_at_char_boundary(&tail_lines_text, tail_budget_final);
    result.push_str(tail_part);

    result
}

// Truncate a &str to a byte budget at a char boundary (prefix)
#[inline]
fn take_bytes_at_char_boundary(s: &str, maxb: usize) -> &str {
    if s.len() <= maxb {
        return s;
    }
    let mut last_ok = 0;
    for (i, ch) in s.char_indices() {
        let nb = i + ch.len_utf8();
        if nb > maxb {
            break;
        }
        last_ok = nb;
    }
    &s[..last_ok]
}

// Take a suffix of a &str within a byte budget at a char boundary
#[inline]
fn take_last_bytes_at_char_boundary(s: &str, maxb: usize) -> &str {
    if s.len() <= maxb {
        return s;
    }
    let mut start = s.len();
    let mut used = 0usize;
    for (i, ch) in s.char_indices().rev() {
        let nb = ch.len_utf8();
        if used + nb > maxb {
            break;
        }
        start = i;
        used += nb;
        if start == 0 {
            break;
        }
    }
    &s[start..]
}

/// Exec output is a pre-serialized JSON payload
fn format_exec_output(exec_output: &ExecToolCallOutput) -> String {
    let ExecToolCallOutput {
        exit_code,
        duration,
        ..
    } = exec_output;

    #[derive(Serialize)]
    struct ExecMetadata {
        exit_code: i32,
        duration_seconds: f32,
    }

    #[derive(Serialize)]
    struct ExecOutput<'a> {
        output: &'a str,
        metadata: ExecMetadata,
    }

    // round to 1 decimal place
    let duration_seconds = ((duration.as_secs_f32()) * 10.0).round() / 10.0;

    let formatted_output = format_exec_output_str(exec_output);

    let payload = ExecOutput {
        output: &formatted_output,
        metadata: ExecMetadata {
            exit_code: *exit_code,
            duration_seconds,
        },
    };

    #[expect(clippy::expect_used)]
    serde_json::to_string(&payload).expect("serialize ExecOutput")
}

pub(super) fn get_last_assistant_message_from_turn(responses: &[ResponseItem]) -> Option<String> {
    responses.iter().rev().find_map(|item| {
        if let ResponseItem::Message { role, content, .. } = item {
            if role == "assistant" {
                content.iter().rev().find_map(|ci| {
                    if let ContentItem::OutputText { text } = ci {
                        Some(text.clone())
                    } else {
                        None
                    }
                })
            } else {
                None
            }
        } else {
            None
        }
    })
}
fn convert_call_tool_result_to_function_call_output_payload(
    call_tool_result: &CallToolResult,
) -> FunctionCallOutputPayload {
    let CallToolResult {
        content,
        is_error,
        structured_content,
    } = call_tool_result;

    // In terms of what to send back to the model, we prefer structured_content,
    // if available, and fallback to content, otherwise.
    let mut is_success = is_error != &Some(true);
    let content = if let Some(structured_content) = structured_content
        && structured_content != &serde_json::Value::Null
        && let Ok(serialized_structured_content) = serde_json::to_string(&structured_content)
    {
        serialized_structured_content
    } else {
        match serde_json::to_string(&content) {
            Ok(serialized_content) => serialized_content,
            Err(err) => {
                // If we could not serialize either content or structured_content to
                // JSON, flag this as an error.
                is_success = false;
                err.to_string()
            }
        }
    };

    FunctionCallOutputPayload {
        content,
        success: Some(is_success),
    }
}

/// Emits an ExitedReviewMode Event with optional ReviewOutput.
async fn exit_review_mode(
    session: Arc<Session>,
    task_sub_id: String,
    review_output: Option<ReviewOutputEvent>,
) {
    let event = Event {
        id: task_sub_id,
        msg: EventMsg::ExitedReviewMode(ExitedReviewModeEvent { review_output }),
    };
    session.send_event(event).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ConfigOverrides;
    use crate::config::ConfigToml;
    use crate::plan_mode::PlanModeConfig;
    use crate::protocol::CompactedItem;
    use crate::protocol::InitialHistory;
    use crate::protocol::ResumedHistory;
    use codex_protocol::models::ContentItem;
    use mcp_types::ContentBlock;
    use mcp_types::TextContent;
    use pretty_assertions::assert_eq;
    use serde_json::json;
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::time::Duration as StdDuration;
    use uuid::Uuid;

    #[test]
    fn ensure_plan_mode_prompt_records_once_per_activation() {
        let (session, turn_context) = make_session_and_context();

        {
            let mut state = session.state.lock_unchecked();
            state.plan_mode = Some(PlanModeSession::new(
                Uuid::new_v4(),
                turn_context.approval_policy,
                Vec::new(),
                &PlanModeConfig::default(),
                true,
            ));
            state.plan_mode_prompt_recorded = false;
        }

        {
            let state = session.state.lock_unchecked();
            assert!(state.plan_mode.is_some());
            assert!(!state.plan_mode_prompt_recorded);
        }

        tokio_test::block_on(session.ensure_plan_mode_prompt_recorded());
        {
            let state = session.state.lock_unchecked();
            assert!(state.plan_mode_prompt_recorded);
        }
        let items = session.turn_input_with_history(Vec::new());
        assert!(!items.is_empty());
        assert_plan_prompt(&items[0]);

        let items = session.turn_input_with_history(Vec::new());
        assert!(!items.is_empty());
        assert_plan_prompt(&items[0]);

        session
            .try_exit_plan_mode()
            .expect("exit plan mode should succeed");

        {
            let mut state = session.state.lock_unchecked();
            state.plan_mode = Some(PlanModeSession::new(
                Uuid::new_v4(),
                turn_context.approval_policy,
                Vec::new(),
                &PlanModeConfig::default(),
                true,
            ));
            state.plan_mode_prompt_recorded = false;
        }

        tokio_test::block_on(session.ensure_plan_mode_prompt_recorded());
        let items = session.turn_input_with_history(Vec::new());
        assert!(!items.is_empty());
        assert_plan_prompt(&items[0]);
    }

    #[test]
    fn ensure_plan_mode_prompt_noop_without_plan_mode() {
        let (session, _turn_context) = make_session_and_context();

        tokio_test::block_on(session.ensure_plan_mode_prompt_recorded());

        let items = session.turn_input_with_history(Vec::new());
        assert!(items.is_empty(), "expected no plan prompt to be included");
    }

    #[test]
    fn reconstruct_history_matches_live_compactions() {
        let (session, turn_context) = make_session_and_context();
        let (rollout_items, expected) = sample_rollout(&session, &turn_context);

        let reconstructed = session.reconstruct_history_from_rollout(&turn_context, &rollout_items);

        assert_eq!(expected, reconstructed);
    }

    #[test]
    fn record_initial_history_reconstructs_resumed_transcript() {
        let (session, turn_context) = make_session_and_context();
        let (rollout_items, expected) = sample_rollout(&session, &turn_context);

        tokio_test::block_on(session.record_initial_history(
            &turn_context,
            InitialHistory::Resumed(ResumedHistory {
                conversation_id: ConversationId::default(),
                history: rollout_items,
                rollout_path: PathBuf::from("/tmp/resume.jsonl"),
            }),
        ));

        let actual = session.state.lock_unchecked().history.contents();
        assert_eq!(expected, actual);
    }

    #[test]
    fn record_initial_history_reconstructs_forked_transcript() {
        let (session, turn_context) = make_session_and_context();
        let (rollout_items, expected) = sample_rollout(&session, &turn_context);

        tokio_test::block_on(
            session.record_initial_history(&turn_context, InitialHistory::Forked(rollout_items)),
        );

        let actual = session.state.lock_unchecked().history.contents();
        assert_eq!(expected, actual);
    }

    #[test]
    fn prefers_structured_content_when_present() {
        let ctr = CallToolResult {
            // Content present but should be ignored because structured_content is set.
            content: vec![text_block("ignored")],
            is_error: None,
            structured_content: Some(json!({
                "ok": true,
                "value": 42
            })),
        };

        let got = convert_call_tool_result_to_function_call_output_payload(&ctr);
        let expected = FunctionCallOutputPayload {
            content: serde_json::to_string(&json!({
                "ok": true,
                "value": 42
            }))
            .unwrap(),
            success: Some(true),
        };

        assert_eq!(expected, got);
    }

    #[test]
    fn model_truncation_head_tail_by_lines() {
        // Build 400 short lines so line-count limit, not byte budget, triggers truncation
        let lines: Vec<String> = (1..=400).map(|i| format!("line{i}")).collect();
        let full = lines.join("\n");

        let exec = ExecToolCallOutput {
            exit_code: 0,
            stdout: StreamOutput::new(String::new()),
            stderr: StreamOutput::new(String::new()),
            aggregated_output: StreamOutput::new(full),
            duration: StdDuration::from_secs(1),
            timed_out: false,
        };

        let out = format_exec_output_str(&exec);

        // Expect elision marker with correct counts
        let omitted = 400 - MODEL_FORMAT_MAX_LINES; // 144
        let marker = format!("\n[... omitted {omitted} of 400 lines ...]\n\n");
        assert!(out.contains(&marker), "missing marker: {out}");

        // Validate head and tail
        let parts: Vec<&str> = out.split(&marker).collect();
        assert_eq!(parts.len(), 2, "expected one marker split");
        let head = parts[0];
        let tail = parts[1];

        let expected_head: String = (1..=MODEL_FORMAT_HEAD_LINES)
            .map(|i| format!("line{i}"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(head.starts_with(&expected_head), "head mismatch");

        let expected_tail: String = ((400 - MODEL_FORMAT_TAIL_LINES + 1)..=400)
            .map(|i| format!("line{i}"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(tail.ends_with(&expected_tail), "tail mismatch");
    }

    #[test]
    fn model_truncation_respects_byte_budget() {
        // Construct a large output (about 100kB) so byte budget dominates
        let big_line = "x".repeat(100);
        let full = std::iter::repeat_n(big_line, 1000)
            .collect::<Vec<_>>()
            .join("\n");

        let exec = ExecToolCallOutput {
            exit_code: 0,
            stdout: StreamOutput::new(String::new()),
            stderr: StreamOutput::new(String::new()),
            aggregated_output: StreamOutput::new(full.clone()),
            duration: StdDuration::from_secs(1),
            timed_out: false,
        };

        let out = format_exec_output_str(&exec);
        assert!(out.len() <= MODEL_FORMAT_MAX_BYTES, "exceeds byte budget");
        assert!(out.contains("omitted"), "should contain elision marker");

        // Ensure head and tail are drawn from the original
        assert!(full.starts_with(out.chars().take(8).collect::<String>().as_str()));
        assert!(
            full.ends_with(
                out.chars()
                    .rev()
                    .take(8)
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect::<String>()
                    .as_str()
            )
        );
    }

    #[test]
    fn includes_timed_out_message() {
        let exec = ExecToolCallOutput {
            exit_code: 0,
            stdout: StreamOutput::new(String::new()),
            stderr: StreamOutput::new(String::new()),
            aggregated_output: StreamOutput::new("Command output".to_string()),
            duration: StdDuration::from_secs(1),
            timed_out: true,
        };

        let out = format_exec_output_str(&exec);

        assert_eq!(
            out,
            "command timed out after 1000 milliseconds\nCommand output"
        );
    }

    #[test]
    fn falls_back_to_content_when_structured_is_null() {
        let ctr = CallToolResult {
            content: vec![text_block("hello"), text_block("world")],
            is_error: None,
            structured_content: Some(serde_json::Value::Null),
        };

        let got = convert_call_tool_result_to_function_call_output_payload(&ctr);
        let expected = FunctionCallOutputPayload {
            content: serde_json::to_string(&vec![text_block("hello"), text_block("world")])
                .unwrap(),
            success: Some(true),
        };

        assert_eq!(expected, got);
    }

    #[test]
    fn success_flag_reflects_is_error_true() {
        let ctr = CallToolResult {
            content: vec![text_block("unused")],
            is_error: Some(true),
            structured_content: Some(json!({ "message": "bad" })),
        };

        let got = convert_call_tool_result_to_function_call_output_payload(&ctr);
        let expected = FunctionCallOutputPayload {
            content: serde_json::to_string(&json!({ "message": "bad" })).unwrap(),
            success: Some(false),
        };

        assert_eq!(expected, got);
    }

    #[test]
    fn success_flag_true_with_no_error_and_content_used() {
        let ctr = CallToolResult {
            content: vec![text_block("alpha")],
            is_error: Some(false),
            structured_content: None,
        };

        let got = convert_call_tool_result_to_function_call_output_payload(&ctr);
        let expected = FunctionCallOutputPayload {
            content: serde_json::to_string(&vec![text_block("alpha")]).unwrap(),
            success: Some(true),
        };

        assert_eq!(expected, got);
    }

    fn text_block(s: &str) -> ContentBlock {
        ContentBlock::TextContent(TextContent {
            annotations: None,
            text: s.to_string(),
            r#type: "text".to_string(),
        })
    }

    fn make_session_and_context() -> (Session, TurnContext) {
        let (tx_event, _rx_event) = async_channel::unbounded();
        let codex_home = tempfile::tempdir().expect("create temp dir");
        let config = Config::load_from_base_config_with_overrides(
            ConfigToml::default(),
            ConfigOverrides::default(),
            codex_home.path().to_path_buf(),
        )
        .expect("load default test config");
        let config = Arc::new(config);
        let conversation_id = ConversationId::default();
        let client = ModelClient::new(
            config.clone(),
            None,
            config.model_provider.clone(),
            config.model_reasoning_effort,
            config.model_reasoning_summary,
            conversation_id,
        );
        let tools_config = ToolsConfig::new(&ToolsConfigParams {
            model_family: &config.model_family,
            approval_policy: config.approval_policy,
            sandbox_policy: config.sandbox_policy.clone(),
            include_plan_tool: config.include_plan_tool,
            include_apply_patch_tool: config.include_apply_patch_tool,
            include_web_search_request: config.tools_web_search_request,
            use_streamable_shell_tool: config.use_experimental_streamable_shell_tool,
            include_view_image_tool: config.include_view_image_tool,
            experimental_unified_exec_tool: config.use_experimental_unified_exec_tool,
        });
        let (subagent_inventory, subagent_tool, subagent_config) =
            compute_subagent_tooling(config.as_ref());
        let user_instructions = merge_subagent_user_instructions(
            config.user_instructions.clone(),
            subagent_inventory.as_deref(),
        );
        let turn_context = TurnContext {
            client,
            cwd: config.cwd.clone(),
            base_instructions: config.base_instructions.clone(),
            user_instructions,
            approval_policy: config.approval_policy,
            sandbox_policy: config.sandbox_policy.clone(),
            shell_environment_policy: config.shell_environment_policy.clone(),
            tools_config,
            is_review_mode: false,
            subagent_inventory,
            subagent_tool,
            subagent_config,
        };
        let session = Session {
            conversation_id,
            tx_event,
            mcp_connection_manager: McpConnectionManager::default(),
            session_manager: ExecSessionManager::default(),
            unified_exec_manager: UnifiedExecSessionManager::default(),
            notify: None,
            hook_executor: HookExecutor::default(),
            rollout: Mutex::new(None),
            state: Mutex::new(State {
                history: ConversationHistory::new(),
                ..Default::default()
            }),
            codex_linux_sandbox_exe: None,
            user_shell: shell::Shell::Unknown,
            show_raw_agent_reasoning: config.show_raw_agent_reasoning,
            #[cfg(feature = "slash_commands")]
            slash_commands: None,
        };
        (session, turn_context)
    }

    fn assert_plan_prompt(item: &ResponseItem) {
        match item {
            ResponseItem::Message { role, content, .. } => {
                assert_eq!(role, "user");
                assert!(content.iter().any(|ci| {
                    matches!(
                        ci,
                        ContentItem::InputText { text }
                        if text == PLAN_MODE_SYSTEM_PROMPT
                    )
                }));
            }
            other => panic!("expected plan mode prompt, got {other:?}"),
        }
    }

    fn sample_rollout(
        session: &Session,
        turn_context: &TurnContext,
    ) -> (Vec<RolloutItem>, Vec<ResponseItem>) {
        let mut rollout_items = Vec::new();
        let mut live_history = ConversationHistory::new();

        let initial_context = session.build_initial_context(turn_context);
        for item in &initial_context {
            rollout_items.push(RolloutItem::ResponseItem(item.clone()));
        }
        live_history.record_items(initial_context.iter());

        let user1 = ResponseItem::Message {
            id: None,
            role: "user".to_string(),
            content: vec![ContentItem::InputText {
                text: "first user".to_string(),
            }],
        };
        live_history.record_items(std::iter::once(&user1));
        rollout_items.push(RolloutItem::ResponseItem(user1.clone()));

        let assistant1 = ResponseItem::Message {
            id: None,
            role: "assistant".to_string(),
            content: vec![ContentItem::OutputText {
                text: "assistant reply one".to_string(),
            }],
        };
        live_history.record_items(std::iter::once(&assistant1));
        rollout_items.push(RolloutItem::ResponseItem(assistant1.clone()));

        let summary1 = "summary one";
        let snapshot1 = live_history.contents();
        let user_messages1 = collect_user_messages(&snapshot1);
        let rebuilt1 = build_compacted_history(
            session.build_initial_context(turn_context),
            &user_messages1,
            summary1,
        );
        live_history.replace(rebuilt1);
        rollout_items.push(RolloutItem::Compacted(CompactedItem {
            message: summary1.to_string(),
        }));

        let user2 = ResponseItem::Message {
            id: None,
            role: "user".to_string(),
            content: vec![ContentItem::InputText {
                text: "second user".to_string(),
            }],
        };
        live_history.record_items(std::iter::once(&user2));
        rollout_items.push(RolloutItem::ResponseItem(user2.clone()));

        let assistant2 = ResponseItem::Message {
            id: None,
            role: "assistant".to_string(),
            content: vec![ContentItem::OutputText {
                text: "assistant reply two".to_string(),
            }],
        };
        live_history.record_items(std::iter::once(&assistant2));
        rollout_items.push(RolloutItem::ResponseItem(assistant2.clone()));

        let summary2 = "summary two";
        let snapshot2 = live_history.contents();
        let user_messages2 = collect_user_messages(&snapshot2);
        let rebuilt2 = build_compacted_history(
            session.build_initial_context(turn_context),
            &user_messages2,
            summary2,
        );
        live_history.replace(rebuilt2);
        rollout_items.push(RolloutItem::Compacted(CompactedItem {
            message: summary2.to_string(),
        }));

        let user3 = ResponseItem::Message {
            id: None,
            role: "user".to_string(),
            content: vec![ContentItem::InputText {
                text: "third user".to_string(),
            }],
        };
        live_history.record_items(std::iter::once(&user3));
        rollout_items.push(RolloutItem::ResponseItem(user3.clone()));

        let assistant3 = ResponseItem::Message {
            id: None,
            role: "assistant".to_string(),
            content: vec![ContentItem::OutputText {
                text: "assistant reply three".to_string(),
            }],
        };
        live_history.record_items(std::iter::once(&assistant3));
        rollout_items.push(RolloutItem::ResponseItem(assistant3.clone()));

        (rollout_items, live_history.contents())
    }
}
