# Repository Guidelines

## Project Structure & Module Organization
The workspace splits into `codex-cli` for the Node.js command-line wrapper and `codex-rs` for the Rust core. Inside `codex-rs`, crates follow the `codex-*` prefix: `codex-core` hosts execution, plan mode, and configuration logic; `codex-tui` renders the interface; `codex-protocol`, `codex-mcp-client`, and other shared crates provide common libraries. Integration and regression tests live under `codex-rs/core/tests`, while UI snapshot cases reside in `codex-rs/tui/tests`. Documentation and assets sit in `docs/` and `.github/`.

## Build, Test, and Development Commands
Use `pnpm install` to set up Node tooling, then `pnpm build` in `codex-cli/` to stage the CLI bundle. Rust code is formatted with `just fmt`, linted via `just fix -p <crate>`, and built with `cargo build`. Run targeted tests such as `cargo test -p codex-core` or `cargo test -p codex-tui`. After touching shared crates (`codex-core`, `codex-common`, or `codex-protocol`), finish with `cargo test --all-features`. Snapshot workflows employ `cargo insta pending-snapshots -p codex-tui` and `cargo insta accept -p codex-tui` when updates are intentional.

## Coding Style & Naming Conventions
Rust files rely on `rustfmt` defaults; prefer compact imports and inlining variables directly inside `format!` braces. Keep module names snake_case and crates prefixed with `codex-`. In the TUI, use Ratatui’s `Stylize` helpers (`"text".dim()`, `url.cyan().underlined()`) instead of manual `Style` construction. Avoid introducing hard-coded white foregrounds; let the theme choose defaults.

## Testing Guidelines
Unit tests should use `pretty_assertions::assert_eq` for readable diffs. Follow the crate’s existing naming pattern (`mod tests` inside the module or dedicated files in `tests/`). Snapshot updates require reviewing the generated `.snap.new` files before accepting. Network-dependent checks rely on the guards defined in `codex-rs/core/src/spawn.rs` (`codex_core::spawn::CODEX_SANDBOX_ENV_VAR = "CODEX_SANDBOX"` and `codex_core::spawn::CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR = "CODEX_SANDBOX_NETWORK_DISABLED"`). Do not add or modify these guards in code; just leave the existing checks in place so sandboxes skip external calls.

## Commit & Pull Request Guidelines
Write commits in the imperative mood (e.g., `Add default plan-mode shell rules`) and keep them focused on a single concern. Pull requests should summarize the change, reference related issues, and call out user-facing impacts. Include test evidence (`cargo test -p …`, `cargo insta pending-snapshots`) in the PR description, and attach screenshots for TUI updates when visuals change.

## Subagents
Codex now supports project and user-scoped subagents. Enable the feature in `~/.codex/config.toml`,
then manage agents through the `codex agents` CLI or the interactive TUI. The transcript shows
override precedence, summaries, and detail URIs after each run. See `docs/subagents.md` for the
complete workflow and examples.

## Dialog Box Guidelines
- Route any approval-style prompts through `ApprovalRequest` and `UserApprovalWidget` so the TUI reuses the standard modal chrome.
- Prefer concise, actionable option labels (e.g., `Yes`, `No`, `Refine`) and back them with keyboard shortcuts that mirror existing dialogs.
- Surface relevant context—reason text, affected tools, models, next actions—inside the prompt body; keep it under ~6 lines to avoid pushing the modal off-screen.
- Always send a follow-up `AppEvent::CodexOp` (or equivalent) so the core receives a structured decision, and add a short history cell summarizing the user’s choice.
- When introducing a new dialog, add unit tests to the owning widget to confirm keyboard shortcuts and emitted events remain stable.
