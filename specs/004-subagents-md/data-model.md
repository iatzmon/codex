# Phase 1 Data Model: Codex Subagents

## Entities

### SubagentDefinition
- Source: Markdown file with YAML frontmatter located under project `.codex/agents/` or user `~/.codex/agents/`.
- Fields:
  - `name` (string, required, kebab-case normalized)
  - `description` (string, required)
  - `tools` (array<string>, optional)
  - `model` (string, optional)
  - `scope` (enum: `project` | `user`, derived from path)
  - `source_path` (absolute path to definition file)
  - `validation_errors` (array<string>, populated when required fields missing or malformed)
- Relationships: Aggregated into a `SubagentInventory`; may override another definition with same normalized `name` when scope is `project`.
- Validation Rules:
  - Reject definitions missing `name` or `description`.
  - Enforce kebab-case normalization for lookup.
  - Ensure `tools` values map to registered tools when feature enabled.

### SubagentInventory
- Purpose: Flattened view of discovered subagents with precedence resolution.
- Fields:
  - `subagents` (map<string, SubagentRecord>) keyed by normalized name.
  - `conflicts` (array<ConflictRecord>) capturing user vs project collisions.
  - `discovery_events` (array<DiscoveryEvent>) for logging/auditing.
- Relationships: Consumed by CLI managers and invocation engine to present metadata and enforce restrictions.
- Validation Rules:
  - Project-level definitions must override user-level entries while logging the override event.
  - Conflicting definitions without a project override surface warnings.

### SubagentRecord
- Fields:
  - `definition` (SubagentDefinition)
  - `effective_tools` (array<string>, resolved from definition or inherited tool registry)
  - `effective_model` (string, resolved from definition → config default → session model)
  - `status` (enum: `active`, `invalid`, `disabled`)
- Behavior: Determines whether subagent can be invoked; invalid records block invocation and surface errors.

### SubagentConfig
- Fields:
  - `enabled` (bool, default `false`)
  - `default_model` (string | null)
  - `discovery` (enum: `auto` | `manual`)
- Relationship: Loaded from `~/.codex/config.toml`; consumed by both discovery and CLI presentation logic.

### InvocationSession
- Purpose: Isolated execution context for subagent runs.
- Fields:
  - `parent_session_id`
  - `subagent_name`
  - `execution_log`
  - `summary`
  - `detail_artifacts` (references to transcripts/logs accessible via manager UI)
- Constraints: Shares same tool registry as parent except when `effective_tools` restricts access.

## State Transitions
1. Discovery scans -> builds `SubagentInventory`.
2. CLI list command -> presents `SubagentRecord` data with override indicators.
3. Invocation request -> validates `SubagentRecord`, clones parent session, applies restricted tools/model, executes, emits summary.
4. Auto-suggestion engine -> matches parent prompt to `SubagentDefinition.description`; requires user confirmation before transition to invocation.

## Consistency Guarantees
- Inventory rebuild occurs on CLI startup and on demand when files change.
- Validation errors logged and stored to prevent repeated parsing penalties.
- Config toggle disablement short-circuits discovery and invocation to avoid accidental usage.
