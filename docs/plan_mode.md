# Plan Mode

Plan Mode keeps Codex in a read-only planning state so you can outline work before touching the workspace. It is available in every frontend that talks to `codex-core`.

## Enabling Plan Mode

- Start Codex with `--plan` to enter planning before the first turn, or
- Send the `/plan` slash command during a session.

When active, the UI renders a `PLAN` badge with quick tips for exiting or applying the plan. A reminder to run `/save-plan <path>` is included so you can persist the artifact outside the transcript.

## Behaviour

- All write operations (shell, patch, MCP tools that can mutate state) are blocked. The attempted action is converted into a plan entry and appended to the active artifact.
- Read-only tools continue to work. Administrators can opt-in specific MCP tools with `plan_mode.allowed_read_only_tools` in the config file.
- Local attachments are only accepted when they live inside the workspace (or explicitly whitelisted writable roots). External paths produce a security refusal that explains how to exit Plan Mode first.
- The agent stores a structured plan artifact containing objectives, constraints, approach, detailed steps, risks, alternatives, rollback/mitigations, success criteria, tests, next actions, and notes.

## Exiting or Applying

- `/exit-plan` leaves Plan Mode and restores the previous approval policy.
- `/apply-plan [mode]` applies the captured plan, injects it into the next turn, and switches to the requested approval policy (defaults to the policy active before planning).

Both commands emit dedicated lifecycle events so frontends can update the badge and tool availability.

## Configuration

Plan Mode settings live in the top-level config under the `plan_mode` table:

```toml
[plan_mode]
plan_enabled = false                # Enable Plan Mode automatically at startup
allowed_read_only_tools = ["fs.read"]
planning_model = "gpt-4o"            # Optional planner model override
apply_requires_confirmation = true   # Require explicit mode when applying
```

List entries can include glob patterns to match families of tools (for example `n8n-mcp__list_*`), and shell allowances may be granted with `shell(<pattern>)`, such as `shell(npm run test:*)` or `shell(cat *)`.

See `docs/config.md` for the full configuration reference.
