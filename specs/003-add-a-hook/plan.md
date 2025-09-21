# Implementation Plan: Add Codex Hook System

**Branch**: `003-add-a-hook` | **Date**: 2025-09-20 | **Spec**: /home/iatzmon/workspace/codex/specs/003-add-a-hook/spec.md  
**Input**: Feature specification from /home/iatzmon/workspace/codex/specs/003-add-a-hook/spec.md

## Execution Flow (/plan command scope)
```
1. ✅ Loaded feature spec from Input path
2. ✅ Filled Technical Context (no outstanding NEEDS CLARIFICATION)
3. ✅ Reviewed constitution requirements from /home/iatzmon/workspace/codex/.specify/memory/constitution.md
4. ✅ Initial Constitution Check recorded
5. ✅ Completed Phase 0 research → /home/iatzmon/workspace/codex/specs/003-add-a-hook/research.md
6. ✅ Completed Phase 1 design outputs → data-model.md, contracts/, quickstart.md
7. ✅ Post-Design Constitution Check recorded
8. ✅ Documented Phase 2 task generation approach → /home/iatzmon/workspace/codex/specs/003-add-a-hook/tasks.md
9. ✅ Ready for /tasks execution (no errors encountered)
```

## Summary
Codex will gain a lifecycle hook platform matching Claude Code’s capabilities by layering managed, project, and local hook definitions, executing hook commands with deterministic JSON payloads, and surfacing decisions through CLI/TUI tooling. New Rust services (`HookRegistry`, `HookExecutor`) load TOML-based configurations, enforce schema-version compatibility, and log every execution, while Codex CLI gains `codex hooks *` commands for inspection, validation, and reload workflows. Legacy `notify` behavior is preserved by synthesizing a Notification hook so existing automations continue to work.

## Technical Context
**Language/Version**: Rust 1.89.0 (`rust-toolchain.toml`), Node.js ≥20 for CLI shell, Bash for hook scripts  
**Primary Dependencies**: `tokio` async runtime, `serde`/`toml` parsing, existing `wildmatch` crate (extended with optional `regex` for advanced matchers), shared `codex_protocol` models  
**Storage**: File-based configuration under `/etc/codex/hooks`, `<project>/.codex/hooks.toml`, and `$CODEX_HOME/hooks` plus JSONL execution logs in `$CODEX_HOME/logs/hooks.jsonl`  
**Testing**: `cargo test -p codex-core`, `cargo test --all-features` when shared crates touched, `cargo test -p codex-tui`, `pnpm test` for CLI snapshots, `cargo insta pending-snapshots` for new UI fixtures  
**Target Platform**: Terminal-based Codex sessions on macOS/Linux with optional sandboxing (workspace-write or danger-full-access)  
**Project Type**: Single project with multi-crate Rust core + Node CLI; no separate frontend/backend split  
**Performance Goals**: Hook resolution adds <50 ms overhead per event and does not increase tool latency beyond configured timeout budget  
**Constraints**: Maintain fork-friendly additive changes (avoid wholesale refactors that hinder upstream merges); enforce fail-safe behavior on hook errors; respect sandbox policies when spawning commands  
**Scale/Scope**: Designed for tens of hooks per layer, sub-second reloads, and persistent audit logs covering multi-hour sessions

## Constitution Check
- ✅ Workspace integrity: CLI additions stay in `codex-cli`, runtime logic in `codex-rs`, shared types in `codex-rs/common`; tests accompany boundary points.
- ✅ Template-governed flow: Plan, research, design, and tasks generated from `.specify/templates/*` with unused sections removed.
- ✅ Test-first assurance: Tasks enumerate failing tests before implementation and mandate `cargo`/`pnpm` evidence.
- ✅ Style discipline: Plan commits to `just fmt`, `just fix`, Ratatui `Stylize`, and avoids new crates unless justified (reuse `wildmatch`/`regex`).
- ✅ Release & observability: Logging strategy documented, CLI exposes auditing, release notes/tasks capture semantic version and doc updates.

