# Data Model — Plan Mode Read-Only Planning State

## Entity: PlanModeSession
- **Description**: Represents an interactive Codex CLI session while Plan Mode is active.
- **Fields**:
  - `session_id: Uuid` — existing conversation id.
  - `entered_from: AskForApproval` — approval policy that was active before enabling Plan Mode.
  - `allowed_tools: Vec<ToolCapability>` — filtered list of read-only tools.
  - `plan_artifact: PlanArtifact` — accumulated planning output.
  - `entered_at: DateTime<Utc>` — timestamp for telemetry.
  - `pending_exit: Option<AskForApproval>` — cached target mode for `/apply-plan`.
- **Relationships**:
  - Has one `PlanArtifact`.
  - References `EnvironmentContext` for sandbox + workspace scope.
- **State Transitions**:
  1. `Idle → Active` when `/plan` or `--plan` is invoked and guardrails pass.
  2. `Active → Applying` when `/apply-plan` requested; commands remain blocked until transition completes.
  3. `Active → Exited` when `/exit-plan` restores the previous approval policy.

## Entity: PlanArtifact
- **Description**: Structured planning document stored in the transcript.
- **Fields**:
  - `title: String` — user-specified or derived scenario title.
  - `objectives: Vec<String>` — stated goals.
  - `constraints: Vec<String>` — derived limitations (e.g., read-only, sandbox scope).
  - `assumptions: Vec<String>` — explicit assumptions called out by planner.
  - `steps: Vec<PlanEntry>` — ordered plan entries.
  - `risks: Vec<String>` — risk register entries.
  - `next_actions: Vec<String>` — recommended operator follow-ups.
  - `tests: Vec<String>` — test plan bullets.
  - `metadata: PlanArtifactMetadata` — includes model + template references.
- **Relationships**:
  - Aggregates many `PlanEntry` items.
  - Binds to conversation transcript for persistence.

## Entity: PlanEntry
- **Description**: Individual actionable proposal captured instead of executing a command.
- **Fields**:
  - `sequence: u16` — ordering number.
  - `entry_type: PlanEntryType` — enum: `Command`, `FileChange`, `Research`, `Decision`.
  - `summary: String` — concise description of the proposed action.
  - `details: Option<String>` — optional extended rationale or diff snippet.
  - `created_at: DateTime<Utc>` — timestamp.
- **Relationships**:
  - Belongs to a single `PlanArtifact`.
  - Can reference target files/modules for future execution.

## Entity: ToolCapability
- **Description**: Capabilities advertised by shell, MCP, or other tools for gating purposes.
- **Fields**:
  - `id: String` — unique tool identifier (e.g., `shell`, `fs.read`).
  - `mode: ToolMode` — enum: `ReadOnly`, `Write`, `Execute`.
  - `requires_network: bool` — indicates whether network approval is needed.
- **Relationships**:
  - Associated with `EnvironmentContext.tool_registry` for filtering.

## Supporting Structures
- `PlanModeConfigOverrides`
  - `plan_enabled: bool`
  - `allowed_read_only_tools: Vec<String>`
  - `planning_model: Option<String>`
  - `apply_requires_confirmation: bool`
  - Lives within `codex-core::config::ConfigOverrides` to honor FR-008.

- `PlanModeTelemetry`
  - `event: PlanModeEvent` (`Entered`, `RefusalCaptured`, `ApplySuccess`, `Exit`)
  - `previous_mode: AskForApproval`
  - `plan_entry_count: usize`

## Workflow Summary
1. Activation records `PlanModeSession` using the prior approval policy and read-only tool list.
2. Each disallowed command/file request produces a `PlanEntry` appended to the session's `PlanArtifact`.
3. `/exit-plan` or `/apply-plan` transitions the session, restores tool registry, and finalizes telemetry.
