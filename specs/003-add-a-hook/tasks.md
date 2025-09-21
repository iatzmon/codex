# Tasks: Add Codex Hook System

**Input**: Design documents from `/home/iatzmon/workspace/codex/specs/003-add-a-hook/`
**Prerequisites**: plan.md (required), research.md, data-model.md, contracts/, quickstart.md

## Execution Flow (main)
```
1. Load plan.md for tech stack, crate layout, and CLI expectations
2. Read research.md, data-model.md, contracts/, and quickstart.md to extract entities, schemas, and scenarios
3. Generate ordered tasks per category (setup → tests → models → services → endpoints → integration → polish)
4. Mark tasks touching different files as [P] for parallel execution; enforce sequencing for shared files
5. Ensure each contract, entity, endpoint, and quickstart scenario has a corresponding task
6. Document dependencies and practical parallel examples with executable commands
7. Output tasks.md under the feature directory for /tasks consumption
```

## Format: `[ID] [P?] Description`
- **[P]**: Task can run in parallel (different files, no blocking dependency)
- Include explicit repository paths in each task description

## Phase 3.1: Setup
- [X] T001 Create `codex-rs/core/src/hooks/mod.rs` skeleton and register the module in `codex-rs/core/src/lib.rs`
- [X] T002 Create `codex-rs/common/src/hooks/mod.rs` skeleton and re-export from `codex-rs/common/src/lib.rs`
- [X] T003 Add hook dependencies (`regex`, `serde_json`, `tokio` features, `wildmatch` flags) in `codex-rs/core/Cargo.toml` and `codex-rs/common/Cargo.toml`
- [X] T004 Scaffold CLI source and test directories for hook commands in `codex-cli/src/` and `codex-cli/tests/`, updating build scripts in `codex-cli/package.json`

## Phase 3.2: Tests First (TDD) ⚠️ MUST COMPLETE BEFORE 3.3
**CRITICAL: These tests MUST be written and MUST FAIL before ANY implementation.**
- [X] T005 [P] Add contract test for `hook-config-schema.yaml` in `codex-rs/core/tests/contracts/hook_config_schema.rs`
- [X] T006 [P] Add contract test for `hook-payload-schema.json` in `codex-rs/core/tests/contracts/hook_payload_schema.rs`
- [X] T007 [P] Add CLI contract test per `hooks-cli.md` in `codex-cli/tests/hooks_cli.contract.test.ts`
- [X] T008 [P] Add integration test for PreToolUse guard denial in `codex-rs/core/tests/hooks_pretool_guard.rs`
- [X] T009 [P] Add integration test for `codex hooks exec-log` tail flow in `codex-cli/tests/hooks_exec_log.integration.test.ts`
- [X] T010 [P] Add integration test for legacy notify synthesis in `codex-rs/core/tests/hooks_notify_compat.rs`

## Phase 3.3: Core Implementation (ONLY after tests are failing)
### Data Models
- [X] T011 [P] Define `HookDefinition` struct with serde helpers in `codex-rs/common/src/hooks/definition.rs`
- [X] T012 [P] Define `HookMatchers` and `Matcher` enums in `codex-rs/common/src/hooks/matchers.rs`
- [X] T013 [P] Define `HookEventPayload` JSON model in `codex-rs/common/src/hooks/payload.rs`
- [X] T014 [P] Define `HookDecision` outcome enum in `codex-rs/common/src/hooks/decision.rs`
- [X] T015 [P] Define `HookExecutionRecord` log struct in `codex-rs/common/src/hooks/execution_record.rs`
- [X] T016 [P] Define `HookScope` precedence enum in `codex-rs/common/src/hooks/scope.rs`
- [X] T017 [P] Define `HookRegistry` container in `codex-rs/core/src/hooks/registry.rs`
- [X] T018 [P] Define `HookLayerSummary` model in `codex-rs/core/src/hooks/layer_summary.rs`
- [X] T019 [P] Define `SkippedHook` record and reasons in `codex-rs/core/src/hooks/skipped.rs`

