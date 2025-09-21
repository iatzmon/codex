# Codex Hooks CLI Contract

## Overview
Namespace `codex hooks` exposes read-only inspection and validation capabilities for lifecycle hooks. All commands must be available in interactive and headless modes.

## Commands

### `codex hooks list`
- **Description**: Print active hooks grouped by event and precedence.
- **Options**:
  - `--event <name>`: filter to a specific lifecycle event.
  - `--scope <managed|project|local>`: filter by configuration layer.
  - `--json`: emit machine-readable JSON payload.
- **Output (text)**:
  ```
  Event        Scope    ID                Command                      Decision
  PreToolUse   managed  audit.pre_shell   /usr/bin/codex-audit --check ask
  PreToolUse   project  project.lint      ./hooks/shell_guard.sh        allow
  ```
- **Output (JSON)**: matches `HookRegistry` snapshot schema.
- **Failure Modes**:
  - Exit `3` when configuration parse failures exist; include reasons on stderr.
  - Exit `4` if hooks disabled by policy.

### `codex hooks exec-log`
- **Description**: Show recent execution records.
- **Options**:
  - `--since <ISO8601>`: filter by timestamp.
  - `--event <name>` / `--hook-id <id>`: filters.
  - `--tail <n>`: limit records.
- **Output (JSON)**:
  ```json
  {
    "decision": "deny",
    "hookId": "managed.shell.guard",
    "event": "PreToolUse",
    "timestamp": "2025-09-20T21:15:43.211Z",
    "durationMs": 128,
    "exitCode": 2,
    "message": "Blocking rm -rf on prod paths",
    "stdout": ["blocked: /var/www"],
    "stderr": []
  }
  ```

### `codex hooks validate`
- **Description**: Run schema validation, resolve precedence conflicts, and warn about disabled hooks.
- **Exit codes**:
  - `0`: validation successful.
  - `2`: fatal error (invalid schema, missing executable, incompatible schema version).
  - `3`: warnings only (conflicts, deprecated fields); prints summary.
- **Integration**: Called automatically by CI job `just hooks-validate` (to be introduced).

### `codex hooks reload`
- **Description**: Invalidate cached registry within running Codex session; emits success or failure message.
- **Constraints**: Works only in daemonized/TUI mode; prints informative error otherwise.

## Data Contracts
All CLI commands returning JSON MUST conform to schemas defined in `hook-config-schema.yaml` and `hook-payload-schema.json`. Execution log records reuse the `HookExecutionRecord` entity shape.
