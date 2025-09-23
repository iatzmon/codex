# Phase 0 Research: Codex Subagents Parity

## Decision Log

### Decision: Discover subagents from project and user scopes with project precedence
- Rationale: Matches FR-001 and Claude parity requirements while allowing workspace overrides without impacting personal defaults.
- Alternatives considered: (a) Single global registry only—rejected because it breaks project-specific overrides; (b) Deep merge of definitions—rejected due to ambiguity around conflicting tool/model settings.

### Decision: Represent subagent definitions as Markdown files with YAML frontmatter
- Rationale: Aligns with FR-002 and existing Claude Code semantics; keeps definitions human-reviewable and versioned.
- Alternatives considered: (a) Plain YAML files—rejected because current CLI expects Markdown docs; (b) TOML—rejected because spec mandates Markdown parity.

### Decision: Load tool permissions and models from definition with inheritance fallbacks
- Rationale: Satisfies FR-003 and FR-009 by allowing explicit restriction while maintaining default behavior when omissions occur.
- Alternatives considered: (a) Require explicit tools/models—rejected for ergonomics; (b) Automatic inference based on usage history—rejected as out of scope and introduces non-minimal changes.

### Decision: Extend agents manager to list, inspect, and create subagents
- Rationale: Delivers FR-004 and FR-008 by exposing precedence and metadata via the CLI without breaking existing workflows.
- Alternatives considered: (a) Introduce separate command namespace—rejected to keep changes surgical; (b) Manage via config-only editing—rejected because spec requires interactive discovery.

### Decision: Configure feature via `subagents.*` keys in `~/.codex/config.toml`
- Rationale: Implements FR-007 and keeps toggles centralized with other configuration flags.
- Alternatives considered: (a) Environment variables—rejected due to observability gaps; (b) project-only config—rejected because user-level defaults are required.

### Decision: Log validation errors and auto-suggestion prompts through existing CLI logging
- Rationale: Supports FR-008, preserves observability (Constitution V) without introducing new logging stacks.
- Alternatives considered: (a) Silent failures—rejected as they violate success criteria; (b) new telemetry pipeline—rejected as scope creep.

## Research Outcomes
- Unknowns resolved: None remaining; spec is explicit about fields, precedence, and user flows.
- Best practices recorded for: YAML frontmatter parsing, CLI command extensions, TOML configuration patterns within Codex.
- Next steps: Proceed to Phase 1 design artifacts using decisions above.
