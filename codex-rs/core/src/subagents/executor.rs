use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::Utc;
use serde::Serialize;

use crate::AuthManager;
use crate::codex::{Codex, CodexSpawnOk};
use crate::config::Config;
use crate::model_family::find_family_for_model;
use crate::openai_model_info::get_model_info;
use crate::subagents::config::SubagentConfig;
use crate::subagents::invocation::InvocationSession;
use crate::subagents::runner::{PreparedSubagentInvocation, SubagentInvocationError};
use codex_protocol::protocol::{Event, EventMsg, InitialHistory, InputItem, Op};

#[derive(Serialize)]
struct TranscriptPayload {
    subagent: String,
    timestamp: String,
    model: Option<String>,
    requested_tools: Vec<String>,
    instructions: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    extra_instructions: Option<String>,
    events: Vec<Event>,
}

fn storage_dir(record_path: &Path) -> PathBuf {
    let base = record_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("sessions")
}

fn persist_transcript(
    record_path: &Path,
    payload: &TranscriptPayload,
) -> Result<PathBuf, SubagentInvocationError> {
    let dir = storage_dir(record_path);
    fs::create_dir_all(&dir).map_err(|err| {
        SubagentInvocationError::ExecutionFailed(format!(
            "failed to prepare transcript directory: {err}"
        ))
    })?;

    let timestamp = &payload.timestamp;
    let artifact_path = dir.join(format!("{timestamp}.json"));
    let body = serde_json::to_vec_pretty(payload).map_err(|err| {
        SubagentInvocationError::ExecutionFailed(format!("failed to serialize transcript: {err}"))
    })?;
    fs::write(&artifact_path, &body).map_err(|err| {
        SubagentInvocationError::ExecutionFailed(format!("failed to write transcript: {err}"))
    })?;

    let latest = dir.join("latest.json");
    fs::write(&latest, &body).map_err(|err| {
        SubagentInvocationError::ExecutionFailed(format!(
            "failed to update latest transcript: {err}"
        ))
    })?;

    Ok(artifact_path)
}

fn format_instruction_block(
    definition_instructions: &str,
    extra: Option<&str>,
    record: &PreparedSubagentInvocation,
) -> String {
    let mut instructions = String::from(definition_instructions.trim());
    if let Some(extra) = extra {
        let extra = extra.trim();
        if !extra.is_empty() {
            instructions.push_str("\n\nAdditional request:\n");
            instructions.push_str(extra);
        }
    }

    if !record.record.effective_tools.is_empty() {
        instructions.push_str("\n\nAllowed tools: ");
        instructions.push_str(&record.record.effective_tools.join(", "));
    }

    instructions
}

fn artifact_uri(name: &str) -> PathBuf {
    PathBuf::from(format!("agents://{name}/sessions/latest"))
}

fn capture_agent_output(
    msg: &EventMsg,
    message_buffer: &mut Option<String>,
    last_message: &mut Option<String>,
) -> bool {
    match msg {
        EventMsg::AgentMessage(agent) => {
            *message_buffer = Some(agent.message.clone());
            *last_message = message_buffer.clone();
            false
        }
        EventMsg::AgentMessageDelta(delta) => {
            let buffer = message_buffer.get_or_insert_with(String::new);
            buffer.push_str(&delta.delta);
            *last_message = Some(buffer.clone());
            false
        }
        EventMsg::TaskComplete(result) => {
            if let Some(message) = result
                .last_agent_message
                .clone()
                .or_else(|| message_buffer.clone())
            {
                *last_message = Some(message);
            }
            true
        }
        _ => false,
    }
}

async fn spawn_subagent_codex(
    mut config: Config,
    auth_manager: Arc<AuthManager>,
) -> Result<CodexSpawnOk, SubagentInvocationError> {
    config.subagents = SubagentConfig::disabled();
    Codex::spawn(config, auth_manager, InitialHistory::New)
        .await
        .map_err(|err| SubagentInvocationError::ExecutionFailed(err.to_string()))
}

