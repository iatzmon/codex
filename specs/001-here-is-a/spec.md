# Feature Specification: Custom Slash Commands for Codex CLI

**Feature Branch**: `001-here-is-a`
**Created**: 2025-01-16
**Status**: Ready
**Input**: User description: "Here is a concise, surgical PRD to add custom slash commands to a Codex CLI fork that matches Claude Code's custom command capabilities while explicitly excluding running bash commands and including other files, keeping changes isolated to minimize upstream merge friction."

## Execution Flow (main)
```
1. Parse user description from Input
   � If empty: ERROR "No feature description provided"
2. Extract key concepts from description
   � Identified: slash commands, Markdown templates, argument interpolation, model override, directory discovery
3. For each unclear aspect:
   � No unclear aspects - PRD is comprehensive
4. Fill User Scenarios & Testing section
   � Clear user flows for command discovery, creation, and execution
5. Generate Functional Requirements
   � Each requirement is testable and derived from PRD specifications
6. Identify Key Entities (command registry, templates, namespaces)
7. Run Review Checklist
   � No implementation details included, focused on user capabilities
8. Return: SUCCESS (spec ready for planning)
```

---

## � Quick Guidelines
-  Focus on WHAT users need and WHY
- L Avoid HOW to implement (no tech stack, APIs, code structure)
- =e Written for business stakeholders, not developers

---

## User Scenarios & Testing *(mandatory)*

### Primary User Story
As a Codex CLI user, I want to create and use custom slash commands that execute predefined prompts with dynamic arguments, so I can reuse common prompt patterns and workflows without retyping them. The commands should work like Claude Code's custom commands, allowing me to organize templates in directories and invoke them with arguments that get interpolated into the prompt text.

### Acceptance Scenarios
1. **Given** I have Markdown files in `.codex/commands/` or `~/.codex/commands/`, **When** I type `/help`, **Then** I see my custom commands listed alongside built-in commands with scope labels
2. **Given** I create a command template with YAML frontmatter, **When** I invoke the command with arguments, **Then** the arguments are interpolated using `$ARGUMENTS` and `$1`, `$2`, etc. placeholders
3. **Given** I have a command template with `model: gpt-4o` in frontmatter, **When** I execute the command, **Then** that model is used for this turn only and the session reverts to the previous model afterward
4. **Given** I have both user and project commands with the same name, **When** I type the unqualified name, **Then** I'm prompted to disambiguate or must use the qualified `/project:name` or `/user:name` syntax
5. **Given** I organize commands in subdirectories, **When** I invoke them, **Then** the directory structure becomes part of the command namespace (e.g., `/project:web:deploy`)

### Edge Cases
- What happens when a command template contains `!bash command` or `@file.txt` syntax? (Should be treated as plain text)
- How does the system handle missing command files after initial discovery?
- What occurs when command templates have malformed YAML frontmatter?
- How are conflicting command names across user/project scopes resolved?

## Requirements *(mandatory)*

### Functional Requirements
- **FR-001**: System MUST discover Markdown command templates from `.codex/commands` (project scope) and `~/.codex/commands` (user scope) directories recursively
- **FR-002**: System MUST support command namespacing where subdirectories become namespace components in the command name
- **FR-003**: System MUST parse YAML frontmatter from templates to extract description, argument-hint, and model metadata
- **FR-004**: Users MUST be able to invoke commands using `/commandname [arguments]` syntax in the REPL
- **FR-005**: System MUST interpolate `$ARGUMENTS` with all command arguments and `$1`, `$2`, etc. with positional arguments
- **FR-006**: System MUST inject the interpolated template body as the next user message in the current conversation
- **FR-007**: System MUST apply model override from frontmatter for exactly one turn, then revert to session model
- **FR-008**: System MUST handle name conflicts by requiring fully qualified names (e.g., `/project:name` vs `/user:name`)
- **FR-009**: System MUST list custom commands in `/help` output with appropriate scope labels
- **FR-010**: System MUST treat `!command` and `@file` syntax in templates as plain text without execution or file inclusion
- **FR-011**: System MUST provide tab completion for command names and show argument hints when available
- **FR-012**: System MUST cache command registry and support reload via `/help reload`
- **FR-013**: System MUST maintain compatibility with existing REPL functionality for non-slash input
- **FR-014**: System MUST support optional environment variables `CODEX_SLASH_COMMANDS_DIR_USER` and `CODEX_SLASH_COMMANDS_DIR_PROJECT` to override default directories

### Key Entities *(include if feature involves data)*
- **Command Template**: Markdown file with optional YAML frontmatter containing description, argument-hint, model, and template body
- **Command Registry**: In-memory index mapping fully qualified command names to template metadata and content
- **Command Scope**: Either "user" (from `~/.codex/commands`) or "project" (from `.codex/commands`) indicating command origin
- **Command Namespace**: Hierarchical path derived from subdirectory structure within command directories
- **Interpolation Context**: Runtime data containing command arguments for template variable replacement

---

## Review & Acceptance Checklist
*GATE: Automated checks run during main() execution*

### Content Quality
- [x] No implementation details (languages, frameworks, APIs)
- [x] Focused on user value and business needs
- [x] Written for non-technical stakeholders
- [x] All mandatory sections completed

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