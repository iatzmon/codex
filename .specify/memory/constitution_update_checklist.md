# Constitution Update Checklist

When amending the constitution (`/memory/constitution.md`), ensure all dependent documents are updated to maintain consistency.

## Templates to Update

### When adding/modifying ANY article:
- [ ] `/templates/plan-template.md` - Update Constitution Check gates and version banner.
- [ ] `/templates/spec-template.md` - Ensure checklists reflect new requirements.
- [ ] `/templates/tasks-template.md` - Align task categories and validation checklist.
- [ ] `/templates/agent-file-template.md` - Confirm generated guidance stays accurate.
- [ ] `/AGENTS.md` & related runtime docs - Propagate principle wording for human operators.

### Principle-specific updates:

#### Principle I (Dual-Core Workspace Integrity):
- [ ] Reconfirm documentation and tasks keep `codex-cli` UI work separate from `codex-rs` core changes.
- [ ] Update examples that reference directory layout or crate naming.

#### Principle II (Template-Governed Flow):
- [ ] Verify every template references mandatory Constitution Check checkpoints.
- [ ] Ensure agent guidance reminds operators to remove unused template sections.

#### Principle III (Test-First Assurance):
- [ ] Document required test ordering, `cargo`/`pnpm` commands, and snapshot review steps.
- [ ] Add explicit gates for providing failing tests before implementation.

#### Principle IV (Style and Simplicity Discipline):
- [ ] Call out `just fmt`, `just fix -p <crate>`, Ratatui `Stylize`, and YAGNI expectations where relevant.
- [ ] Remove or update examples that encourage unnecessary layering.

#### Principle V (Release and Observability Control):
- [ ] Reiterate semantic versioning triggers and required logging/documentation touchpoints.
- [ ] Ensure PR/commit guidance includes imperative mood and test evidence expectations.

## Validation Steps

1. **Before committing constitution changes:**
   - [ ] All templates reference new requirements
   - [ ] Examples updated to match new rules
   - [ ] No contradictions between documents

2. **After updating templates:**
   - [ ] Run through a sample implementation plan
   - [ ] Verify all constitution requirements addressed
   - [ ] Check that templates are self-contained (readable without constitution)

3. **Version tracking:**
   - [ ] Update constitution version number
   - [ ] Note version in template footers
   - [ ] Add amendment to constitution history

## Common Misses

Watch for these often-forgotten updates:
- Command documentation (`/commands/*.md`)
- Checklist items in templates
- Example code/commands
- Domain-specific variations (web vs mobile vs CLI)
- Cross-references between documents

## Template Sync Status

Last sync check: 2025-09-20
- Constitution version: 3.0.0
- Templates aligned: ✅ (plan/spec/tasks templates updated 2025-09-20)

---

*This checklist ensures the constitution's principles are consistently applied across all project documentation.*
