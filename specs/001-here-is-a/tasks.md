# Tasks: Custom Slash Commands for Codex CLI

**Input**: Design documents from `/specs/001-here-is-a/`
**Prerequisites**: plan.md (required), research.md, data-model.md, contracts/

## Execution Flow (main)
```
1. Load plan.md from feature directory
   → Tech stack: Rust 1.75+, serde, tokio, clap, ratatui, existing codex crates
   → Structure: Single new crate `codex-slash-commands` in workspace
2. Load design documents:
   → data-model.md: 6 entities (Command, CommandScope, CommandRegistry, etc.)
   → contracts/: API functions for registry, parsing, interpolation
   → research.md: Surgical implementation approach with minimal intervention
3. Generate tasks by category:
   → Setup: Feature flag, crate creation, dependencies
   → Tests: API contract tests, security tests, integration tests
   → Core: Data models, parsing, interpolation, registry
   → Integration: REPL hook, TUI help extension
   → Polish: Error handling, performance, documentation
4. Apply task rules:
   → Different files = mark [P] for parallel
   → Same file = sequential (no [P])
   → Tests before implementation (TDD)
5. Number tasks sequentially (T001-T032)
6. Focus on surgical integration with minimal codebase changes
```

## Format: `[ID] [P?] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- Include exact file paths in descriptions

## Path Conventions
- **Single project**: New crate `codex-rs/slash-commands/` in existing workspace
- Tests in `codex-rs/slash-commands/tests/`
- Integration with existing `codex-rs/core/`, `codex-rs/tui/`, `codex-rs/cli/`

## Phase 3.1: Setup
- [ ] T001 Create codex-slash-commands crate in codex-rs/slash-commands/
- [ ] T002 Add slash_commands feature flag to codex-rs/Cargo.toml
- [ ] T003 [P] Configure dependencies (serde, serde_yaml, dirs) in codex-rs/slash-commands/Cargo.toml

## Phase 3.2: Tests First (TDD) ⚠️ MUST COMPLETE BEFORE 3.3
**CRITICAL: These tests MUST be written and MUST FAIL before ANY implementation**
- [ ] T004 [P] Command parsing API tests in codex-rs/slash-commands/tests/test_parsing.rs
- [ ] T005 [P] Command registry API tests in codex-rs/slash-commands/tests/test_registry.rs
- [ ] T006 [P] Template interpolation API tests in codex-rs/slash-commands/tests/test_interpolation.rs
- [ ] T007 [P] Integration API tests in codex-rs/slash-commands/tests/test_integration.rs
- [ ] T008 [P] Security constraint tests in codex-rs/slash-commands/tests/test_security.rs
- [ ] T009 [P] End-to-end command execution tests in codex-rs/slash-commands/tests/test_e2e.rs

## Phase 3.3: Core Implementation (ONLY after tests are failing)
- [ ] T010 [P] Command data model in codex-rs/slash-commands/src/models/command.rs
- [ ] T011 [P] CommandScope enum in codex-rs/slash-commands/src/models/scope.rs
- [ ] T012 [P] FrontmatterMetadata struct in codex-rs/slash-commands/src/models/metadata.rs
- [ ] T013 [P] InterpolationContext struct in codex-rs/slash-commands/src/models/context.rs
- [ ] T014 CommandRegistry implementation in codex-rs/slash-commands/src/registry.rs
- [ ] T015 Frontmatter parsing logic in codex-rs/slash-commands/src/parsing.rs
- [ ] T016 Template interpolation engine in codex-rs/slash-commands/src/interpolation.rs
- [ ] T017 Command discovery and scanning in codex-rs/slash-commands/src/discovery.rs
- [ ] T018 Main library interface in codex-rs/slash-commands/src/lib.rs

## Phase 3.4: Integration
- [ ] T019 REPL input interception hook in codex-rs/core/src/repl.rs
- [ ] T020 TUI help system extension in codex-rs/tui/src/help.rs
- [ ] T021 Model override integration in codex-rs/core/src/session.rs
- [ ] T022 CLI feature flag integration in codex-rs/cli/src/main.rs

