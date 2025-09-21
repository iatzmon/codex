# Phase 0 Research – Codex Hook System

## Decision: Lifecycle event coverage and payload semantics
- **Decision**: Support the full Claude Code lifecycle set (`PreToolUse`, `PostToolUse`, `UserPromptSubmit`, `Notification`, `Stop`, `SubagentStop`, `PreCompact`, `SessionStart`, `SessionEnd`) with structured JSON payloads delivered via stdin and support for decision/control fields (`permissionDecision`, `continue`, `systemMessage`, etc.). Exit codes follow Claude’s conventions: `0` success, `2` decisive block, other non-zero = soft error.
- **Rationale**: Claude Code documentation and community references emphasize deterministic control, payload structure, and exit-code semantics for all nine events, and matching them guarantees portability for users migrating from Claude Code while satisfying FR-001–FR-013. citeturn0search1turn0search4turn0search9
- **Alternatives considered**: Limiting scope to tool-centric events would reduce implementation effort but leave gaps in session governance, violating spec requirements and forcing users to maintain Claude-specific hooks separately.

## Decision: Configuration layering and file layout
- **Decision**: Load hooks from three layers in precedence order — managed policy (`/etc/codex/hooks/*.toml` or `$CODEX_MANAGED_HOOKS`), project (`<project>/.codex/hooks.toml` + directory overrides), and local user (`$CODEX_HOME/hooks/hooks.toml`). Merge definitions by event, evaluating the highest-precedence decisive result first while preserving audit trails of lower-precedence hooks.
- **Rationale**: Claude Code resolves hooks from enterprise policy, project, and user scopes with first decisive decision winning; mirroring that behavior makes migrations predictable and satisfies FR-002. Using TOML aligns with existing Codex config tooling and avoids introducing a new parser. citeturn0search1turn0search10
- **Alternatives considered**: Embedding hooks directly in `config.toml` would minimize new files but complicate managed policy distribution and contradict the spec’s layered override requirement.

## Decision: Hook runtime execution model
- **Decision**: Introduce a `HookExecutor` service inside `codex-rs/core/src/hooks/` that resolves eligible hooks, spawns commands via `tokio::process::Command`, enforces per-hook timeouts (default 60s, configurable), streams stdout/stderr for logging, and emits structured execution records to `~/.codex/logs/hooks.jsonl`.
- **Rationale**: Claude Code runs hooks as shell commands with timeouts and parallelism; reproducing this preserves parity and supports compliance logging (FR-014/FR-015). Async execution keeps the agent responsive while honoring the spec’s fail-safe requirement for timeouts and errors. citeturn0search0turn0search1
- **Alternatives considered**: Synchronous blocking execution would be simpler but risks stalling Codex during long-running hooks and violates fail-safe expectations.

## Decision: Matcher and schema version enforcement
- **Decision**: Implement matcher support (exact, glob via `wildmatch`, regex via `regex` crate) for tool-scoped events and require each hook entry to declare supported schema versions. Reject execution when versions are incompatible and surface warnings when only future versions are advertised.
- **Rationale**: Claude Code matchers accept patterns (`Write|Edit`, `*`) and spec mandates schema version negotiation to prevent incompatible payloads; we already depend on `wildmatch` for shell policies, so reuse avoids extra dependencies. citeturn0search1turn0search10
- **Alternatives considered**: Using only exact matchers would simplify parsing but break parity and reduce usefulness for multi-tool workflows.

## Decision: CLI and observability tooling
- **Decision**: Extend `codex-cli` with a `codex hooks` namespace to list active hooks, show layered source paths, tail recent executions, and validate configuration. Surface the same data via a `codex-rs` TUI panel and persist execution records in JSONL for off-line audits.
- **Rationale**: Claude Code exposes `/hooks` UI and debug tooling; FR-014/FR-015 require inspectability. Providing CLI access fits Codex’s headless workflows and leverages existing logging patterns. citeturn0search1turn0search4
- **Alternatives considered**: Restricting to log files alone would satisfy minimal auditing but fails discoverability and increases support load.

## Decision: Compatibility with existing notification command
- **Decision**: Treat the current `notify` shell integration as a specialized `Notification` hook preset that is automatically generated when legacy `notify` config is detected. Users can migrate to explicit hooks but existing behavior persists without change.
- **Rationale**: FR-017 demands continuity for the notify workflow. Wrapping it as a synthesized hook avoids breaking behavior while steering users toward the new system.
- **Alternatives considered**: Removing the legacy path would force users to rewrite configs immediately and risk regressions.

## Decision: Fork-friendly implementation boundaries
- **Decision**: Introduce new modules (`codex-rs/core/src/hooks`, shared types in `codex-rs/common`, minimal touchpoints in core execution pipeline) and avoid broad refactors. Hook-specific code stays additive with feature flags where possible to ease future upstream merges.
- **Rationale**: User instructions note this work happens in a fork; reducing churn minimizes conflicts when syncing with upstream Codex.
- **Alternatives considered**: Refactoring existing config or exec modules heavily would provide cleaner abstractions but violate the “avoid disruptive refactors in a fork” constraint.

