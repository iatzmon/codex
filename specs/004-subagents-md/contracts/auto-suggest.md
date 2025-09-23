# Contract: Subagent Auto-Suggestion

## Purpose
Offer relevant subagent suggestions when the main session prompt aligns with a subagent description.

## Trigger
- Parent message classification identifies intent overlap with one or more `SubagentDefinition.description` values.
- `subagents.discovery` set to `auto`.

## Response Schema
```json
{
  "suggestions": [
    {
      "name": "code-reviewer",
      "confidence": 0.84,
      "reason": "Detected keywords: review, diff, regression",
      "requires_confirmation": true
    }
  ]
}
```

## Confirmation Flow
1. Present suggestions with reason codes.
2. Require explicit `yes`/`run` confirmation before invoking.
3. Respect `subagents.discovery = manual` by skipping suggestions entirely.

## Failure Modes
- No matches → return empty list without blocking message flow.
- Feature disabled → skip suggestion pipeline silently.

## Contract Tests
- `auto_suggest_requires_confirmation`: Expect no invocation without confirmation.
- `auto_suggest_respects_manual_mode`: When `subagents.discovery = manual`, expect zero suggestions even on matching prompts.