## Phase 3.5: Polish
- [ ] T023 [P] Error handling and user feedback in codex-rs/slash-commands/src/errors.rs
- [ ] T024 [P] Performance optimizations in codex-rs/slash-commands/src/performance.rs
- [ ] T025 [P] Namespace and conflict resolution in codex-rs/slash-commands/src/namespace.rs
- [ ] T026 [P] Configuration and environment variables in codex-rs/slash-commands/src/config.rs
- [ ] T027 [P] TUI snapshot tests for help changes in codex-rs/tui/tests/snapshots/
- [ ] T028 [P] Tab completion integration in codex-rs/tui/src/completion.rs
- [ ] T029 Manual testing against quickstart.md scenarios
- [ ] T030 Security validation (no shell execution, no file inclusion)
- [ ] T031 Performance benchmarks (< 1ms lookup, < 10ms interpolation)
- [ ] T032 Documentation updates for CLAUDE.md and feature usage

## Dependencies
- Setup (T001-T003) before all other phases
- Tests (T004-T009) before implementation (T010-T018)
- Core models (T010-T013) before registry and parsing (T014-T017)
- T014 (registry) blocks T019 (REPL integration)
- T015 (parsing) blocks T016 (interpolation)
- T018 (lib interface) blocks integration phase (T019-T022)
- Integration (T019-T022) before polish (T023-T032)

## Parallel Example
```
# Launch T004-T009 together (all test files):
Task: "Command parsing API tests in codex-rs/slash-commands/tests/test_parsing.rs"
Task: "Command registry API tests in codex-rs/slash-commands/tests/test_registry.rs"
Task: "Template interpolation API tests in codex-rs/slash-commands/tests/test_interpolation.rs"
Task: "Integration API tests in codex-rs/slash-commands/tests/test_integration.rs"
Task: "Security constraint tests in codex-rs/slash-commands/tests/test_security.rs"
Task: "End-to-end command execution tests in codex-rs/slash-commands/tests/test_e2e.rs"

# Launch T010-T013 together (model files):
Task: "Command data model in codex-rs/slash-commands/src/models/command.rs"
Task: "CommandScope enum in codex-rs/slash-commands/src/models/scope.rs"
Task: "FrontmatterMetadata struct in codex-rs/slash-commands/src/models/metadata.rs"
Task: "InterpolationContext struct in codex-rs/slash-commands/src/models/context.rs"
```

## Notes
- [P] tasks = different files, no dependencies
- Verify tests fail before implementing
- Use feature flag `#[cfg(feature = "slash_commands")]` throughout
- Maintain surgical integration approach - minimal changes to existing code
- Focus on upstream merge compatibility
- Follow existing Rust conventions and tooling patterns

## Task Generation Rules
*Applied during main() execution*

1. **From Contracts**:
   - API functions → contract test tasks [P]
   - Each major API area → implementation task

2. **From Data Model**:
   - Each entity → model creation task [P]
   - Registry and relationships → service layer tasks

3. **From Quickstart Scenarios**:
   - Command discovery → integration test
   - Template interpolation → integration test
   - Model override → integration test
   - Namespace handling → integration test

4. **Ordering**:
   - Setup → Tests → Models → Core Logic → Integration → Polish
   - Feature-flagged implementation throughout

## Validation Checklist
*GATE: Checked by main() before returning*

- [x] All API contracts have corresponding tests
- [x] All entities have model tasks
- [x] All tests come before implementation
- [x] Parallel tasks truly independent (different files)
- [x] Each task specifies exact file path
- [x] No task modifies same file as another [P] task
- [x] Feature flag integration included
- [x] Surgical implementation approach maintained
- [x] Security constraints addressed (no shell/file execution)
- [x] Performance requirements addressed
- [x] TUI integration for help and completion included

## Implementation Strategy Notes

**Surgical Integration Points**:
- Single hook in REPL input processing for command interception
- Minimal TUI changes for help system extension and tab completion
- Feature flag guards all new functionality
- Independent crate with clear boundaries

**Security Requirements**:
- Template processing only (no shell execution)
- No file inclusion beyond initial template discovery
- Validate all user input during interpolation
- Test security constraints comprehensively

**Performance Goals**:
- O(1) command lookup via HashMap
- Lazy loading to avoid CLI startup impact
- Memory proportional to number of commands
- Fast template interpolation (< 10ms for 10KB templates)

**Quality Assurance**:
- TDD approach with failing tests first
- Comprehensive test coverage including edge cases
- Snapshot tests for TUI changes
- End-to-end validation against quickstart scenarios