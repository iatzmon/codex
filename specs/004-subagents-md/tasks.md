# Tasks: Subagents Parity for Codex CLI

**Input**: Design documents from `/home/iatzmon/workspace/codex/specs/004-subagents-md/`
**Prerequisites**: plan.md (required), research.md, data-model.md, contracts/, quickstart.md

## Execution Flow (main)
```
1. Load plan.md and capture stack boundaries (Rust core vs Node CLI vs TUI) and feature flag constraints.
   → If missing: ERROR "No implementation plan found".
2. Load additional design artifacts as available:
   → data-model.md → enumerate entities for model tasks.
   → contracts/ → map each Markdown contract to a contract test task.
   → research.md → pull configuration, precedence, and logging decisions for setup/service tasks.
   → quickstart.md → derive integration scenarios for end-to-end tests.
3. Generate tasks by category:
   → Setup: workspace prep and scaffolding to keep tests compiling.
   → Tests: contract + integration coverage (failing first).
   → Core: entity models, services, CLI endpoints in Rust.
   → Integration: Node CLI IPC, TUI rendering, logging, sandbox wiring.
   → Polish: unit coverage, docs, final verification runs.
4. Apply task rules:
   → Different files ⇒ mark [P] for parallel execution.
   → Same file ⇒ sequential (no [P]).
   → Tests land before implementation (TDD enforced).
   → Models precede services; services precede CLI endpoints.
5. Number tasks sequentially (T001, T002, …) and update dependency map.
6. Provide parallel execution examples with runnable `/tasks run` commands.
7. Validate completeness before returning SUCCESS:
   → All contracts mapped to tests.
   → All entities mapped to model tasks.
   → Integration scenarios represented.
   → Paths are absolute.
```

## Format: `[ID] [P?] Description`
- `[P]` marks tasks that can run in parallel because they touch independent files.
- Include the exact absolute file path(s) or command location for every task.

## Path Conventions
- Workspace root: `/home/iatzmon/workspace/codex`.
- Rust core lives in `/home/iatzmon/workspace/codex/codex-rs/core/src`.
- Rust CLI lives in `/home/iatzmon/workspace/codex/codex-rs/cli/src`.
- TUI crate lives in `/home/iatzmon/workspace/codex/codex-rs/tui/src` with tests under `/home/iatzmon/workspace/codex/codex-rs/tui/tests`.
- Node CLI commands/IPCs live in `/home/iatzmon/workspace/codex/codex-cli/src` with tests in `/home/iatzmon/workspace/codex/codex-cli/tests`.

## Phase 3.1: Setup
- [X] T001 Install CLI dependencies by running `pnpm install` in `/home/iatzmon/workspace/codex` to ensure TypeScript commands build before new tasks run.
- [X] T002 Scaffold subagents skeleton: add `pub mod subagents;` in `/home/iatzmon/workspace/codex/codex-rs/core/src/lib.rs`, create placeholder modules (`mod.rs`, `definition.rs`, `record.rs`, `inventory.rs`, `config.rs`, `invocation.rs`, `parser.rs`, `discovery.rs`, `builder.rs`, `auto_suggest.rs`, `runner.rs`) under `/home/iatzmon/workspace/codex/codex-rs/core/src/subagents/` with `todo!()` stubs, and seed empty test files in `/home/iatzmon/workspace/codex/codex-rs/core/tests/contracts/agents_list.rs`, `/home/iatzmon/workspace/codex/codex-rs/core/tests/contracts/subagent_invoke.rs`, `/home/iatzmon/workspace/codex/codex-rs/core/tests/contracts/auto_suggest.rs`, `/home/iatzmon/workspace/codex/codex-rs/core/tests/suite/subagents_primary_story.rs`, `/home/iatzmon/workspace/codex/codex-rs/core/tests/suite/subagents_tool_restrictions.rs`, and `/home/iatzmon/workspace/codex/codex-rs/tui/tests/suite/subagents_quickstart.rs` with module exports wired in their respective `mod.rs` files.

## Phase 3.2: Tests First (TDD)
- [X] T003 [P] Author failing contract tests for `codex agents list` precedence and `--invalid` filtering in `/home/iatzmon/workspace/codex/codex-rs/core/tests/contracts/agents_list.rs` using `pretty_assertions::assert_eq` and the contracts JSON schema.
- [X] T004 [P] Author failing contract tests for subagent invocation (tool allowlist, model fallback, confirmation) in `/home/iatzmon/workspace/codex/codex-rs/core/tests/contracts/subagent_invoke.rs`.
- [X] T005 [P] Author failing contract tests for auto-suggestion confirmation and manual mode gating in `/home/iatzmon/workspace/codex/codex-rs/core/tests/contracts/auto_suggest.rs`.
- [X] T006 [P] Add integration test covering the primary user story (project override + invocation summary) in `/home/iatzmon/workspace/codex/codex-rs/core/tests/suite/subagents_primary_story.rs` orchestrating CLI flows against fixture agents.
- [X] T007 [P] Add integration test for restricted tool denial (acceptance scenario 2) in `/home/iatzmon/workspace/codex/codex-rs/core/tests/suite/subagents_tool_restrictions.rs`.
- [X] T008 [P] Add TUI integration test for the quickstart walk-through (enable flag, list, run, show detail) in `/home/iatzmon/workspace/codex/codex-rs/tui/tests/suite/subagents_quickstart.rs`.

