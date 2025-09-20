
# Implementation Plan: Custom Slash Commands for Codex CLI

**Branch**: `001-here-is-a` | **Date**: 2025-01-16 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/home/iatzmon/workspace/codex/specs/001-here-is-a/spec.md`

## Execution Flow (/plan command scope)
```
1. Load feature spec from Input path
   → If not found: ERROR "No feature spec at {path}"
2. Fill Technical Context (scan for NEEDS CLARIFICATION)
   → Detect Project Type from context (web=frontend+backend, mobile=app+api)
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
Add custom slash commands to Codex CLI that read Markdown templates from `.codex/commands` directories, support argument interpolation, model override, and namespacing - implemented as a surgical extension with minimal codebase intervention for upstream merge compatibility.

## Technical Context
**Language/Version**: Rust 1.75+ (existing codebase standard)
**Primary Dependencies**: serde (YAML/JSON), tokio (async), clap (CLI), ratatui (TUI), existing codex-core crates
**Storage**: File system only (.codex/commands directories, in-memory command registry cache)
**Testing**: cargo test, cargo insta (snapshot testing for TUI), existing test patterns from codex-rs
**Target Platform**: Cross-platform (macOS, Linux, Windows/WSL2) - same as existing Codex CLI
**Project Type**: Single crate extension (new codex-slash-commands crate in existing workspace)
**Performance Goals**: O(1) command lookup, O(n) argument interpolation, minimal REPL startup impact
**Constraints**: No network access, no shell execution, no file inclusion, upstream merge compatibility, feature-flagged
**Scale/Scope**: Hundreds of custom commands per user/project, nested namespaces up to 5 levels deep

**Implementation Strategy**: Our goal is to implement this surgically as an extension of the codebase with minimal intervention into existing code as this is a fork and we'd like to be able to merge from upstream easily. We should maintain the tech stack and convention already in place.

## Constitution Check
*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Security-First Architecture ✅
- **PASS**: Feature explicitly excludes shell execution and file inclusion
- **PASS**: Commands are read-only template operations with no network access
- **PASS**: No modification of CODEX_SANDBOX_* environment variables
- **PASS**: Default read-only with explicit escalation model maintained

### II. Library-Centric Design ✅
- **PASS**: New `codex-slash-commands` crate follows naming convention
- **PASS**: Core logic isolated in library, CLI integration minimal
- **PASS**: Self-contained with clear single purpose
- **PASS**: Independent testability maintained

### III. Test-Driven Quality ✅
- **PASS**: Will implement comprehensive test coverage
- **PASS**: Snapshot testing for TUI changes using cargo insta
- **PASS**: Tests fail first, implementation follows
- **PASS**: Project-specific tests before workspace-wide

### IV. Rust Standards & Tooling ✅
- **PASS**: Follows existing cargo fmt/clippy standards
- **PASS**: Format strings with inline variables
- **PASS**: Uses `just fmt` and `just fix` workflow
- **PASS**: Maintains existing tooling patterns

### V. User Experience Excellence ✅
- **PASS**: Concise, friendly command interaction
- **PASS**: Rich configuration via environment variables
- **PASS**: Maintains existing REPL user experience
- **PASS**: Clear help and autocomplete support

## Project Structure

### Documentation (this feature)
```
specs/[###-feature]/
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

**Structure Decision**: Option 1 (Single project) - Adding new crate to existing Rust workspace

## Phase 0: Outline & Research
1. **Extract unknowns from Technical Context** above:
   - For each NEEDS CLARIFICATION → research task
   - For each dependency → best practices task
   - For each integration → patterns task

2. **Generate and dispatch research agents**:
   ```
   For each unknown in Technical Context:
     Task: "Research {unknown} for {feature context}"
   For each technology choice:
     Task: "Find best practices for {tech} in {domain}"
   ```

3. **Consolidate findings** in `research.md` using format:
   - Decision: [what was chosen]
   - Rationale: [why chosen]
   - Alternatives considered: [what else evaluated]

**Output**: research.md with all NEEDS CLARIFICATION resolved

## Phase 1: Design & Contracts
*Prerequisites: research.md complete*

1. **Extract entities from feature spec** → `data-model.md`:
   - Entity name, fields, relationships
   - Validation rules from requirements
   - State transitions if applicable

2. **Generate API contracts** from functional requirements:
   - For each user action → endpoint
   - Use standard REST/GraphQL patterns
   - Output OpenAPI/GraphQL schema to `/contracts/`

3. **Generate contract tests** from contracts:
   - One test file per endpoint
   - Assert request/response schemas
   - Tests must fail (no implementation yet)

4. **Extract test scenarios** from user stories:
   - Each story → integration test scenario
   - Quickstart test = story validation steps

5. **Update agent file incrementally** (O(1) operation):
   - Run `.specify/scripts/bash/update-agent-context.sh claude` for your AI assistant
   - If exists: Add only NEW tech from current plan
   - Preserve manual additions between markers
   - Update recent changes (keep last 3)
   - Keep under 150 lines for token efficiency
   - Output to repository root

**Output**: data-model.md, /contracts/*, failing tests, quickstart.md, agent-specific file

## Phase 2: Task Planning Approach
*This section describes what the /tasks command will do - DO NOT execute during /plan*

**Task Generation Strategy**:
- Load `.specify/templates/tasks-template.md` as base
- Generate tasks from Phase 1 design docs (contracts, data model, quickstart)
- Each API function in contracts → unit test task [P]
- Each entity in data model → struct/enum definition task [P]
- Each user scenario → integration test task
- TUI integration tasks for help system and completion
- REPL integration task for command interception
- Implementation tasks to make failing tests pass

**Ordering Strategy**:
- TDD order: Tests before implementation
- Foundation first: Core data types, then parsing, then registry, then integration
- Independent crates can be developed in parallel [P]
- TUI/REPL integration comes after core functionality
- Feature flag setup early to enable conditional compilation

**Specific Task Categories**:
1. **Setup Tasks**: Crate creation, feature flag configuration, dependency setup
2. **Core Library Tasks [P]**: Data models, parsing logic, interpolation engine, registry
3. **Test Tasks [P]**: Unit tests for each core component, security constraint tests
4. **Integration Tasks**: REPL hook, TUI help extension, model override integration
5. **Validation Tasks**: End-to-end testing, quickstart validation, error handling

**Estimated Output**: 28-32 numbered, ordered tasks in tasks.md

**IMPORTANT**: This phase is executed by the /tasks command, NOT by /plan

## Phase 3+: Future Implementation
*These phases are beyond the scope of the /plan command*

**Phase 3**: Task execution (/tasks command creates tasks.md)  
**Phase 4**: Implementation (execute tasks.md following constitutional principles)  
**Phase 5**: Validation (run tests, execute quickstart.md, performance validation)

## Complexity Tracking
*Fill ONLY if Constitution Check has violations that must be justified*

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |


## Progress Tracking
*This checklist is updated during execution flow*

**Phase Status**:
- [x] Phase 0: Research complete (/plan command)
- [x] Phase 1: Design complete (/plan command)
- [x] Phase 2: Task planning complete (/plan command - describe approach only)
- [ ] Phase 3: Tasks generated (/tasks command)
- [ ] Phase 4: Implementation complete
- [ ] Phase 5: Validation passed

**Gate Status**:
- [x] Initial Constitution Check: PASS
- [x] Post-Design Constitution Check: PASS
- [x] All NEEDS CLARIFICATION resolved
- [x] Complexity deviations documented (none required)

---
*Based on Constitution v2.1.1 - See `/memory/constitution.md`*
