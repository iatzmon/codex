# Codex Hooks

Codex can run external programs at key lifecycle events (pre/post tool use,
session transitions, notifications, etc.). Hooks allow teams to audit, enrich,
or block automated actions using simple shell scripts driven by structured JSON
payloads.

## Configuration layers

Hook definitions live in layered TOML files. The loader evaluates them in
precedence order—the first decisive hook wins while lower-priority layers are
still logged for observability.

1. **Managed policy** – `/etc/codex/hooks/*.toml` or the directory referenced by
   `CODEX_MANAGED_HOOKS` (enterprise administrators).
2. **Project** – files in `<workspace>/.codex/hooks.toml` and
   `<workspace>/.codex/hooks/` (shared by the repository).
3. **Local user** – `$CODEX_HOME/hooks/hooks.toml` and `$CODEX_HOME/hooks/*.toml`
   (personal overrides, defaults to `~/.codex/hooks`).

Every file must begin with `schemaVersion = "1.0"` and provide a `[[hooks]]`
array. Each entry describes the event, matchers, command to execute, timeout,
and optional metadata surfaced in audit logs.

```toml
schemaVersion = "1.0"

defaultTimeoutMs = 60000

[[hooks]]
id = "project.shell.guard"
event = "PreToolUse"
command = ["./hooks/pretool.sh"]
schemaVersions = ["1.0"]
timeoutMs = 10000

  [hooks.matchers]
  toolNames = [{ type = "glob", value = "shell*" }]
```

### Matchers

Match against tool names, session sources, file paths, or custom tags using one
of three matcher types:

- `"exact"` – literal match.
- `"glob"` – wildcard matching (`*`, `?`, character classes) powered by
  `wildmatch`.
- `"regex"` – full Rust regular expressions.

Entries without matchers apply to every execution of the selected event.

## Lifecycle events

Codex mirrors Claude Code’s lifecycle coverage. Hooks may subscribe to:

- `PreToolUse` / `PostToolUse`
- `UserPromptSubmit`
- `Notification`
- `Stop` / `SubagentStop`
- `PreCompact`
- `SessionStart` / `SessionEnd`

The hook payload is delivered via stdin as JSON, with a `schemaVersion`, shared
session context, and an event-specific `eventContext` object. Exit code `0`
means success, `2` indicates a decisive block, and any other non-zero code is
considered a soft failure (Codex will warn and fall back to the conservative
baseline).

Full payload schemas are published under `specs/003-add-a-hook/contracts/`.

## CLI commands

Inspect and manage hooks directly from the CLI. All commands support `--json`
for machine-readable output.

| Command | Description |
| --- | --- |
| `codex hooks list [--event <name>] [--scope managed|project|local] [--json]` | View registry snapshot and per-layer statistics. |
| `codex hooks validate [--scope …] [--json]` | Run schema validation and report skipped hooks. |
| `codex hooks exec-log [--since …] [--event …] [--hook-id …] [--tail <n>] [--json]` | Tail execution records written to `$CODEX_HOME/logs/hooks.jsonl`. |
| `codex hooks reload` | Ask the running daemon to reload configuration from disk. |

## Quickstart example

1. Create project hook file at `.codex/hooks.toml`:

    ```toml
    schemaVersion = "1.0"

    [[hooks]]
    id = "project.shell.guard"
    event = "PreToolUse"
    schemaVersions = ["1.0"]
    command = ["./hooks/pretool.sh"]
    timeoutMs = 10000

      [hooks.matchers]
      toolNames = [{ type = "glob", value = "shell*" }]
    ```

2. Author `./hooks/pretool.sh` (remember to `chmod +x`):

    ```bash
    #!/usr/bin/env bash
    payload=$(cat)
    if jq -e '.eventContext.arguments.command | test("rm -rf /var/www")' <<<"$payload"; then
      jq -n '{ decision: "deny", message: "Production paths are blocked" }'
      exit 2
    fi
    jq -n '{ decision: "allow" }'
    ```

3. Validate and inspect:

    ```bash
    codex hooks validate --json
    codex hooks list --json
    codex hooks exec-log --tail 5 --json
    ```

4. Trigger the hook by asking Codex to run `rm -rf /var/www`. The `PreToolUse`
   hook denies the command, surfaces the message in the transcript, and records
   the decision in `hooks.jsonl`.

## Logs and observability

All hook executions are appended to `$CODEX_HOME/logs/hooks.jsonl`. Each entry
includes the event, decision, stdout/stderr samples, payload hash, duration, and
layer precedence. Use `codex hooks exec-log` for ad-hoc inspection or consume
the log file directly for centralized auditing.

## Legacy notify migration

Existing `notify` settings inside `config.toml` are automatically converted into
Notification hooks at runtime. You can remove the legacy config once an explicit
hook is in place.

## Performance budgeting

Future work introduces a microbenchmark in `codex-rs/core/benches` to assert the
hook executor adds less than **50 ms** of latency per invocation. Keep scripts
lightweight and cache external dependencies where possible.