## Phase 3.3: Core Implementation (Rust)
- [X] T009 [P] Implement `SubagentDefinition` struct, normalization, and validation helpers in `/home/iatzmon/workspace/codex/codex-rs/core/src/subagents/definition.rs`.
- [X] T010 [P] Implement `SubagentRecord` with status transitions and effective tool/model resolution in `/home/iatzmon/workspace/codex/codex-rs/core/src/subagents/record.rs`.
- [X] T011 [P] Implement `SubagentInventory` aggregation with conflict tracking and discovery events in `/home/iatzmon/workspace/codex/codex-rs/core/src/subagents/inventory.rs`.
- [X] T012 [P] Implement `SubagentConfig` defaults and accessors in `/home/iatzmon/workspace/codex/codex-rs/core/src/subagents/config.rs`.
- [X] T013 [P] Implement `InvocationSession` isolation metadata and transcript references in `/home/iatzmon/workspace/codex/codex-rs/core/src/subagents/invocation.rs`.
- [X] T014 Build Markdown+YAML parser for subagent files with descriptive validation errors in `/home/iatzmon/workspace/codex/codex-rs/core/src/subagents/parser.rs`.
- [X] T015 Build discovery loader scanning project `.codex/agents/` and user `~/.codex/agents/` directories in `/home/iatzmon/workspace/codex/codex-rs/core/src/subagents/discovery.rs` with feature-flag gating.
- [X] T016 Implement precedence resolver and inventory builder with structured logging in `/home/iatzmon/workspace/codex/codex-rs/core/src/subagents/builder.rs`.
- [X] T017 Implement auto-suggestion matcher with confidence scoring and manual-mode short circuit in `/home/iatzmon/workspace/codex/codex-rs/core/src/subagents/auto_suggest.rs`.
- [X] T018 Implement invocation runner enforcing tool allowlists, model overrides, and session persistence in `/home/iatzmon/workspace/codex/codex-rs/core/src/subagents/runner.rs`.
- [X] T019 Extend config loader to parse `subagents.enabled`, `subagents.default_model`, and `subagents.discovery` into `SubagentConfig` in `/home/iatzmon/workspace/codex/codex-rs/core/src/config.rs` and `/home/iatzmon/workspace/codex/codex-rs/core/src/config_types.rs`.
- [X] T020 Wire auto-suggestion prompts and explicit invocation handling into `/home/iatzmon/workspace/codex/codex-rs/core/src/conversation_manager.rs` using the new subagents APIs.
- [X] T021 Add `Agents` Clap subcommand (list/run/show) in `/home/iatzmon/workspace/codex/codex-rs/cli/src/main.rs` and thread config overrides.
- [X] T022 Implement CLI handlers invoking core APIs and rendering JSON/TTY output in `/home/iatzmon/workspace/codex/codex-rs/cli/src/agents.rs` (and register the module in `/home/iatzmon/workspace/codex/codex-rs/cli/src/lib.rs` if needed).

## Phase 3.4: Integration (CLI, IPC, TUI, Logging)
- [X] T023 [P] Add IPC bridge for subagent operations in `/home/iatzmon/workspace/codex/codex-cli/src/ipc/subagents.ts` calling the Rust CLI binary and normalizing responses.
- [X] T024 [P] Implement `codex agents list` command in `/home/iatzmon/workspace/codex/codex-cli/src/commands/agents/list.ts` with text and JSON output support.
- [X] T025 [P] Implement `codex agents run` command in `/home/iatzmon/workspace/codex/codex-cli/src/commands/agents/run.ts` handling confirmation prompts and tool overrides.
- [X] T026 [P] Implement `codex agents show` command in `/home/iatzmon/workspace/codex/codex-cli/src/commands/agents/show.ts` including detail transcript retrieval.
- [X] T027 Update exports and CLI surface by editing `/home/iatzmon/workspace/codex/codex-cli/src/index.ts` (and `/home/iatzmon/workspace/codex/codex-cli/package.json` bin mappings if required) to expose the new agents commands.
- [X] T028 [P] Add Node-based integration tests for agents commands in `/home/iatzmon/workspace/codex/codex-cli/tests/agents.test.ts` covering list/run/show flows.
- [X] T029 Update TUI history/detail rendering to show subagent scope, overrides, and detail links in `/home/iatzmon/workspace/codex/codex-rs/tui/src/history_cell.rs`.
- [X] T030 Update TUI chat widget to surface auto-suggestion prompts and confirmation flow in `/home/iatzmon/workspace/codex/codex-rs/tui/src/chatwidget.rs` and related components.
- [X] T031 Ensure sandbox guards apply during subagent invocation by updating `/home/iatzmon/workspace/codex/codex-rs/core/src/spawn.rs` to propagate `CODEX_SANDBOX` / `CODEX_SANDBOX_NETWORK_DISABLED` checks for subagent sessions.
- [X] T032 Emit structured logs for discovery, overrides, suggestions, and invocation summaries in `/home/iatzmon/workspace/codex/codex-rs/core/src/event_mapping.rs` (or a dedicated subagents logging module linked from there).

