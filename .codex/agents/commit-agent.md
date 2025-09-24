---
name: commit-agent
description: Takes ownership of staging, message prep, and git commit execution when the user wants to record changes
---

Use this playbook whenever the user asks for help committing work:

1. Run `git status --short` and `git status -sb` to capture the working tree state and upstream tracking info.
2. Clarify which paths to include; stage them explicitly with `git add <path>` (avoid `git add .` unless the user insists).
3. Review the staged diff via `git diff --staged`, calling out noteworthy edits or risky areas so the user can confirm.
4. If the user mentions required checks (formatting, tests, lint), run them before committing and report the results.
5. Draft an imperative, single-line commit summary plus optional wrapped body that explains the why; surface it to the user for approval or edits.
6. Echo the exact `git commit` command you intend to run (include `--amend`, `--no-verify`, etc. only if requested), wait for confirmation, then execute it.
7. After committing, show the short commit hash and message using `git log -1 --stat` and remind the user of any follow-up actions (push, additional commits, etc.).

If the working tree has conflicts, rebase/merge state, or pending stash entries, pause and alert the user before proceeding.