/// Execute a prepared subagent invocation by spawning an isolated Codex
/// conversation and returning the populated session state with real outputs.
pub async fn execute_subagent_invocation(
    base_config: &Config,
    auth_manager: Arc<AuthManager>,
    mut prepared: PreparedSubagentInvocation,
) -> Result<InvocationSession, SubagentInvocationError> {
    let record = prepared.record.clone();
    let instructions = record.definition.instructions.trim();
    if instructions.is_empty() {
        return Err(SubagentInvocationError::InvalidSubagent(
            record.definition.name.clone(),
        ));
    }

    let resolved_model = prepared
        .session
        .resolved_model
        .clone()
        .or_else(|| record.effective_model.clone())
        .or_else(|| base_config.model.clone().into());

    let mut config = base_config.clone();
    if let Some(model) = resolved_model.clone() {
        config.model = model.clone();
        if let Some(family) = find_family_for_model(&model) {
            config.model_family = family.clone();
            if let Some(info) = get_model_info(&family) {
                config.model_context_window = Some(info.context_window);
            }
        }
    }

    let CodexSpawnOk { codex, session, .. } =
        spawn_subagent_codex(config.clone(), auth_manager.clone()).await?;

    // The first event must be SessionConfigured; record it for the transcript.
    let mut transcript: Vec<Event> = Vec::new();
    match codex.next_event().await {
        Ok(event) => transcript.push(event),
        Err(err) => {
            return Err(SubagentInvocationError::ExecutionFailed(format!(
                "failed to initialize subagent session: {err}"
            )));
        }
    }

    let instruction_block = format_instruction_block(
        instructions,
        prepared.session.extra_instructions.as_deref(),
        &prepared,
    );
    let submit_id = codex
        .submit(Op::UserTurn {
            items: vec![InputItem::Text {
                text: instruction_block.clone(),
            }],
            cwd: config.cwd.clone(),
            approval_policy: config.approval_policy,
            sandbox_policy: config.sandbox_policy.clone(),
            model: config.model.clone(),
            effort: config.model_reasoning_effort,
            summary: config.model_reasoning_summary,
        })
        .await
        .map_err(|err| SubagentInvocationError::ExecutionFailed(err.to_string()))?;

    let mut last_message: Option<String> = None;
    let mut message_buffer: Option<String> = None;
    loop {
        let event = codex
            .next_event()
            .await
            .map_err(|err| SubagentInvocationError::ExecutionFailed(err.to_string()))?;
        if event.id != submit_id {
            continue;
        }

        let error_message = if let EventMsg::Error(err) = &event.msg {
            Some(err.message.clone())
        } else {
            None
        };
        let should_break = capture_agent_output(&event.msg, &mut message_buffer, &mut last_message);
        transcript.push(event);
        if let Some(message) = error_message {
            return Err(SubagentInvocationError::ExecutionFailed(message));
        }
        if should_break {
            break;
        }
    }

    // Persist transcript before shutting down the session.
    let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let instruction_preview = instruction_block
        .lines()
        .take(4)
        .collect::<Vec<&str>>()
        .join("\n");

    let payload = TranscriptPayload {
        subagent: prepared.session.subagent_name.clone(),
        timestamp: timestamp.clone(),
        model: resolved_model.clone(),
        requested_tools: prepared.session.requested_tools.clone(),
        instructions: instruction_block,
        extra_instructions: prepared.session.extra_instructions.clone(),
        events: transcript.clone(),
    };

    let artifact_path = persist_transcript(&record.definition.source_path, &payload)?;

    // Attempt graceful shutdown; ignore errors.
    let _ = codex.submit(Op::Shutdown).await;
    session.notify_session_end().await;

    prepared.session.summary = last_message.or_else(|| {
        Some(format!(
            "Subagent '{}' completed without returning a final message.",
            prepared.session.subagent_name
        ))
    });
    prepared.session.detail_artifacts = vec![artifact_uri(&prepared.session.subagent_name)];
    prepared.session.resolved_model = resolved_model;
    prepared
        .session
        .execution_log
        .push(format!("instructions: {}", instruction_preview));
    prepared
        .session
        .execution_log
        .push(format!("transcript saved to {}", artifact_path.display()));

    Ok(prepared.session)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::subagents::config::{SubagentConfig, SubagentDiscoveryMode};
    use crate::subagents::definition::{SubagentDefinition, SubagentScope};
    use crate::subagents::record::SubagentRecord;
    use codex_protocol::protocol::{
        AgentMessageDeltaEvent, AgentMessageEvent, EventMsg, TaskCompleteEvent,
    };
    use std::path::PathBuf;

    #[test]
    fn format_instruction_block_adds_extra_request() {
        let mut definition = SubagentDefinition::new(
            "code-reviewer",
            "Reviews staged diffs",
            SubagentScope::Project,
            PathBuf::from("/tmp/code-reviewer.md"),
        );
        definition.instructions = "Review the staged diffs.".into();
        definition.tools = vec!["git_diff".into()];

        let config = SubagentConfig::enabled(SubagentDiscoveryMode::Auto);
        let record = SubagentRecord::from_definition(definition.clone(), &config);

        let mut session = InvocationSession::new(&definition.name).confirmed();
        session.extra_instructions = Some("  Focus on the docs folder  ".into());

        let prepared = PreparedSubagentInvocation {
            session: session.clone(),
            record,
        };

        let block = format_instruction_block(
            &definition.instructions,
            session.extra_instructions.as_deref(),
            &prepared,
        );

        assert!(
            block.contains("Additional request:\nFocus on the docs folder"),
            "{block}"
        );
        assert!(block.contains("Allowed tools: git_diff"), "{block}");
    }

    #[test]
    fn capture_agent_output_accumulates_deltas() {
        let mut buffer = None;
        let mut last = None;

        let should_break = capture_agent_output(
            &EventMsg::AgentMessageDelta(AgentMessageDeltaEvent {
                delta: "Part 1 ".into(),
            }),
            &mut buffer,
            &mut last,
        );
        assert!(!should_break);
        assert_eq!(last.as_deref(), Some("Part 1 "));

        let should_break = capture_agent_output(
            &EventMsg::AgentMessageDelta(AgentMessageDeltaEvent {
                delta: "and 2".into(),
            }),
            &mut buffer,
            &mut last,
        );
        assert!(!should_break);
        assert_eq!(last.as_deref(), Some("Part 1 and 2"));

        let should_break = capture_agent_output(
            &EventMsg::TaskComplete(TaskCompleteEvent {
                last_agent_message: None,
            }),
            &mut buffer,
            &mut last,
        );
        assert!(should_break);
        assert_eq!(last.as_deref(), Some("Part 1 and 2"));
    }

    #[test]
    fn capture_agent_output_prefers_task_complete_message() {
        let mut buffer = Some("stale".into());
        let mut last = buffer.clone();

        let should_break = capture_agent_output(
            &EventMsg::TaskComplete(TaskCompleteEvent {
                last_agent_message: Some("final".into()),
            }),
            &mut buffer,
            &mut last,
        );
        assert!(should_break);
        assert_eq!(buffer.as_deref(), Some("stale"));
        assert_eq!(last.as_deref(), Some("final"));

        let mut buffer = None;
        let mut last = None;
        let should_break = capture_agent_output(
            &EventMsg::AgentMessage(AgentMessageEvent {
                message: "complete message".into(),
            }),
            &mut buffer,
            &mut last,
        );
        assert!(!should_break);
        assert_eq!(last.as_deref(), Some("complete message"));
    }
}