## Phase 3.5: Polish
- [X] T033 [P] Add regression tests for config fallback ordering in `/home/iatzmon/workspace/codex/codex-rs/core/tests/subagents_config.rs` (cover enabled=false short circuit and default model hierarchy).
- [X] T034 [P] Add regression tests for tool access denials and session transcript persistence in `/home/iatzmon/workspace/codex/codex-rs/core/tests/subagents_invocation.rs`.
- [X] T035 [P] Document subagent workflow in `/home/iatzmon/workspace/codex/docs/subagents.md` and update `/home/iatzmon/workspace/codex/AGENTS.md` with CLI usage and precedence notes.
- [X] T036 [P] Refresh getting started flow with subagent quickstart steps in `/home/iatzmon/workspace/codex/docs/getting-started.md` and cross-link the new docs page.
- [X] T037 Run end-to-end verification: `cargo test -p codex-core`, `cargo test -p codex-tui`, `cargo test --all-features`, and `pnpm build` from `/home/iatzmon/workspace/codex`.

## Phase 3.6: Tool Migration
- [X] T038 Wire subagent inventory into `invoke_subagent` tool registration by updating `/home/iatzmon/workspace/codex/codex-rs/core/src/codex.rs` and `/home/iatzmon/workspace/codex/codex-rs/core/src/openai_tools.rs` to surface metadata for all discovered subagents each turn.
- [X] T039 Handle `invoke_subagent` tool calls inside Codex by parsing arguments, delegating to `SubagentRunner`, and returning structured results; edit `/home/iatzmon/workspace/codex/codex-rs/core/src/codex.rs` and supporting subagent modules as needed.
- [X] T040 Remove keyword-based auto-suggestion flows from `/home/iatzmon/workspace/codex/codex-rs/core/src/conversation_manager.rs` and TUI components under `/home/iatzmon/workspace/codex/codex-rs/tui/src/` so subagents operate solely through the tool API.
- [X] T041 Update automated coverage in `/home/iatzmon/workspace/codex/codex-rs/core/tests/` and `/home/iatzmon/workspace/codex/codex-rs/tui/tests/` plus documentation under `/home/iatzmon/workspace/codex/docs/` to reflect tool-driven subagents.
- [X] T042 Re-run validation commands (`cargo test -p codex-core`, `cargo test -p codex-tui`, `cargo test --all-features`, `pnpm build`) from `/home/iatzmon/workspace/codex` after migrations.

## Dependencies
- T002 depends on T001.
- Tests T003–T008 depend on scaffolding T002.
- Core implementation tasks T009–T022 consume failing tests (T003–T008) and should execute after they exist.
- Config wiring T019 blocks conversation integration T020 and CLI handlers T021–T022.
- TypeScript IPC/command tasks T023–T027 require Rust CLI endpoints (T021–T022).
- T029–T032 rely on core services (T009–T020) being available.
- Polish tasks T033–T037 execute after core + integration tasks complete.
- Tool migration tasks T038–T041 execute after prior subagent infrastructure (T009–T032) and polish tasks (T033–T037) are in place.
- Validation task T042 depends on completing T038–T041.

## Parallel Example
```
# Launch contract and integration tests together once scaffolding exists:
/tasks run --feature 004-subagents-md --id T003
/tasks run --feature 004-subagents-md --id T004
/tasks run --feature 004-subagents-md --id T005
/tasks run --feature 004-subagents-md --id T006
/tasks run --feature 004-subagents-md --id T007
/tasks run --feature 004-subagents-md --id T008
```

## Notes
- Marked [P] tasks touch distinct files and can execute concurrently once dependencies are met.
- Keep Rust code formatted with `just fmt` and run `just fix -p <crate>` as needed before T037.
- New docs should follow existing style guides and link from relevant indexes.
- Verify new binaries respect existing sandbox environment variables and do not bypass logging pipelines.
