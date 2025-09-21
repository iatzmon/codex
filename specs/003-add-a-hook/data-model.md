# Phase 1 Data Model – Codex Hook System

## Entity: HookDefinition
- **Description**: Declarative rule describing when and how to execute an external hook.
- **Fields**:
  - `id: String` – Stable identifier derived from scope + name (used for auditing).
  - `event: HookEvent` – Enum of lifecycle events (`PreToolUse`, `PostToolUse`, `UserPromptSubmit`, `Notification`, `Stop`, `SubagentStop`, `PreCompact`, `SessionStart`, `SessionEnd`).
  - `scope: HookScope` – `{ManagedPolicy | Project | LocalUser}` with precedence metadata.
  - `source_path: PathBuf` – Absolute path to the TOML file that defined the hook.
  - `command: CommandSpec` – Executable + args array.
  - `working_dir: Option<PathBuf>` – Optional directory override when running the hook.
  - `env: HashMap<String, String>` – Extra environment variables injected when spawning.
  - `timeout_ms: u64` – Max lifetime before fail-safe triggers (default 60000).
  - `matchers: HookMatchers` – Tool or source filters depending on event type.
  - `schema_versions: Vec<String>` – Supported JSON schema versions.
  - `allow_parallel: bool` – Whether multiple instances may run concurrently for same event key.
  - `notes: Option<String>` – Human-readable description surfaced in audits.

## Entity: HookMatchers
- **Description**: Predicate set used to decide whether a hook should run for a given event instance.
- **Fields**:
  - `tool_names: Vec<Matcher>` – Applies to tool-driven events (`PreToolUse`, `PostToolUse`).
  - `sources: Vec<Matcher>` – Applies to session lifecycle events (startup source, stop reason, notification type).
  - `paths: Vec<Matcher>` – Applies when events reference file paths.
  - `tags: Vec<String>` – Arbitrary labels for grouping and UI filters.
- **Relationships**: Nested inside `HookDefinition`; `Matcher` uses exact/glob/regex variants.

## Entity: HookEventPayload
- **Description**: JSON payload sent to hook processes.
- **Fields (common)**:
  - `schemaVersion: String`
  - `event: String` – Lifecycle event name.
  - `sessionId: Uuid`
  - `timestamp: DateTime<Utc>`
  - `workspaceRoot: PathBuf`
  - `currentWorkingDirectory: PathBuf`
  - `transcriptPath: Option<PathBuf>`
  - `sandboxes: SandboxContext` – Contains sandbox mode, network policy, writable roots.
  - `eventContext: serde_json::Value` – Event-specific object.
- **Event-specific context**:
  - `PreToolUse`: `{ toolName, toolKind, arguments, dryRun, matchedHooks }`
  - `PostToolUse`: `{ toolName, arguments, result, diffSummary }`
  - `UserPromptSubmit`: `{ promptText, attachments, derivedInstructions }`
  - `Notification`: `{ kind, message, requiresApproval }`
  - `Stop` / `SubagentStop`: `{ reason, outstandingTasks }`
  - `PreCompact`: `{ trigger, totalTokens, estimatedSavings }`
  - `SessionStart`: `{ source, restoredConversation, approvalsPolicy }`
  - `SessionEnd`: `{ reason, durationMs, totalTurns }`

## Entity: HookDecision
- **Description**: Normalized result returned by a hook execution.
- **Fields**:
  - `decision: HookOutcome` – Enum `{Allow, Ask, Deny, Block, Continue}`.
  - `message: Option<String>` – Feedback for the user / transcript.
  - `system_message: Option<String>` – Additional instructions inserted into agent prompt.
  - `stop_reason: Option<String>` – Populated when `decision == Block` or `Deny`.
  - `extra: serde_json::Value` – Hook-specific metadata (e.g., context keys to cache).
  - `exit_code: i32`
- **Relationships**: Produced by `HookExecutor`, persisted within `HookExecutionRecord`.

## Entity: HookExecutionRecord
- **Description**: Structured audit log entry for each hook invocation.
- **Fields**:
  - `id: Uuid`
  - `timestamp: DateTime<Utc>`
  - `event: HookEvent`
  - `scope: HookScope`
  - `hook_id: String`
  - `decision: HookDecision`
  - `duration_ms: u128`
  - `stdout: Vec<String>` – Truncated lines (configurable length).
  - `stderr: Vec<String>` – Truncated lines.
  - `error: Option<String>` – Populated when hook fails or times out.
  - `precedence_rank: u8` – 0=managed, 1=project, 2=local.
  - `payload_hash: String` – SHA256 of serialized payload for deduping repeated events.
  - `trigger_id: String` – Turn/tool identifier for correlation with transcripts.

## Entity: HookRegistry
- **Description**: Runtime aggregation of hooks grouped by event and precedence.
- **Fields**:
  - `events: HashMap<HookEvent, Vec<HookDefinition>>`
  - `schema_registry: HashMap<String, SchemaVersionMetadata>` – Valid schema versions and compatibility notes.
  - `last_loaded: DateTime<Utc>`
  - `source_layers: Vec<HookLayerSummary>` – Captures file paths, checksums, errors.
- **Relationships**: Instantiated during session startup, refreshed when config files change or on explicit CLI reload.

## Entity: HookLayerSummary
- **Description**: Metadata about each configuration layer used in auditing and CLI output.
- **Fields**:
  - `scope: HookScope`
  - `path: PathBuf`
  - `checksum: String`
  - `loaded_hooks: usize`
  - `skipped_hooks: Vec<SkippedHook>` – Reasons for ignoring invalid entries.

## Entity: SkippedHook
- **Description**: Record for any hook filtered out during load.
- **Fields**:
  - `hook_id: Option<String>`
  - `reason: HookSkipReason` – Enum (InvalidSchema, UnsupportedVersion, DuplicateId, MissingExecutable, InvalidMatcher).
  - `details: Option<String>`

## Entity: HookScope
- **Description**: Enum representing hook origin with precedence awareness.
- **Variants**:
  - `ManagedPolicy { name: String }`
  - `Project { project_root: PathBuf }`
  - `LocalUser { codex_home: PathBuf }`
- **Relationships**: Referenced by `HookDefinition`, `HookExecutionRecord`, and CLI listing responses.

