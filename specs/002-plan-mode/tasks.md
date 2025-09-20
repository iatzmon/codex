# Tasks: Codex CLI Plan Mode Read-Only Planning State

**Input**: Design documents from `/home/iatzmon/workspace/codex/specs/002-plan-mode/`
**Prerequisites**: plan.md (required), research.md, data-model.md, contracts/, quickstart.md

## Execution Flow (main)
```
1. Load plan.md from feature directory
   → If not found: ERROR "No implementation plan found"
   → Extract: tech stack, libraries, structure
2. Load optional design documents:
   → data-model.md: Extract entities → model tasks
   → contracts/: Each file → contract test task
   → research.md: Extract decisions → setup tasks
   → quickstart.md: Map scenarios → integration tests
3. Generate tasks by category:
   → Setup: module scaffolding, config hooks, linting prerequisites
   → Tests: contract tests, integration tests
   → Core: models, config, services, CLI commands
   → Integration: TUI, telemetry, tool gating
   → Polish: unit tests, performance, docs
4. Apply task rules:
   → Different files = mark [P] for parallel
   → Same file = sequential (no [P])
   → Tests before implementation (TDD)
5. Number tasks sequentially (T001, T002...)
6. Generate dependency guidance
7. Provide parallel execution examples with `codex tasks run`
8. Validate task completeness:
   → All contracts have tests
   → All entities have model tasks
   → All endpoints implemented
9. Return: SUCCESS (tasks ready for execution)
```

## Format: `[ID] [P?] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- Include exact absolute file paths in descriptions

## Path Conventions
- Workspace root: `/home/iatzmon/workspace/codex/codex-rs`
- Feature specs: `/home/iatzmon/workspace/codex/specs/002-plan-mode`
- All task paths below are absolute

## Phase 3.1: Setup
- [ ] T001 Initialize Plan Mode module skeleton in /home/iatzmon/workspace/codex/codex-rs/core/src/lib.rs and /home/iatzmon/workspace/codex/codex-rs/core/src/plan_mode/mod.rs to expose session, artifact, entry, capability, telemetry, and config submodules.
- [ ] T002 Add Plan Mode CLI flag placeholders by updating /home/iatzmon/workspace/codex/codex-rs/tui/src/cli.rs, /home/iatzmon/workspace/codex/codex-rs/cli/src/main.rs, and /home/iatzmon/workspace/codex/codex-rs/common/src/config_override.rs so `--plan` and `plan_mode.*` overrides parse without enabling behavior.
- [ ] T003 Seed Plan Mode integration test scaffolding by adding `mod plan_mode;` to /home/iatzmon/workspace/codex/codex-rs/core/tests/suite/mod.rs and creating stub modules in /home/iatzmon/workspace/codex/codex-rs/core/tests/suite/plan_mode/{mod.rs,activation.rs,refusal_file_edit.rs,refusal_shell.rs,read_only_tool.rs,template_fallback.rs,exit.rs,apply.rs,attachment_guardrail.rs}.

## Phase 3.2: Tests First (TDD) ⚠️ MUST COMPLETE BEFORE 3.3
- [ ] T004 [P] Author contract test covering `/commands/plan`, `/commands/exit-plan`, `/commands/apply-plan`, and `/events/plan-update` from /home/iatzmon/workspace/codex/specs/002-plan-mode/contracts/plan-mode.yaml in /home/iatzmon/workspace/codex/codex-rs/cli/tests/plan_mode_contract.rs using pretty_assertions snapshots.
- [ ] T005 [P] Add failing integration test for Plan Mode activation flow (Quickstart step 1) in /home/iatzmon/workspace/codex/codex-rs/core/tests/suite/plan_mode/activation.rs verifying PLAN state is entered and telemetry recorded.
- [ ] T006 [P] Add failing integration test for write-operation refusal (Quickstart step 2) in /home/iatzmon/workspace/codex/codex-rs/core/tests/suite/plan_mode/refusal_file_edit.rs asserting file edits become plan entries.
- [ ] T007 [P] Add failing integration test for shell execution refusal (Quickstart step 3) in /home/iatzmon/workspace/codex/codex-rs/core/tests/suite/plan_mode/refusal_shell.rs ensuring commands are blocked and captured in the plan artifact.
- [ ] T008 [P] Add failing integration test for read-only tool allowance (Quickstart step 4) in /home/iatzmon/workspace/codex/codex-rs/core/tests/suite/plan_mode/read_only_tool.rs confirming file read tools still work while plan entries accumulate.
- [ ] T009 [P] Add failing integration test for missing template fallback warning (Quickstart step 5) in /home/iatzmon/workspace/codex/codex-rs/core/tests/suite/plan_mode/template_fallback.rs validating warning and default tooltip guidance.
- [ ] T010 [P] Add failing integration test for `/exit-plan` (Quickstart step 6) in /home/iatzmon/workspace/codex/codex-rs/core/tests/suite/plan_mode/exit.rs asserting approval mode restoration and PLAN badge removal events.
- [ ] T011 [P] Add failing integration test for `/apply-plan` happy path (Quickstart step 7) in /home/iatzmon/workspace/codex/codex-rs/core/tests/suite/plan_mode/apply.rs asserting plan artifact injection and approval mode override behavior.
- [ ] T012 [P] Add failing integration test for external attachment guardrail (Quickstart step 8) in /home/iatzmon/workspace/codex/codex-rs/core/tests/suite/plan_mode/attachment_guardrail.rs ensuring out-of-workspace attachments are refused with security messaging.
- [ ] T013 [P] Add failing TUI snapshot test for PLAN badge and tooltip rendering (Quickstart step 1 UX) in /home/iatzmon/workspace/codex/codex-rs/tui/tests/suite/plan_mode_badge.rs and register module in /home/iatzmon/workspace/codex/codex-rs/tui/tests/suite/mod.rs.

