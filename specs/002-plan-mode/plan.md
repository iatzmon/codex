# Implementation Plan: Codex CLI Plan Mode Read-Only Planning State

**Branch**: `002-plan-mode` | **Date**: 2025-09-18 | **Spec**: /home/iatzmon/workspace/codex/specs/002-plan-mode/spec.md
**Input**: Feature specification from /home/iatzmon/workspace/codex/specs/002-plan-mode/spec.md

## Execution Flow (/plan command scope)
```
1. Load feature spec from Input path
   → If not found: ERROR "No feature spec at {path}"
2. Fill Technical Context (scan for NEEDS CLARIFICATION)
   → Detect Project Type from context (web=frontend/backend, mobile=app+api)
   → Set Structure Decision based on project type
3. Fill the Constitution Check section based on the content of the constitution document.
4. Evaluate Constitution Check section below
   → If violations exist: Document in Complexity Tracking
   → If no justification possible: ERROR "Simplify approach first"
   → Update Progress Tracking: Initial Constitution Check
5. Execute Phase 0 → research.md
   → If NEEDS CLARIFICATION remain: ERROR "Resolve unknowns"
6. Execute Phase 1 → contracts, data-model.md, quickstart.md, agent-specific template file (e.g., `CLAUDE.md` for Claude Code, `.github/copilot-instructions.md` for GitHub Copilot, or `GEMINI.md` for Gemini CLI).
7. Re-evaluate Constitution Check section
   → If new violations: Refactor design, return to Phase 1
   → Update Progress Tracking: Post-Design Constitution Check
8. Plan Phase 2 → Describe task generation approach (DO NOT create tasks.md)
9. STOP - Ready for /tasks command
```

**IMPORTANT**: The /plan command STOPS at step 7. Phases 2-4 are executed by other commands:
- Phase 2: /tasks command creates tasks.md
- Phase 3-4: Implementation execution (manual or via tools)

## Summary
Introduce a guarded Plan Mode for Codex CLI that activates via `/plan` or `--plan`, keeps the workspace strictly read-only, captures proposed diffs and commands as structured plan entries, surfaces a persistent PLAN indicator in the UI, and allows operators to exit or apply the plan via explicit commands without mutating files or running shells during planning.

## Technical Context
**Language/Version**: Rust 1.89.0 (rust-toolchain)  
**Primary Dependencies**: clap (CLI parsing), tokio (async runtime), ratatui (TUI rendering), serde/serde_json (config + protocol), codex-protocol crates (shared message types)  
**Storage**: Local file reads only; no new persistence beyond conversation transcripts (reuse existing logging)  
**Testing**: `cargo test -p codex-core`, `cargo test -p codex-tui`, `cargo test -p codex-cli`, snapshot suites in `codex-tui` via `cargo insta`  
**Target Platform**: macOS & Linux terminals (current CLI targets)  
**Project Type**: single (multi-crate Rust workspace; no separate frontend/backend split)  
**Performance Goals**: Maintain current interactive latency (<200ms agent round-trip overhead for policy checks); no additional network calls while in Plan Mode  
**Constraints**: Plan Mode must never relax sandboxing defaults, must honor `CODEX_SANDBOX_NETWORK_DISABLED=1`, cannot mutate files or execute commands, must respect existing approval-mode plumbing, and must fail-safe to the prior mode on errors  
**Scale/Scope**: Typical CLI sessions with tens of plan entries per conversation; expected to run within existing memory footprint (<500MB) and open file handle limits  

**Integration Touchpoints (minimal change preference)**:
- `codex-rs/tui/src/cli.rs` & `codex-rs/cli/src/main.rs`: add `--plan` flag wiring and propagate Plan Mode into interactive session setup without duplicating flag parsing.
- `codex-rs/common/src/approval_mode_cli_arg.rs` & `codex-rs/core/src/config.rs`: extend approval/session configuration with a `Plan` state that maps to existing `AskForApproval` semantics without altering default modes.
- `codex-rs/core/src/environment_context.rs` & `codex-rs/core/src/shell.rs`: enforce read-only tool gating and command refusal paths by augmenting existing sandbox/approval checks rather than introducing parallel pipelines.
- `codex-rs/tui/src/status_indicator_widget.rs` and chat history components: layer in PLAN badge/tooltip using current ratatui helpers to avoid reworking layout managers.
- `codex-rs/core/src/plan_tool.rs` & protocol enums: reuse plan update messaging to capture suggested diffs/commands instead of inventing new transport.
- `codex-rs/common/src/config_profile.rs` & user config loaders: add Plan Mode overrides with backward-compatible defaults.

## Constitution Check
- **Security-First Architecture**: PASS — Plan Mode increases, not decreases, sandboxing by blocking mutations and commands; all new logic will default to read-only and reuse existing sandbox/env guards.
- **Library-Centric Design**: PASS — augment existing `codex-core` session orchestration and expose toggles through thin CLI/TUI layers without embedding business logic in binaries.
- **Test-Driven Quality**: PASS (plan) — design enumerates unit + snapshot coverage before implementation; tasks ensure tests precede functional code.
- **Rust Standards & Tooling**: PASS — plan commits to workspace `just fmt`, `just fix -p`, and Clippy compliance; format strings will inline variables per constitution.
- **User Experience Excellence**: PASS — UI updates focus on clear PLAN indicator, refusal messaging, and guidance to exit/apply plans.

