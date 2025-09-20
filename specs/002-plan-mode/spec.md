# Feature Specification: Codex CLI Plan Mode Read-Only Planning State

**Feature Branch**: `002-plan-mode`  
**Created**: September 18, 2025  
**Status**: Draft  
**Input**: User description: "Introduce a read-only Plan Mode that mirrors Claude Code's planning workflow by producing structured implementation plans without executing commands, editing files, or expanding the workspace scope, while keeping existing Codex CLI modes unchanged."

## Execution Flow (main)
```
1. Parse user description from Input
   → If empty: ERROR "No feature description provided"
2. Extract key concepts from description
   → Identify: actors, actions, data, constraints
3. For each unclear aspect:
   → Mark with [NEEDS CLARIFICATION: specific question]
4. Fill User Scenarios & Testing section
   → If no clear user flow: ERROR "Cannot determine user scenarios"
5. Generate Functional Requirements
   → Each requirement must be testable
   → Mark ambiguous requirements
6. Identify Key Entities (if data involved)
7. Run Review Checklist
   → If any [NEEDS CLARIFICATION]: WARN "Spec has uncertainties"
   → If implementation details found: ERROR "Remove tech details"
8. Return: SUCCESS (spec ready for planning)
```

---

## ⚡ Quick Guidelines
- ✅ Focus on WHAT users need and WHY
- ❌ Avoid HOW to implement (no tech stack, APIs, code structure)
- 👥 Written for business stakeholders, not developers

### Section Requirements
- **Mandatory sections**: Must be completed for every feature
- **Optional sections**: Include only when relevant to the feature
- When a section doesn't apply, remove it entirely (don't leave as "N/A")

### For AI Generation
When creating this spec from a user prompt:
1. **Mark all ambiguities**: Use [NEEDS CLARIFICATION: specific question] for any assumption you'd need to make
2. **Don't guess**: If the prompt doesn't specify something (e.g., "login system" without auth method), mark it
3. **Think like a tester**: Every vague requirement should fail the "testable and unambiguous" checklist item
4. **Common underspecified areas**:
   - User types and permissions  
   - Data retention/deletion policies  
   - Performance targets and scale
   - Error handling behaviors
   - Integration requirements
   - Security/compliance needs

---

## User Scenarios & Testing *(mandatory)*

### Primary User Story
As a Codex CLI operator, I want to enter a dedicated Plan Mode that keeps the workspace read-only while I gather research, analyze requirements, and create a structured plan so that I can review and approve the plan before allowing any execution changes.

### Acceptance Scenarios
1. **Given** the operator launches Codex CLI with the `--plan` flag or issues the `/plan` command, **When** the session activates Plan Mode, **Then** the interface must show a PLAN indicator, list the read-only capabilities, and restrict the agent to planning responses only.
2. **Given** the operator is in Plan Mode and requests actions that would modify files or run commands, **When** the agent evaluates the request, **Then** the agent must refuse to execute, capture the proposal as part of the plan output, and explain that execution requires leaving Plan Mode.

### Edge Cases
- What happens when the operator triggers `/apply-plan` without specifying a destination mode? [NEEDS CLARIFICATION: should CLI prompt for a mode or default to prior mode?]
- How does the system handle loading `.codex/plan.md` when the file is missing or unreadable?
- What response is shown if the operator attempts to attach a file from outside the workspace while still in Plan Mode?

## Requirements *(mandatory)*

### Functional Requirements
- **FR-001**: The system MUST allow users to start Plan Mode via the `/plan` command and the `--plan` CLI flag, confirming the transition and outlining restrictions.
- **FR-002**: The interface MUST display a persistent PLAN badge and a concise tooltip that explains read-only behavior and exit options for the duration of Plan Mode.
- **FR-003**: The system MUST block any file write, edit, creation, deletion, or move requests while in Plan Mode, responding with guidance to exit Plan Mode before making changes.
- **FR-004**: The system MUST prevent command execution (including shell, notebook, and state-changing MCP tools) during Plan Mode and record suggested commands as plan items instead of running them.
- **FR-005**: The system MUST limit tool availability in Plan Mode to read-only capabilities such as viewing, listing, globbing, searching, and approved read-only MCP integrations.
- **FR-006**: The system MUST enforce workspace scope so that Plan Mode can only read from the active workspace and any pre-configured additional directories.
- **FR-007**: The system MUST incorporate existing plan templates from `.codex/plan.md` into the planning prompt when the file is present, without modifying the template file.
- **FR-008**: The system MUST support optional overrides in `~/.codex/config.yaml` or `~/.codex/config.json` for planning enablement, allowed read-only tools, and the model used for planning sessions.
- **FR-009**: The planner MUST produce a structured plan artifact in the transcript with the required sections: title, objectives, constraints, assumptions, approach, detailed steps, affected files or modules, test plan, risks, alternatives, rollback or mitigations, success criteria, and next actions.
- **FR-010**: The system MUST provide `/exit-plan` to return to the previous approval mode without modifying the plan content.
- **FR-011**: The system MUST provide `/apply-plan [mode]` to exit Plan Mode, inject the generated plan into the next action's context, and switch to the specified approval mode while leaving the plan unchanged.
- **FR-012**: The system MUST ensure that exiting Plan Mode restores normal behavior for all other approval modes without regressions or new blockers.
- **FR-013**: The system MUST surface a clear error when Plan Mode users attempt to include external files or attachments outside the permitted directories, instructing them to exit Plan Mode first.
- **FR-014**: The system MUST allow optional read-only web research tools in Plan Mode only when the operator has granted the necessary network approvals, and MUST continue blocking state-changing tools even after approval.
- **FR-015**: The system MUST capture any model-suggested patches or inline diffs as plan entries rather than applying them to the workspace.
- **FR-016**: The planner MUST call out assumptions, constraints, risks, and next actions explicitly within the plan artifact to aid later execution review.
- **FR-017**: The system MUST communicate how to persist a plan (for example, advising `/save-plan`) while refraining from writing files automatically during Plan Mode.

### Key Entities
- **Plan Mode Session**: Represents the state of a Codex CLI session while Plan Mode is active, including the originating approval mode, the read-only tool set, and current plan artifact.
- **Plan Artifact**: The structured planning output stored in the conversation transcript, containing required sections and queued execution steps for later approval.
- **Planning Configuration**: Optional user-defined settings and templates (such as `.codex/plan.md` and `~/.codex/config.yaml|json`) that shape Plan Mode defaults, allowed tools, and model selection.

---

## Review & Acceptance Checklist
*GATE: Automated checks run during main() execution*

### Content Quality
- [x] No implementation details (languages, frameworks, APIs)
- [x] Focused on user value and business needs
- [x] Written for non-technical stakeholders
- [x] All mandatory sections completed

### Requirement Completeness
- [ ] No [NEEDS CLARIFICATION] markers remain
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
- [ ] Review checklist passed

---
