# Quickstart — Validating Plan Mode Read-Only Planning State

1. **Activate Plan Mode**
   - Run `codex --plan` with a simple prompt.
   - Expect a PLAN badge and tooltip explaining read-only restrictions.
   - Verify telemetry/log indicates `PlanModeEntered` with previous approval mode.

2. **Attempt a write operation**
   - Request a file edit (e.g., "Modify README").
   - Confirm the agent refuses, appends a plan entry summarizing the requested change, and reiterates how to exit Plan Mode.

3. **Attempt shell execution**
   - Ask the agent to run `ls`.
   - Ensure execution is blocked, entry captured in the plan, and no command is queued for the sandbox.

4. **Use read-only tooling**
   - Ask for `cat Cargo.toml` or use file search.
   - Verify read-only tools respond normally while plan entries continue accumulating.

5. **Test missing template fallback**
   - Temporarily rename `/home/iatzmon/workspace/codex/.codex/plan.md` if present.
   - Re-enter Plan Mode and observe warning that defaults are being used, with guidance to restore the template.

6. **Exit without applying**
   - Run `/exit-plan`.
   - Confirm approval mode reverts to the pre-plan value and PLAN badge disappears.

7. **Apply plan**
   - Re-enter Plan Mode, accumulate at least two entries, then run `/apply-plan on-request`.
   - Validate the session leaves Plan Mode, injects the plan artifact into the next turn, and sets approval mode to the specified target.

8. **External attachment guardrail**
   - In Plan Mode, attempt to attach a file outside the workspace.
   - Expect refusal with explicit security messaging and no attempt to read the file.

9. **Snapshot checks**
   - Run `cargo test -p codex-tui -- chatwidget::` to update/validate TUI snapshots reflecting PLAN badges and refusal text.
