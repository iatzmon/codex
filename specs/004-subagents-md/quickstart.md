# Quickstart: Codex Subagents

1. Enable the feature flag:
   ```bash
   echo "[subagents]\nenabled = true" >> ~/.codex/config.toml
   ```
2. Create a project-scoped subagent at `.codex/agents/code-reviewer.md` with required frontmatter:
   ```markdown
   ---
   name: code-reviewer
   description: Reviews staged diffs for safety regressions
   tools:
     - git_diff
     - tests
   model: gpt-4.1-mini
   ---

   Provide review instructions here.
   ```
3. Launch Codex CLI and run the agents manager to inspect available subagents:
   ```bash
   codex agents list
   ```
4. Invoke the subagent explicitly in a session:
   ```text
   > Use the code-reviewer subagent to check the README updates.
   ```
5. Review the summarized result and open the detailed transcript through the manager UI:
   ```bash
   codex agents show code-reviewer --detail
   ```
