# Feature Specification: Subagents Parity for Codex CLI

**Feature Branch**: `[004-subagents-md]`  
**Created**: September 21, 2025  
**Status**: Draft  
**Input**: User description: "pm create-brownfield-prd for this Codex CLI fork: add a Subagents feature mirroring Claude Code with minimal, surgical changes"

## Execution Flow (main)
```
1. Parse user description from Input
   → Completed: captured goals around Claude Code parity, isolation, and minimal diff surface
2. Extract key concepts from description
   → Identified: subagent scopes (project vs user), invocation modes, configuration toggles, tool/model constraints
3. For each unclear aspect:
   → No unresolved ambiguities; assumptions avoided to respect parity mandate
4. Fill User Scenarios & Testing section
   → Drafted scenarios covering explicit invocation, overrides, and tool restrictions
5. Generate Functional Requirements
   → Authored testable statements spanning discovery, execution, configuration, and safety expectations
6. Identify Key Entities (if data involved)
   → Listed Subagent Definition and Subagent Inventory artifacts
7. Run Review Checklist
   → Checklist passes with no [NEEDS CLARIFICATION] markers
8. Return: SUCCESS (spec ready for planning)
```

---

## ⚡ Quick Guidelines
- ✅ Emphasize user value of reusable, scoped subagents with Claude Code parity
- ❌ Avoid dictating implementation details; focus on behavioral expectations and governance
- 👥 Language positioned for product stakeholders coordinating CLI enhancements

### Section Requirements
- Mandatory sections populated: User Scenarios & Testing, Requirements, Review & Acceptance Checklist, Execution Status
- Optional Key Entities included because subagents introduce structured artifacts

### For AI Generation
- All critical concepts explicitly sourced from the provided PRD; no speculative additions
- No [NEEDS CLARIFICATION] items remain because scope and behaviors are well defined in the prompt
- Testing considerations focus on isolating subagent runs and verifying precedence rules

---

## User Scenarios & Testing *(mandatory)*

### Primary User Story
A developer working in the Codex CLI wants to delegate review tasks to a named subagent defined in `.codex/agents`, ensuring the subagent uses its scoped instructions and tool permissions without disturbing the main session.

### Acceptance Scenarios
1. **Given** project and user directories each contain a subagent named "code-reviewer", **When** the developer lists agents from the REPL manager, **Then** the project-defined version appears and is flagged as overriding the user-level definition.
2. **Given** the feature flag `subagents.enabled` is on and a subagent file specifies a restricted tool list, **When** the developer invokes that subagent by name within a session, **Then** the session confirms the restricted tools and returns a summarized outcome aligned with the subagent description.

### Edge Cases
- What happens when a subagent file is missing required frontmatter fields? The system must surface a validation error and ignore the faulty definition without blocking other agents.
- How does system handle auto-suggestion when multiple subagents describe similar intents? The manager should present the top matches with rationale and allow explicit selection before execution.

## Requirements *(mandatory)*

### Functional Requirements
- **FR-001**: System MUST discover subagents from `.codex/agents/` within the project and `~/.codex/agents/` at the user scope, giving precedence to project definitions when names collide.
- **FR-002**: System MUST require each subagent file to include YAML frontmatter fields `name` and `description`, and MAY optionally honor `tools` and `model`; missing required fields invalidate that definition with user-facing feedback.
- **FR-003**: System MUST allow subagents to inherit all session tools by default and honor explicit tool allowlists per file, preventing access to tools not enumerated when restrictions are present.
- **FR-004**: System MUST support explicit invocation in prompts and via an `agents` manager command that lists, inspects, and creates subagents consistent with Claude Code workflows.
- **FR-005**: System MUST execute each subagent in an isolated context that reports a summarized result to the invoking session while retaining access to full detail through the manager UI.
- **FR-006**: System MUST enable optional auto-suggestions of relevant subagents when the main session message aligns with a subagent description, while requiring user confirmation before execution.
- **FR-007**: System MUST allow configuration via `~/.codex/config.toml`, honoring `subagents.enabled`, `subagents.default_model`, and `subagents.discovery` settings, with the feature disabled by default when the flag is off.
- **FR-008**: System MUST document and enforce precedence rules when both scopes define the same subagent name, including event logging or messaging that clarifies which definition ran.
- **FR-009**: System MUST respect the model specified in a subagent file when present, otherwise fall back to the main-session model or the configured default model hierarchy without altering unrelated behaviors.
- **FR-010**: System MUST ensure newly added subagent functionality remains isolated from existing workflows, enabling downstream merges to remain low-risk by confining new logic to the feature pathways described above.

### Key Entities *(include if feature involves data)*
- **Subagent Definition**: Markdown document with YAML frontmatter capturing name, description, optional tools, and optional model that governs how the subagent appears and behaves.
- **Subagent Inventory**: Aggregated view of discovered project and user subagents, including metadata on scope, status, precedence resolution, and validation messages for surfaced errors.

---

## Review & Acceptance Checklist
*GATE: Automated checks run during main() execution*

### Content Quality
- [x] No implementation details (languages, frameworks, APIs)
- [x] Focused on user value and business needs
- [x] Written for non-technical stakeholders
- [x] All mandatory sections completed
- [x] Constitution alignment: scope highlights CLI vs core responsibilities, testability, and logging needs for planners

### Requirement Completeness
- [x] No [NEEDS CLARIFICATION] markers remain
- [x] Requirements are testable and unambiguous
- [x] Success criteria are measurable
- [x] Scope is clearly bounded
- [x] Dependencies and assumptions identified

---

## Execution Status
*Updated by main() during processing*

- [x] User description parsed
- [x] Key concepts extracted
- [x] Ambiguities marked
- [x] User scenarios defined
- [x] Requirements generated
- [x] Entities identified
- [x] Review checklist passed

---