## Project Structure
### Documentation & Planning Artifacts
```
/home/iatzmon/workspace/codex/specs/003-add-a-hook/
├── plan.md          # This implementation plan
├── research.md      # Phase 0 findings
├── data-model.md    # Phase 1 entity definitions
├── quickstart.md    # Operator walkthrough
├── contracts/       # Schemas & CLI contracts
└── tasks.md         # Phase 2 task backlog
```

### Source Code Touchpoints
```
/home/iatzmon/workspace/codex/
├── codex-rs/core/src/hooks/         # NEW: registry, executor, runtime glue
├── codex-rs/common/src/hooks.rs     # Shared types for config + CLI bindings
├── codex-rs/core/src/config.rs      # Layered hook loading (additive changes)
├── codex-rs/core/tests/             # Integration tests for hook flows
├── codex-rs/tui/src/...             # Hook inspector panel & status messages
├── codex-cli/src/hooks/             # NEW: CLI namespace implementation
└── docs/config.md                   # Documentation updates & migration guide
```

**Structure Decision**: Retain existing single-project layout; add focused modules within current crates instead of introducing new crates or restructuring directories.

## Phase 0: Outline & Research
- Resolved lifecycle parity, exit-code semantics, and payload fields based on Claude Code references (see research.md).
- Selected TOML configuration with layered discovery, defined default locations, and confirmed precedence logic.
- Chose async execution model with per-hook timeouts and JSONL audit logging to satisfy compliance requirements.
- Documented compatibility strategy for legacy `notify` setting and fork-friendly additive implementation boundaries.

**Artifact**: /home/iatzmon/workspace/codex/specs/003-add-a-hook/research.md

## Phase 1: Design & Contracts
- Captured entities (`HookDefinition`, `HookExecutionRecord`, etc.) in data-model.md to align runtime behavior and logging expectations.
- Authored configuration and payload schemas plus CLI contract under /home/iatzmon/workspace/codex/specs/003-add-a-hook/contracts/.
- Produced operator quickstart covering guard setup, validation, execution logs, and legacy notify migration.
- Identified CLI ↔ backend interactions requiring new RPC endpoints and JSON schemas for list/exec-log/validate flows.

**Artifacts**:
- /home/iatzmon/workspace/codex/specs/003-add-a-hook/data-model.md  
- /home/iatzmon/workspace/codex/specs/003-add-a-hook/contracts/hook-config-schema.yaml  
- /home/iatzmon/workspace/codex/specs/003-add-a-hook/contracts/hook-payload-schema.json  
- /home/iatzmon/workspace/codex/specs/003-add-a-hook/contracts/hooks-cli.md  
- /home/iatzmon/workspace/codex/specs/003-add-a-hook/quickstart.md

## Phase 2: Task Planning Approach
Tasks prioritize failing tests, followed by implementation and documentation. Parallel-safe tasks marked `[P]` for CLI vs core work streams. TDD enforcement includes `cargo test`/`pnpm test` evidence and snapshot review. See /home/iatzmon/workspace/codex/specs/003-add-a-hook/tasks.md for the ordered backlog (28 tasks covering tests, runtime, CLI, TUI, docs, and release prep).

## Complexity Tracking
No constitutional deviations identified; table not required.

## Progress Tracking
**Phase Status**:
- [x] Phase 0: Research complete (/plan command)
- [x] Phase 1: Design complete (/plan command)
- [x] Phase 2: Task planning complete (/plan command)
- [ ] Phase 3: Tasks generated (/tasks command)
- [ ] Phase 4: Implementation complete
- [ ] Phase 5: Validation passed

**Gate Status**:
- [x] Initial Constitution Check: PASS
- [x] Post-Design Constitution Check: PASS
- [x] All NEEDS CLARIFICATION resolved
- [ ] Complexity deviations documented (not needed)

---
*Based on Constitution v3.0.0 – see /home/iatzmon/workspace/codex/.specify/memory/constitution.md*
