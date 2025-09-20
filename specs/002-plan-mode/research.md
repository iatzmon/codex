# Phase 0 Research — Plan Mode Read-Only Planning State

## Decision: `/apply-plan` fallback behavior
- **Decision**: When `/apply-plan` is invoked without an explicit mode, default to the approval policy that was active immediately before Plan Mode; if none is recoverable, prompt the operator to choose before exiting.
- **Rationale**: Maintains continuity with user expectations and avoids silently elevating privileges; reuses existing `AskForApproval` state captured in `EnvironmentContext`.
- **Alternatives Considered**:
  - *Always prompt for mode*: adds friction for the common case where the prior mode is safe and known.
  - *Default to on-request*: could override admin-configured policies and surprise operators accustomed to stricter or looser modes.

## Decision: Handling missing or unreadable `.codex/plan.md`
- **Decision**: If `/home/iatzmon/workspace/codex/.codex/plan.md` is absent or unreadable, emit a non-fatal warning in the transcript, proceed with built-in defaults, and surface guidance in the PLAN tooltip explaining how to add the template.
- **Rationale**: Preserves Plan Mode availability while highlighting configuration gaps; avoids blocking planning due to filesystem issues and aligns with existing template fallbacks in `codex-core`.
- **Alternatives Considered**:
  - *Fail Plan Mode activation*: prevents planning entirely and regresses usability for new installations.
  - *Silently ignore template*: loses discoverability for template customization and impairs UX.

## Decision: Refusal UX for disallowed actions
- **Decision**: Reuse the existing refusal messaging pipeline in `codex-core::user_notification` augmented with a Plan Mode context note ("Plan Mode is read-only; request captured in plan"), and append the proposed action to the active `UpdatePlanArgs` payload instead of executing.
- **Rationale**: Keeps messaging consistent with current refusals, leverages plan tool schema, and satisfies FR-003/FR-004/FR-015 without duplicating UI code.
- **Alternatives Considered**:
  - *Separate Plan Mode refusal component*: increases TUI complexity and risks divergent tone.
  - *Silent capture without notification*: fails acceptance criteria requiring explicit guidance.

## Decision: External attachment guardrail
- **Decision**: When attachments reference paths outside the approved workspace roots, block immediately, log the attempt for telemetry, and respond with a constitutional warning that instructs users to exit Plan Mode first.
- **Rationale**: Aligns with Security-First principle, fulfills FR-013, and reuses existing sandbox boundary checks in `codex-core::environment_context`.
- **Alternatives Considered**:
  - *Allow reads from outside roots*: violates scope constraint and increases risk.
  - *Defer decision until apply-plan*: delays necessary feedback and complicates plan review.

## Decision: MCP & web research tool gating
- **Decision**: Restrict Plan Mode tool registry to read-only capabilities by filtering `ToolRegistration` objects against a new `ToolCapability::ReadOnly` marker and require explicit operator approval before enabling any network-backed research tools.
- **Rationale**: Satisfies FR-005 and FR-014 with minimal change by extending the existing registry filtering logic.
- **Alternatives Considered**:
  - *Manual allowlist per tool name*: brittle and duplicates configuration.
  - *Disable MCP entirely*: blocks valuable read-only integrations.

## Decision: Telemetry for Plan Mode transitions
- **Decision**: Emit structured events via the existing analytics hook when Plan Mode is entered, exited, or when apply-plan succeeds, tagging with previous approval mode and counts of captured plan entries.
- **Rationale**: Provides observability for rollout without impacting runtime logic and supports success criteria validation.
- **Alternatives Considered**:
  - *No telemetry*: limits ability to monitor adoption and regressions.
  - *New telemetry pipeline*: unnecessary duplication of infrastructure.
