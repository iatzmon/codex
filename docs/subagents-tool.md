# Subagents Tool Migration

Codex now exposes subagents through a dedicated `invoke_subagent` tool instead of relying on
keyword heuristics. This document summarizes the resulting workflow and guardrails.

## Tool registration

- Each turn builds the merged project/user inventory and serializes it into the tool description.
- The function schema accepts `name`, optional `instructions`, optional `requested_tools`, and an
  optional `model` override.
- Discovery mode (`auto` vs `manual`) is reflected in the description but the tool remains
  available in both cases.

## Invocation semantics

- Tool calls are validated against the inventory and `SubagentRunner`; errors return structured
  JSON with `success = false` so the model can retry or back off.
- Successful calls auto-confirm, enforce tool allowlists, resolve models, and emit summary/detail
  metadata (`agents://...` URIs).
- The JSON response is streamed back to the model, allowing it to incorporate the summary directly
  into the conversation.

## UX updates

- The TUI no longer surfaces keyword-based suggestion prompts; results appear in-line with the
  conversation or via `codex agents` commands.
- CLI behaviour is unchanged: `codex agents list`, `run`, and `show` remain the primary entry
  points for manual inspection and invocation.

## Observability and tests

- Structured logs under `codex::subagents` cover discovery, overrides, tool invocations, and
  failures.
- Unit and integration tests assert tool registration, invocation error paths, and precedence.
