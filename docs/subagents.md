# Subagents

Subagents let you delegate focused tasks to reusable playbooks defined in Markdown. Codex discovers
agent files from the current workspace and your global `~/.codex` directory, then surfaces them
through both the CLI and the interactive TUI.

## Enable the feature

Add the `subagents` section to your Codex configuration. The feature is off by default, so you must
set `enabled = true` and choose the discovery mode.

```toml
[subagents]
enabled = true
discovery = "auto"      # or "manual"
default_model = "gpt-4.1-mini"
```

- `discovery = "auto"` keeps Codex's inline hints enabled; `"manual"` avoids additional hints but
  the `invoke_subagent` tool remains available in both modes.
- `default_model` is used when a subagent file omits a `model` field.

## Define subagents

Create Markdown files under either `.codex/agents/` inside your repository or `~/.codex/agents/` for
machine-wide defaults. Project definitions override user definitions when names collide.

```markdown
---
name: code-reviewer
description: Reviews staged diffs for safety regressions
model: gpt-4.1-mini
tools:
  - git_diff
  - tests
---

Provide the playbook instructions here. Codex will render the description in listings and reuse the
body when invoking the agent.
```

## CLI workflow

Use the dedicated `codex agents` commands to inspect and execute subagents.

| Command | Description |
| --- | --- |
| `codex agents list` | Print discovered agents, showing scope (`project` or `user`) and whether a project file overrides a user file. Use `--invalid` to inspect definitions that failed validation. |
| `codex agents show <name>` | Display the normalized record, including description, tools, model, status, and source path. |
| `codex agents run <name>` | Invoke an agent immediately. The CLI prints the resolved model, requested tools, a short summary, and a detail URI such as `agents://code-reviewer/sessions/latest`. |

All commands accept `--json` to emit machine‑readable output, making it easy to script preflight
checks or visualize inventories.

## Invoking via tool calling

Every turn exposes an `invoke_subagent` tool to the model. Codex serializes the discovered
inventory—including scope, tools, model, and source path—into the tool description so the model can
decide whether delegation is appropriate. The function accepts the following arguments:

```json
{
  "name": "code-reviewer",
  "instructions": "Explain the changes in docs/",
  "requested_tools": ["git_diff"],
  "model": "gpt-4.1-mini"
}
```

Codex confirms that the subagent exists, applies tool/model restrictions, and records a summary plus
detail URI (for example `agents://code-reviewer/sessions/latest`). The summary is sent back to the
model so it can incorporate the result into the broader conversation.

Because invocations now flow through the tool pipeline, the TUI no longer interrupts the transcript
with keyword-based suggestions. Instead, results appear inline—either through the assistant's
follow-up message or via the `codex agents` CLI commands.

## Invocation summaries and detail links

Successful runs capture:

- The normalized subagent name and scope (including override state).
- The resolved model and the tools that were allowed or requested.
- A synthesized summary of the run.
- `detail_artifacts` URIs (`agents://...`) that callers can follow to inspect the full transcript.

These details appear in both the `codex agents run` CLI output and the TUI history cell so you can
jump into the manager UI or a later review workflow.

## Observability and precedence logging

Codex emits structured logs under the `codex::subagents` target covering discovery events,
override decisions, tool invocations, and failures. Enable tracing for that target if you need to
audit which definition ran or why a call was rejected.
