# Contract: Subagent Invocation

## Purpose
Invoke a subagent by name within the Codex CLI session and return summarized output with restricted tools.

## Request
- CLI form: natural-language prompt containing "Use the {name} subagent" OR explicit command `codex agents run {name} [--tool=...]`.
- Preconditions:
  - Subagent exists and is `active`.
  - Invoking session acknowledges suggested subagent when auto-suggestion is used.

## Workflow
1. Validate subagent existence and `status`.
2. Establish `InvocationSession` cloning parent session metadata.
3. Apply `effective_model` and tool allowlist.
4. Execute instruction set and capture transcript.
5. Emit summary to parent session and persist detail to manager history.

## Response Schema
```json
{
  "name": "code-reviewer",
  "summary": "Reviewed 3 files; flagged 1 high-risk issue.",
  "detail_ref": "agents://code-reviewer/sessions/2025-09-21T17:05:00Z",
  "tools_used": ["git_diff"],
  "model": "gpt-4.1-mini"
}
```

## Failure Modes
- Unknown subagent → message: "No subagent named 'code-reviewer'. Run `codex agents list`."
- Validation error → message: "Subagent 'code-reviewer' invalid: missing description field."
- Restricted tool denial → message: "Tool 'filesystem' not allowed for subagent 'code-reviewer'."

## Contract Tests
- `invoke_applies_tool_allowlist`: Expect denied operation when tool not allowed.
- `invoke_uses_configured_model`: Expect effective model equals definition or fallback order.
- `invoke_requires_confirmation_for_auto_suggest`: Auto-suggest path should not execute without user confirmation.
