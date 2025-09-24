# Implementation Plan: Subagents Parity for Codex CLI

**Branch**: `[004-subagents-md]` | **Date**: September 21, 2025 | **Spec**: `/home/iatzmon/workspace/codex/specs/004-subagents-md/spec.md`
**Input**: Feature specification from `/home/iatzmon/workspace/codex/specs/004-subagents-md/spec.md`

## Execution Flow (/plan command scope)
```
1. Load feature spec from Input path
   → Completed 2025-09-21: spec parsed without errors.
2. Fill Technical Context (scan for NEEDS CLARIFICATION)
   → Completed: context populated with Rust/Node stack; no unresolved items.
3. Fill the Constitution Check section based on the content of the constitution document.
   → Completed: mapped plan rules to Constitution v3.0.0 principles.
4. Evaluate Constitution Check section below
   → Result: PASS with no complexity deviations recorded.
   → Progress Tracking updated: Initial Constitution Check.
5. Execute Phase 0 → research.md
   → Completed: decisions logged in `/home/iatzmon/workspace/codex/specs/004-subagents-md/research.md`.
6. Execute Phase 1 → contracts, data-model.md, quickstart.md, agent-specific template file.
   → Completed: artifacts written under `/home/iatzmon/workspace/codex/specs/004-subagents-md/` and contracts/.
7. Re-evaluate Constitution Check section
   → Result: PASS after design; no new violations.
   → Progress Tracking updated: Post-Design Constitution Check.
8. Plan Phase 2 → Describe task generation approach (DO NOT create tasks.md)
   → Completed: approach documented and tasks enumerated in `/home/iatzmon/workspace/codex/specs/004-subagents-md/tasks.md` per user directive.
9. STOP - Ready for /tasks command
   → Status: All plan-phase artifacts generated; awaiting implementation workflows.
```

## Summary
Subagents feature delivers Claude Code parity by discovering Markdown-based definitions at project and user scopes, enforcing project override precedence, and gating execution through `subagents.*` configuration. Research concluded that extending the existing agents manager with isolated invocation sessions, structured logging, and feature flag toggles provides the minimal, surgical pathway to meet FR-001 through FR-010 without introducing new service layers.

## Technical Context
**Language/Version**: Rust 1.89.0 (`codex-rs`), TypeScript/Node.js ≥22 (`codex-cli`)  
**Primary Dependencies**: `codex-core`, `codex-cli`, Ratatui TUI stack, pnpm toolchain  
**Storage**: File-system backed `.codex/agents/` and `~/.codex/agents/` directories (Markdown with YAML frontmatter)  
**Testing**: `cargo test -p codex-core`, `cargo test -p codex-tui`, `cargo test --all-features`, `pnpm build` (CLI bundle), targeted integration tests in `codex-rs/core/tests`  
**Target Platform**: Cross-platform Codex CLI (macOS/Linux developer environments)  
**Project Type**: single  
**Performance Goals**: Maintain existing CLI responsiveness; subagent list/invoke commands complete within current interactive latency (<1s under typical inventories).  
**Constraints**: Feature flag `subagents.enabled` defaults off; changes isolated to subagent pathways; reuse existing sandbox guards; avoid new crates or cross-boundary coupling violations.  
**Scale/Scope**: Supports dozens of subagent definitions per workspace with single-session invocation; scoped to Codex CLI + core parity, no remote services introduced.

## Constitution Check
- [x] **Dual-Core Workspace Integrity**: Plan confines CLI surfaces to `codex-cli` while discovery/invocation logic resides in `codex-core`; paired tests noted for any interface changes.
- [x] **Template-Governed Flow**: Spec, plan, research, data-model, quickstart, contracts, and tasks all generated from `.specify/templates` guidance with unused sections removed.
- [x] **Test-First Assurance**: Phase 2 tasks front-load failing contract/unit/integration tests before implementation and record required `cargo`/`pnpm` runs.
- [x] **Style and Simplicity Discipline**: Plan honors `just fmt`/`just fix`, Ratatui `Stylize`, and avoids new crates or abstractions beyond subagent modules.
- [x] **Release and Observability Control**: Logging expectations captured via structured discovery/override events; documentation and PR evidence tasks included.
- **Initial Constitution Check**: PASS (2025-09-21)
- **Post-Design Constitution Check**: PASS (2025-09-21)

## Project Structure

### Documentation (this feature)
```
/home/iatzmon/workspace/codex/specs/004-subagents-md/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── agents-list.md
│   ├── subagent-invoke.md
│   └── auto-suggest.md
└── tasks.md
```

### Source Code (repository root)
```
/home/iatzmon/workspace/codex/
├── codex-cli/          # Node.js CLI wrapper (command UX, agents manager surfaces)
├── codex-rs/           # Rust core engine (discovery, invocation, config)
│   ├── core/           # Execution logic extension points for subagents
│   ├── tui/            # Ratatui UI for manager integration tests
│   └── common/         # Shared types if needed (no new crates required)
└── docs/               # User docs referencing quickstart additions
```

**Structure Decision**: Option 1 (single project) with explicit CLI/core split per Constitution Principle I.

## Phase 0: Outline & Research
- Unknowns resolved: None; spec fully defined requirements and parity expectations.
- Research artifacts: `/home/iatzmon/workspace/codex/specs/004-subagents-md/research.md` capturing discovery precedence, configuration, CLI integration, and logging decisions.
- Result: Ready for design with no outstanding clarifications.

## Phase 1: Design & Contracts
- Data model recorded in `/home/iatzmon/workspace/codex/specs/004-subagents-md/data-model.md` covering `SubagentDefinition`, `SubagentInventory`, `SubagentRecord`, `SubagentConfig`, and `InvocationSession`.
- Contracts authored under `/home/iatzmon/workspace/codex/specs/004-subagents-md/contracts/` for agents list, subagent invocation, and auto-suggestion flows with failure modes and test hooks.
- Quickstart instructions documented at `/home/iatzmon/workspace/codex/specs/004-subagents-md/quickstart.md` for enabling, defining, and invoking subagents.
- Agent context update triggered via `/home/iatzmon/workspace/codex/.specify/scripts/bash/update-agent-context.sh codex` (will re-run after final review to incorporate updated plan content).

## Phase 2: Task Planning Approach
- Tasks enumerated in `/home/iatzmon/workspace/codex/specs/004-subagents-md/tasks.md` covering TDD-first tests, discovery/CLI implementation, configuration, logging, docs, and release evidence.
- Ordering preserves TDD: tests (1-6) precede implementation (7-23), followed by validation and documentation (24-27).
- Parallelizable tasks marked implicitly via scope separation (tests per module, CLI vs core) ready for `/tasks` command execution.

## Phase 3+: Future Implementation
- Phase 3: `/tasks` command will orchestrate execution order.
- Phase 4: Implement feature guided by tasks and ensure minimal diff surface.
- Phase 5: Validate via recorded commands, quickstart, and release checklist.

## Complexity Tracking
None required; no constitutional deviations identified.

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
- [ ] Complexity deviations documented

---
*Based on Constitution v3.0.0 - See `/home/iatzmon/workspace/codex/.specify/memory/constitution.md`*
