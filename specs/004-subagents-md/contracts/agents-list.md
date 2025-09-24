# Contract: `codex agents list`

## Purpose
Enumerate discovered subagents with scope precedence and validation state.

## Request
- Command: `codex agents list`
- Prerequisites: `subagents.enabled = true`
- Options:
  - `--scope [project|user|all]` (default `all`)
  - `--invalid` (show definitions with validation errors)

## Response Schema
```json
{
  "subagents": [
    {
      "name": "code-reviewer",
      "display_name": "code-reviewer",
      "scope": "project",
      "description": "Reviews staged diffs for safety regressions",
      "tools": ["git_diff", "tests"],
      "model": "gpt-4.1-mini",
      "status": "active",
      "overrides": true,
      "source_path": "/abs/path/to/.codex/agents/code-reviewer.md"
    }
  ],
  "conflicts": [
    {
      "name": "code-reviewer",
      "losing_scope": "user",
      "reason": "project override"
    }
  ]
}
```

## Failure Modes
- Feature disabled → exit code 1, message: "Subagents feature is disabled. Set subagents.enabled = true."
- Unreadable agent file → exit code 2, message surfaces path and parse error.

## Contract Tests
- `agents_list_returns_project_override_first`: Expect override flag when both scopes define same name.
- `agents_list_filters_invalid_definitions`: With `--invalid`, list only invalid entries.