## Project Structure

### Documentation (this feature)
```
specs/002-plan-mode/
├── plan.md              # This file (/plan command output)
├── research.md          # Phase 0 output (/plan command)
├── data-model.md        # Phase 1 output (/plan command)
├── quickstart.md        # Phase 1 output (/plan command)
├── contracts/           # Phase 1 output (/plan command)
└── tasks.md             # Phase 2 output (/tasks command - NOT created by /plan)
```

### Source Code (repository root)
```
# Option 1: Single project (DEFAULT)
src/
├── models/
├── services/
├── cli/
└── lib/

tests/
├── contract/
├── integration/
└── unit/

# Option 2: Web application (when "frontend" + "backend" detected)
backend/
├── src/
│   ├── models/
│   ├── services/
│   └── api/
└── tests/

frontend/
├── src/
│   ├── components/
│   ├── pages/
│   └── services/
└── tests/

# Option 3: Mobile + API (when "iOS/Android" detected)
api/
└── [same as backend above]

ios/ or android/
└── [platform-specific structure]
```

**Structure Decision**: Option 1 (single project) — existing Rust workspace already aligns with this structure.

## Phase 0: Outline & Research
1. **Extract unknowns & dependencies**
   - Clarify `/apply-plan` default mode fallback when none provided.
   - Define UX for missing `.codex/plan.md` template and unreadable plan files.
   - Determine rejection messaging/telemetry when users attempt disallowed actions (writes, external attachments).
   - Confirm gating rules for read-only MCP tools and web research approvals.

2. **Research task queue**
   - "Research `/apply-plan` fallback strategy that is consistent with existing approval policy handling."
   - "Research fallback messaging when `.codex/plan.md` missing/unreadable in Plan Mode."
   - "Research refusal UX patterns in Codex CLI for disallowed commands/files to keep tone consistent."
   - "Research MCP/web tool gating hooks to ensure Plan Mode reuses existing allowlists without bypasses."

3. **Consolidate findings**
   - Record decisions, rationale, and alternatives in /home/iatzmon/workspace/codex/specs/002-plan-mode/research.md ensuring no NEEDS CLARIFICATION remain.

**Output**: /home/iatzmon/workspace/codex/specs/002-plan-mode/research.md with resolved decisions for activation, gating, and user messaging.

## Phase 1: Design & Contracts
1. **Data modelling**
   - Capture PlanModeSession, PlanArtifact, PlanEntry, and ToolPolicy entities with state transitions in /home/iatzmon/workspace/codex/specs/002-plan-mode/data-model.md.

2. **Contracts**
   - Define CLI/TUI interaction contract (activation, refusal, apply/exit flows) as OpenAPI-style documentation under /home/iatzmon/workspace/codex/specs/002-plan-mode/contracts/.
   - Specify event payload schema for plan updates leveraging existing protocol types.

3. **Test scaffolding**
   - Outline new unit + integration + snapshot tests derived from requirements inside contracts and quickstart docs so they fail pre-implementation.

4. **Quickstart**
   - Document manual validation script in /home/iatzmon/workspace/codex/specs/002-plan-mode/quickstart.md covering activation, refusal, plan generation, and exit/apply flows.

5. **Agent context**
   - Run `/home/iatzmon/workspace/codex/.specify/scripts/bash/update-agent-context.sh claude` after consolidating new tech/context so CLAUDE.md reflects Plan Mode information without disturbing manual edits.

**Output**: data model, contracts, quickstart, and updated agent context capturing Plan Mode details.

## Phase 2: Task Planning Approach
**Task Generation Strategy**:
- Use /home/iatzmon/workspace/codex/.specify/templates/tasks-template.md to derive tasks from Phase 1 artifacts.
- Map each functional requirement to at least one implementation and one verification task; label protocol + UI updates separately for parallelism.
- Include tasks for updating CLAUDE.md and ensuring configuration overrides are documented.

**Ordering Strategy**:
- Begin with protocol/core changes (blocking enforcement), then CLI flag wiring, then TUI indicator, followed by documentation & config updates.
- Prepend test tasks (unit + snapshot) before corresponding implementation steps; mark independent UI vs core workstreams with `[P]` for parallel execution.

**Estimated Output**: 24-28 ordered tasks in /home/iatzmon/workspace/codex/specs/002-plan-mode/tasks.md ready for `/tasks` consumption.

## Phase 3+: Future Implementation
**Phase 3**: Task execution (/tasks command writes tasks.md)  
**Phase 4**: Implementation (execute tasks.md following constitutional principles)  
**Phase 5**: Validation (run tests, quickstart, performance checks)

## Complexity Tracking
_No constitutional violations identified; table intentionally left empty._

## Progress Tracking
**Phase Status**:
- [x] Phase 0: Research complete (/plan command)
- [x] Phase 1: Design complete (/plan command)
- [x] Phase 2: Task planning complete (/plan command - describe approach only)
- [ ] Phase 3: Tasks generated (/tasks command)