### Services & Core Logic
- [X] T020 Implement layered TOML loader in `codex-rs/core/src/hooks/config_loader.rs`
- [X] T021 Implement registry builder and precedence evaluation in `codex-rs/core/src/hooks/registry.rs`
- [X] T022 Implement schema version validation utilities in `codex-rs/core/src/hooks/schema_registry.rs`
- [X] T023 Implement async `HookExecutor` runner with timeout handling in `codex-rs/core/src/hooks/executor.rs`
- [X] T024 Implement JSONL log writer for execution records in `codex-rs/core/src/hooks/log_writer.rs`
- [X] T025 Extend protocol types with hook RPC messages in `codex-rs/protocol/src/protocol.rs`
- [ ] T026 Implement backend hook RPC handlers in `codex-rs/core/src/client.rs`
- [ ] T027 Add CLI IPC helper for hook RPCs in `codex-cli/src/ipc/hooks.ts`
- [ ] T028 [P] Implement `codex hooks list` command in `codex-cli/src/commands/hooks/list.ts`
- [ ] T029 [P] Implement `codex hooks exec-log` command in `codex-cli/src/commands/hooks/exec-log.ts`
- [ ] T030 [P] Implement `codex hooks validate` command in `codex-cli/src/commands/hooks/validate.ts`
- [ ] T031 [P] Implement `codex hooks reload` command in `codex-cli/src/commands/hooks/reload.ts`
- [ ] T032 Register hook command namespace in `codex-cli/bin/codex.js`

## Phase 3.4: Integration
- [ ] T033 Wire hook config loader into runtime config pipeline in `codex-rs/core/src/config.rs`
- [ ] T034 Connect PreToolUse/PostToolUse flow to `HookExecutor` in `codex-rs/core/src/exec.rs`
- [ ] T035 Connect user prompt and session lifecycle events to hooks in `codex-rs/core/src/conversation_manager.rs`
- [ ] T036 Synthesize legacy notify configuration as Notification hook in `codex-rs/core/src/user_notification.rs`
- [ ] T037 Stream execution results to JSONL log writer from `codex-rs/core/src/hooks/executor.rs`
- [ ] T038 Surface hook registry data to TUI app state in `codex-rs/tui/src/app.rs`
- [ ] T039 Render hook inspector panel using Stylize helpers in `codex-rs/tui/src/render/hooks_panel.rs`

## Phase 3.5: Polish
- [ ] T040 [P] Add unit tests for matcher and scope precedence in `codex-rs/common/tests/hook_matchers.rs`
- [ ] T041 [P] Add CLI snapshot tests for `codex hooks list` output in `codex-cli/tests/__snapshots__/hooks_list.snap.ts`
- [ ] T042 [P] Update documentation in `docs/config.md` and create `docs/hooks.md` covering configuration, CLI, and logs
- [ ] T043 [P] Add benchmark ensuring HookExecutor latency <50 ms in `codex-rs/core/benches/hook_executor.rs`

## Dependencies
- T005–T010 depend on setup tasks T001–T004; all tests must be green (or failing as expected) before starting T011
- T011–T019 depend on module scaffolding T001–T004 and unblock services T020–T024
- T020 depends on T011–T019; T021 depends on T020; T022 depends on T020
- T023 depends on T020–T022; T024 depends on T023; T037 depends on T023 and T024
- T025 depends on T011–T019; T026 depends on T025; T027 depends on T026; T028–T031 depend on T027; T032 depends on T028–T031
- T033 depends on T020 and T021; T034 depends on T023 and T033; T035 depends on T023 and T033; T036 depends on T023 and T033; T038 depends on T033 and T034; T039 depends on T038
- Polish tasks T040–T043 depend on all prior phases completing

## Parallel Execution Examples
```
# Run contract and integration test authoring together once setup is done
codex tasks run --id T005 &
codex tasks run --id T006 &
codex tasks run --id T007 &
codex tasks run --id T008 &
codex tasks run --id T009 &
codex tasks run --id T010 &
wait

# Implement CLI commands in parallel after IPC helper exists
codex tasks run --id T028 &
codex tasks run --id T029 &
codex tasks run --id T030 &
codex tasks run --id T031 &
wait
```

## Notes
- Respect `just fmt` / `just fix -p <crate>` after Rust changes and `pnpm test` for CLI updates
- Re-run `cargo test -p codex-core`, `cargo test -p codex-tui`, and `pnpm test` before marking integration tasks complete
- Review `cargo insta pending-snapshots -p codex-tui` output before accepting TUI snapshot updates