## Phase 3.3: Core Implementation (ONLY after tests are failing)
- [ ] T014 [P] Implement PlanModeSession struct with state transitions and telemetry counters in /home/iatzmon/workspace/codex/codex-rs/core/src/plan_mode/session.rs.
- [ ] T015 [P] Implement PlanArtifact structure—including metadata, constraints, next actions, and tests vectors—in /home/iatzmon/workspace/codex/codex-rs/core/src/plan_mode/artifact.rs.
- [ ] T016 [P] Implement PlanEntry enum-backed record with sequencing and detail fields in /home/iatzmon/workspace/codex/codex-rs/core/src/plan_mode/entry.rs.
- [ ] T017 [P] Implement ToolCapability data model and read-only marker logic in /home/iatzmon/workspace/codex/codex-rs/core/src/plan_mode/capability.rs.
- [ ] T018 Extend Plan Mode config and protocol support by updating /home/iatzmon/workspace/codex/codex-rs/protocol/src/protocol.rs, /home/iatzmon/workspace/codex/codex-rs/protocol/src/plan_tool.rs, /home/iatzmon/workspace/codex/codex-rs/core/src/config.rs, and /home/iatzmon/workspace/codex/codex-rs/core/src/config_profile.rs to serialize PlanModeSession, PlanArtifact, and `plan_mode.*` overrides.
- [ ] T019 Implement environment gating and attachment guardrails in /home/iatzmon/workspace/codex/codex-rs/core/src/environment_context.rs and /home/iatzmon/workspace/codex/codex-rs/core/src/mcp_connection_manager.rs using ToolCapability to filter read-only tooling.
- [ ] T020 Implement plan capture and refusal pipeline by updating /home/iatzmon/workspace/codex/codex-rs/core/src/plan_tool.rs, /home/iatzmon/workspace/codex/codex-rs/core/src/shell.rs, and /home/iatzmon/workspace/codex/codex-rs/core/src/user_notification.rs to append plan entries and emit Plan Mode messaging.

## Phase 3.4: Endpoints & Integration
- [ ] T021 Implement `/commands/plan` activation workflow across /home/iatzmon/workspace/codex/codex-rs/cli/src/main.rs, /home/iatzmon/workspace/codex/codex-rs/core/src/conversation_manager.rs, and /home/iatzmon/workspace/codex/codex-rs/core/src/plan_mode/session.rs to enter Plan Mode, seed allowed tools, and emit telemetry.
- [ ] T022 Implement `/commands/exit-plan` handling in /home/iatzmon/workspace/codex/codex-rs/core/src/conversation_manager.rs and /home/iatzmon/workspace/codex/codex-rs/core/src/environment_context.rs to restore prior approval policy and clear PlanModeSession state.
- [ ] T023 Implement `/commands/apply-plan` flow in /home/iatzmon/workspace/codex/codex-rs/core/src/conversation_manager.rs, /home/iatzmon/workspace/codex/codex-rs/core/src/config.rs, and /home/iatzmon/workspace/codex/codex-rs/cli/src/main.rs to validate requested approval mode, finalize telemetry, and stage plan artifact.
- [ ] T024 Implement `/events/plan-update` ingestion by wiring plan entry payloads through /home/iatzmon/workspace/codex/codex-rs/core/src/plan_tool.rs and /home/iatzmon/workspace/codex/codex-rs/core/src/conversation_history.rs to append to the active PlanArtifact.
- [ ] T025 [P] Render PLAN badge, tooltip, and blocked-action banners in the TUI by updating /home/iatzmon/workspace/codex/codex-rs/tui/src/status_indicator_widget.rs, /home/iatzmon/workspace/codex/codex-rs/tui/src/app.rs, and associated layout modules to read PlanModeSession state.
- [ ] T026 [P] Emit Plan Mode telemetry events by updating /home/iatzmon/workspace/codex/codex-rs/core/src/event_mapping.rs, /home/iatzmon/workspace/codex/codex-rs/core/src/plan_mode/telemetry.rs, and analytics hooks so transitions and apply counts are reported.

## Phase 3.5: Polish
- [ ] T027 [P] Add focused unit tests for PlanModeSession transitions and ToolCapability filtering in /home/iatzmon/workspace/codex/codex-rs/core/src/plan_mode/{session.rs,capability.rs} using pretty_assertions.
- [ ] T028 [P] Update documentation and agent context by revising /home/iatzmon/workspace/codex/docs/plan_mode.md (create if missing), /home/iatzmon/workspace/codex/CLAUDE.md, and /home/iatzmon/workspace/codex/docs/configuration.md to describe Plan Mode usage, telemetry, and overrides.

## Dependencies
- T001 → prerequisite for all Plan Mode code tasks.
- T002 → prerequisite for T021 and T023.
- T003 → prerequisite for T004–T012.
- T004–T013 must exist and fail before starting T014–T026.
- T014–T020 must finish before T021–T026.
- T021–T024 unblock T025–T026.
- T025–T026 unblock polish tasks T027–T028.

## Parallel Example
```
# After completing setup (T001–T003), run independent test authoring tasks in parallel:
codex tasks run T005 &
codex tasks run T006 &
codex tasks run T007 &
codex tasks run T008 &
codex tasks run T009 &
codex tasks run T010 &
codex tasks run T011 &
codex tasks run T012 &
wait
```

## Validation Checklist
- [ ] All contract and integration tests authored before implementation
- [ ] Every data-model entity mapped to a core task
- [ ] Endpoint handlers implemented after supporting services
- [ ] PLAN badge rendered and snapshot updated
- [ ] Documentation and agent context refreshed for Plan Mode
