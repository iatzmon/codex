<!--
Sync Impact Report
Version: 2.1.1 → 3.0.0
Modified Principles:
- I. Library-First → I. Dual-Core Workspace Integrity
- II. CLI Interface → II. Template-Governed Flow
- III. Test-First → III. Test-First Assurance
- IV. Integration Testing → IV. Style and Simplicity Discipline
- V. Observability/Versioning/Simplicity → V. Release and Observability Control
Added Sections:
- Operational Constraints
- Workflow Expectations
Removed Sections:
- None
Templates requiring updates:
- ✅ .specify/templates/plan-template.md
- ✅ .specify/templates/spec-template.md
- ✅ .specify/templates/tasks-template.md
- ✅ .specify/templates/agent-file-template.md (Reviewed; no content changes required)
Follow-up TODOs:
- None.
-->
# OpenAI Codex CLI Constitution

## Core Principles

### I. Dual-Core Workspace Integrity
- MUST keep Node.js command UX inside `codex-cli` and Rust execution logic inside `codex-rs`; cross-boundary changes require paired updates and tests.
- MUST prefix new Rust crates with `codex-`, place integration tests in `codex-rs/core/tests`, and store TUI snapshots in `codex-rs/tui/tests`.
- MUST store shared documentation and assets in `docs/` or `.github/` so tooling and releases stay deterministic.
Rationale: Preserving the CLI/core split keeps packaging predictable, stabilizes build tooling, and prevents test suites from drifting.

### II. Template-Governed Flow
- MUST base specs, plans, and tasks on the templates under `.specify/templates/`; unused sections are removed instead of left blank.
- MUST execute `/plan` before `/tasks`, completing the Constitution Check gates and documenting any justified complexity in the plan.
- MUST update downstream guidance (agent files, docs) whenever principles change so contributors receive consistent instructions.
Rationale: Template-driven flow delivers auditable planning and prevents agents from bypassing guardrails.

### III. Test-First Assurance
- MUST author contract, integration, and unit tests before implementation; tests MUST fail prior to writing functional code.
- MUST run targeted `cargo test -p <crate>` (or `cargo test --all-features` when touching shared crates) and `pnpm build` for CLI packaging before merge.
- MUST review snapshot diffs via `cargo insta pending-snapshots` and use `pretty_assertions::assert_eq` in Rust unit tests for readable failures.
Rationale: Enforcing TDD and explicit test evidence protects the CLI’s stability across fast release cycles.

### IV. Style and Simplicity Discipline
- MUST format Rust with `just fmt`, lint with `just fix -p <crate>`, and keep module names snake_case with compact imports and inlined `format!` arguments.
- MUST style Ratatui widgets via `Stylize` helpers (e.g., `.dim()`, `.cyan().underlined()`) and avoid hard-coded white foregrounds so themes stay accessible.
- MUST apply YAGNI: introduce patterns, layers, or extra crates only when demanded by current requirements and recorded in Complexity Tracking.
Rationale: Consistent style and minimalism keep the codebase approachable for new agents and prevent UI regressions.

### V. Release and Observability Control
- MUST log CLI-visible operations with structured, text-first output so users can troubleshoot without additional tooling.
- MUST bump project and constitution versions with semantic intent (major for rule replacements, minor for new mandates, patch for clarifications) and document changes in CHANGELOG entries or release notes.
- MUST use imperative commit messages and include test evidence (e.g., `cargo test -p …`, `cargo insta pending-snapshots`) in PR descriptions for reviewer accountability.
Rationale: Disciplined versioning and observability ensure released binaries remain trustworthy and auditable.

## Operational Constraints
- pnpm is the authoritative package manager for Node tooling; agents MUST NOT substitute npm/yarn without governance approval.
- Rust workflows MUST rely on the provided `just` recipes (`just fmt`, `just fix -p <crate>`) and `cargo` commands; ad-hoc scripts require documentation in specs.
- Sandbox guards (`CODEX_SANDBOX`, `CODEX_SANDBOX_NETWORK_DISABLED`) MUST remain in place so automated runs respect offline environments.
- Network-enabled actions MUST follow documented approval policies; destructive commands are prohibited unless explicitly authorized in writing.

## Workflow Expectations
- Each feature begins with a spec derived from `.specify/templates/spec-template.md`, followed by a plan and Constitution Check before implementation tasks are generated.
- Agents MUST track progress through plan checkpoints (research, design, task generation) and pause when Constitution gates fail.
- Implementation MUST follow tasks in TDD order, keeping documentation and agent guidance files synchronized with actual changes.
- Pull requests MUST summarize user-facing impacts, note related issues, and attach validation artifacts (tests, snapshots, screenshots for TUI updates).

## Governance
- Amendments to this constitution require a PR that links affected templates, records rationale in the Sync Impact Report, and gains maintainer approval.
- Version increments follow semantic rules: MAJOR for principle removal/replacement, MINOR for new sections or mandates, PATCH for clarifications; the comment header MUST reflect the latest version.
- Compliance reviews occur at Constitution Check during planning and at code review; failure to meet principles blocks merges until remediated.

**Version**: 3.0.0 | **Ratified**: 2025-07-16 | **Last Amended**: 2025-09-20
