# Codex Hooks Quickstart

This walkthrough demonstrates how a project admin configures a `PreToolUse` guard and a `PostToolUse` formatter using the new hook system.

## Prerequisites
- Codex CLI built from branch `003-add-a-hook`
- Project workspace at `/workspace/project`
- Managed policy hooks distributed under `/etc/codex/hooks`

## Steps

1. **Create a project hook file**
   ```toml
   # /workspace/project/.codex/hooks.toml
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

2. **Author the hook script**
   ```bash
   # /workspace/project/hooks/pretool.sh
   #!/usr/bin/env bash
   read payload
   if jq -e '.eventContext.arguments.command | test("rm -rf /var/www")' <<<"$payload"; then
     jq -n '{ decision: "deny", message: "Production paths are blocked" }'
     exit 2
   fi
   jq -n '{ decision: "allow" }'
   ```

3. **Run validation**
   ```bash
   codex hooks validate --scope project --json
   ```
   - Expect exit code `0` and JSON summary of loaded hooks.

4. **Start Codex session in the project**
   ```bash
   codex --cwd /workspace/project
   ```
   - On startup the session loads managed + project hooks and logs the registry snapshot.

5. **Trigger the guard**
   - Ask Codex to run `rm -rf /var/www`. Codex emits a `PreToolUse` payload, the hook returns `deny`, and Codex surfaces the message in the transcript while blocking the command.

6. **Review execution logs**
   ```bash
   codex hooks exec-log --tail 1 --json
   ```
   - Output includes event name, decision `deny`, and exit code `2` confirming enforcement.

7. **Migrate legacy notify**
   - If `notify = ["notify-send", "Codex"]` exists in `config.toml`, `codex hooks list` will display a synthetic `Notification` hook. Remove the legacy entry once an explicit hook is created.

## Expected Outcomes
- Hook scripts run with deterministic payloads and enforce safety policies before dangerous tool calls.
- CLI tooling exposes hook registry and execution history for audits.
- Legacy notify workflows continue to function while guided toward the new hook system.
